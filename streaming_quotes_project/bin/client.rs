use clap::Parser;
use log::{error, info};
use socket2::{Domain, Protocol, Socket, Type};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::time::Duration;
use std::{
    io::{self, BufRead, BufReader, Error, ErrorKind, Write},
    net::{SocketAddr, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use streaming_quotes_project::StockQuote;
use streaming_quotes_project::{Validators, read_data_from_file};

static PAUSE: u64 = 2;
static TIMEOUT: u64 = 5;
static MAX_BUFFER_SIZE: usize = 65535;

#[derive(Parser, Debug)]
#[command(author, version, about = "Stock Quotes Client")]
struct Args {
    #[arg(long)]
    udp_port: u16,
    #[arg(long)]
    server_addr: String,
    #[arg(long)]
    tickers_file: String,
}

use std::net::UdpSocket;

fn spawn_udp_reader(port: u16, running: Arc<AtomicBool>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let addr = format!("0.0.0.0:{}", port);
        let socket = match UdpSocket::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                error!("Could not bind UDP socket on {}: {}", addr, e);
                return;
            }
        };

        if let Err(e) = socket.set_read_timeout(Some(Duration::from_secs(TIMEOUT))) {
            error!("Failed to set read timeout: {}", e);
            return;
        }

        let mut buf = [0u8; MAX_BUFFER_SIZE];
        info!("UDP Reader started on {}", addr);

        while running.load(Ordering::SeqCst) {
            match socket.recv_from(&mut buf) {
                Ok((amt, _src)) => {
                    if let Ok(text) = std::str::from_utf8(&buf[..amt]) {
                        if !running.load(Ordering::SeqCst) {
                            break;
                        }

                        if let Some(quote) = StockQuote::from_json(text) {
                            info!(
                                "time={}, ticker={}, price={:.2}, volume={}",
                                quote.timestamp, quote.ticker, quote.price, quote.volume
                            );
                        } else {
                            error!("Invalid quote JSON: {}", text);
                        }
                    } else {
                        error!("Invalid UTF-8 in UDP packet");
                    }
                }
                Err(e) => {
                    error!("UDP recv error: {}", e);
                    break;
                }
            }
        }

        info!("UDP reader stopped gracefully");
    })
}

fn connect(
    addr: String,
    tickers: String,
    port: u16,
) -> io::Result<(TcpStream, BufReader<TcpStream>)> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    let addr: SocketAddr = addr
        .parse()
        .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;

    socket.connect(&addr.into())?;

    let mut stream: TcpStream = socket.into();
    stream.set_read_timeout(Some(Duration::from_secs(TIMEOUT)))?;
    let mut reader = BufReader::new(stream.try_clone()?);

    stream.write_all(format!("STREAM udp://127.0.0.1:{} {}\n", port, tickers).as_bytes())?;
    stream.flush()?;

    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    let response = buffer.trim();

    if response != "OK" {
        error!("Server rejected STREAM: '{}'", response);
        return Err(Error::other(format!(
            "Server rejected STREAM: {}",
            response
        )));
    }

    info!("Connected to server!");
    Ok((stream, reader))
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    if !Validators::validate_port(&args.udp_port.to_string()) {
        error!("Invalid UDP port: {}", args.udp_port);
        return;
    }

    if !Validators::validate_addr(&args.server_addr) {
        error!("Invalid server address: {}", args.server_addr);
        return;
    }

    let tickers_content = match read_data_from_file(&args.tickers_file) {
        Ok(content) => content,
        Err(e) => {
            error!("Read tickers error: {}", e);
            return;
        }
    };

    let tickers_list: Vec<String> = tickers_content
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    if tickers_list.is_empty() {
        error!("No valid tickers found in file: {}", args.tickers_file);
        return;
    }

    let tickers = tickers_list.join(",");

    info!("Loaded {} tickers", tickers_list.len());

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        info!("Received Ctrl+C! Shutting down...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let (stream, _) = match connect(args.server_addr.clone(), tickers, args.udp_port) {
        Ok((s, r)) => (s, r),
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let stream = Arc::new(Mutex::new(stream));
    let udp_running = running.clone();
    let ping_running = running.clone();
    let udp_handle = spawn_udp_reader(args.udp_port, udp_running);

    let thread = std::thread::spawn(move || {
        while ping_running.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_secs(PAUSE));

            let mut s = stream.lock().unwrap();
            if let Err(e) = s.write_all(b"PING\n") {
                error!("Failed to send PING: {}", e);
                break;
            }
            s.flush().ok();
        }
    });

    thread.join().unwrap();
    udp_handle.join().unwrap();

    info!("Connection closed");
}
