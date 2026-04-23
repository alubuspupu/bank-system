use log::debug;
use log::error;
use log::info;
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use streaming_quotes_project::StockQuote;
use streaming_quotes_project::Validators;
use streaming_quotes_project::errors::RequestParamError;
use streaming_quotes_project::read_data_from_file;

static TIMEOUT: u64 = 5;

#[derive(Debug, Clone, Deserialize)]
pub struct GeneratorStock {
    pub ticker: String,
    pub current_price: f64,
    pub current_volume: u32,
}

impl GeneratorStock {
    pub fn new(ticker: &str, current_price: f64, current_volume: u32) -> Self {
        Self {
            ticker: ticker.to_string(),
            current_price,
            current_volume,
        }
    }

    // # Генератор следующей цены и объема акции
    /// Используется метод случайного блуждения
    pub fn next_ticker(&mut self) -> StockQuote {
        static VOLATILITY: f64 = 0.01;
        // Принимаю как факт, что волатильность цены акции составляет не более 1%
        let change = rand::random_range(-VOLATILITY..=VOLATILITY);
        self.current_price *= 1.0 + change;

        // Пусть объем скачет быстрее чем цена +-20%
        static VOLUME_CHANGE: f64 = 0.2;
        static MAX_RANGE: f64 = 1.0 + VOLUME_CHANGE;
        static MIN_RANGE: f64 = 1.0 - VOLUME_CHANGE;

        let vol_change = rand::random_range(MIN_RANGE..=MAX_RANGE);
        self.current_volume = (self.current_volume as f64 * vol_change) as u32;

        StockQuote {
            ticker: self.ticker.clone(),
            price: self.current_price,
            volume: self.current_volume,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(TIMEOUT))
                .as_millis() as u64,
        }
    }
}

pub trait DataLoader {
    fn load_data(&self) -> Result<Vec<GeneratorStock>, String>;
}
pub struct JsonLoader {
    pub filepath: String,
}

impl DataLoader for JsonLoader {
    fn load_data(&self) -> Result<Vec<GeneratorStock>, String> {
        match read_data_from_file(&self.filepath) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(generators) => Ok(generators),
                Err(err) => Err(format!("Error while parse tickets: {}", err)),
            },
            Err(err) => Err(format!("{}", err)),
        }
    }
}

pub struct QuoteGeneratorPublisher<L>
where
    L: DataLoader,
{
    subs: Arc<Mutex<HashMap<String, Vec<mpsc::Sender<StockQuote>>>>>,
    loader: L,
    supported_quote: Vec<GeneratorStock>,
    loaded: bool,
}

impl<L: DataLoader> QuoteGeneratorPublisher<L> {
    pub fn new(loader: L) -> Self {
        Self {
            loader,
            subs: Arc::new(Mutex::new(HashMap::new())),
            supported_quote: Vec::new(),
            loaded: false,
        }
    }

    pub fn load_data(&mut self) -> Result<(), String> {
        let data = self.loader.load_data()?;

        for stock in data {
            self.supported_quote.push(GeneratorStock {
                ticker: stock.ticker,
                current_price: stock.current_price,
                current_volume: stock.current_volume,
            });
        }

        self.loaded = true;

        info!("Data successed loaded");

        Ok(())
    }

    pub fn subscribe(&self, tickers: &Vec<&str>) -> Result<mpsc::Receiver<StockQuote>, String> {
        if !self.loaded {
            return Err("Data not loaded".to_string());
        }

        let (tx, rx) = mpsc::channel();

        for ticker in tickers {
            if !self.supported_quote.iter().any(|s| s.ticker == *ticker) {
                return Err(format!("Ticker {} not supported", ticker));
            }

            if let Ok(mut guard) = self.subs.lock() {
                guard
                    .entry(ticker.to_string())
                    .or_insert_with(Vec::new)
                    .push(tx.clone());
            } else {
                error!("Failed to lock subscription map: mutex poisoned");
            }

            info!("Subscribed to {}", ticker);
        }

        Ok(rx)
    }

    fn publish(&self, tick: StockQuote) -> Result<(), String> {
        if !self.loaded {
            return Err("Data not loaded".to_string());
        }

        if !self.supported_quote.iter().any(|s| s.ticker == tick.ticker) {
            return Err(format!("Ticker {} not supported", tick.ticker));
        }

        if let Ok(mut guard) = self.subs.lock() {
            if let Some(senders) = guard.get_mut(&tick.ticker) {
                senders.retain(|tx| tx.send(tick.clone()).is_ok());
            }
            debug!("Published {}", tick.ticker);

            return Ok(());
        };

        Err("Failed to lock subscription map: mutex poisoned".to_string())
    }

    pub fn update_tickers(&mut self) -> Result<(), String> {
        if !self.loaded {
            return Err("Data not loaded".to_string());
        }

        let new_quotes: Vec<StockQuote> = self
            .supported_quote
            .iter_mut()
            .map(|stock| stock.next_ticker())
            .collect();

        for quote in new_quotes {
            self.publish(quote)?;
        }

        Ok(())
    }
}

pub fn quote_thread(
    running_update: Arc<AtomicBool>,
    generator: Arc<Mutex<QuoteGeneratorPublisher<JsonLoader>>>,
) {
    while running_update.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(
            (rand::random_range(GEN_MIN_VALUE..=GEN_MAX_VALUE) as u32).into(),
        ));

        if let Ok(mut gen_lock) = generator.lock()
            && let Err(e) = gen_lock.update_tickers()
        {
            error!("Update error: {}", e);
        }
    }

    info!("Quote generator thread stopped.");
}

fn parse_upd_address(addr: &str) -> Result<(String, u32), RequestParamError> {
    let re = Regex::new(r"^udp://([^:\s]+:\d+)$").map_err(|_| RequestParamError {
        name: "udp regex".to_string(),
        value: "imposiblle parse".to_string(),
    })?;

    let host_port_full = match re.captures(addr) {
        Some(caps) => caps.get(1).map_or("", |m| m.as_str()),
        None => {
            return Err(RequestParamError {
                name: "udp addr".to_string(),
                value: addr.to_string(),
            });
        }
    };

    if !Validators::validate_addr(host_port_full) {
        return Err(RequestParamError {
            name: "udp address".to_string(),
            value: addr.to_string(),
        });
    }

    let mut parts = host_port_full.split(':');

    let host = match parts.next() {
        Some(h) => h,
        None => {
            return Err(RequestParamError {
                name: "udp host".to_string(),
                value: addr.to_string(),
            });
        }
    };

    let port_str = match parts.next() {
        Some(h) => h,
        None => {
            return Err(RequestParamError {
                name: "udp addr".to_string(),
                value: addr.to_string(),
            });
        }
    };

    if parts.next().is_some() {
        return Err(RequestParamError {
            name: "udp addr".to_string(),
            value: addr.to_string(),
        });
    }

    let port: u32 = port_str.parse().map_err(|_| RequestParamError {
        name: "udp port".to_string(),
        value: port_str.to_string(),
    })?;

    if port == 0 || port > 65535 {
        return Err(RequestParamError {
            name: "udp port not in [1, 65535]".to_string(),
            value: port_str.to_string(),
        });
    }

    Ok((host.to_string(), port))
}

fn handle_stream_command(
    addr: String,
    port: u32,
    tickers: &str,
    client_alive: Arc<AtomicBool>,
    generator: Arc<Mutex<QuoteGeneratorPublisher<JsonLoader>>>,
) -> Result<(), String> {
    let tickers_vec = tickers.split(',').map(|s| s.trim()).collect::<Vec<_>>();
    let send_addr = format!("{}:{}", addr, port);
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    socket.connect(&send_addr).map_err(|e| e.to_string())?;

    let receiver = generator
        .lock()
        .map_err(|_| "Failed to lock generator: mutex poisoned".to_string())?
        .subscribe(&tickers_vec)?;

    while client_alive.load(Ordering::SeqCst) {
        if let Ok(quote) = receiver.recv_timeout(Duration::from_secs(TIMEOUT)) {
            socket.send(quote.to_json().as_bytes()).ok();
        }
    }

    info!("Stream stopped: {}:{}", addr, port);

    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    generator: Arc<Mutex<QuoteGeneratorPublisher<JsonLoader>>>,
    global_running: Arc<AtomicBool>,
) {
    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(TIMEOUT))) {
        info!("Failed to set timeout: {}", e);
        return;
    }

    let stream_clone = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut reader = BufReader::new(stream_clone);
    let client_running = Arc::new(AtomicBool::new(true));
    let mut udp_thread = None;
    let mut last_ping = std::time::Instant::now();

    while global_running.load(Ordering::SeqCst) && client_running.load(Ordering::SeqCst) {
        let mut line = String::new();

        if last_ping.elapsed() > Duration::from_secs(TIMEOUT) {
            info!("Client timed out: no PING for {} seconds", TIMEOUT);
            break;
        }

        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                last_ping = std::time::Instant::now();

                let trimmed = line.trim().to_string();

                if trimmed.is_empty() {
                    continue;
                }

                let trimmed = line.trim().to_string(); // теперь trimmed владеет данными
                let mut parts = trimmed.split_whitespace();
                let command = parts.next().unwrap_or("");

                match command {
                    "EXIT" => break,
                    "PING" => {
                        if stream.write_all(b"PONG\n").is_err() {
                            break;
                        }
                    }
                    "STREAM" => {
                        if udp_thread.is_some() {
                            let _ = stream.write_all(b"ERR\n");
                        } else {
                            let addr_part = parts.next();
                            let tickers_part = parts.next();

                            match (addr_part, tickers_part) {
                                (Some(addr), Some(tickers)) => match parse_upd_address(addr) {
                                    Ok((host, port)) => {
                                        let gener = generator.clone();
                                        let running = client_running.clone();
                                        let tickers_owned = tickers.to_string();
                                        udp_thread = Some(thread::spawn(move || {
                                            let _ = handle_stream_command(
                                                host,
                                                port,
                                                &tickers_owned,
                                                running,
                                                gener,
                                            );
                                        }));
                                        let _ = stream.write_all(b"OK\n");
                                    }
                                    Err(_) => {
                                        let _ = stream.write_all(b"ERR Incoorect header \n");
                                    }
                                },
                                _ => {
                                    let _ = stream.write_all(b"ERR Incorrect header\n");
                                }
                            }
                        }
                    }
                    _ => {
                        let _ = stream.write_all(b"ERR Incorrect command\n");
                    }
                }
            }
            Err(e) => {
                error!("Met error in stream: {}", e);
                break;
            }
        }
    }

    client_running.store(false, Ordering::Relaxed);

    if let Some(handle) = udp_thread {
        let _ = handle.join();
    }

    info!("Closed connection");
}

fn get_listener(args: &[String]) -> Result<TcpListener, std::io::Error> {
    let ip = &args[1];
    let port = &args[2];

    if !Validators::validate_ip(ip) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid IP address: {}", ip),
        ));
    }

    if !Validators::validate_port(port) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid port: {}", port),
        ));
    }

    let addr = format!("{}:{}", ip, port);
    let listener = TcpListener::bind(&addr)
        .map_err(|e| std::io::Error::other(format!("Failed to bind to {}: {}", addr, e)));

    info!("binded addr: {}", addr);

    listener
}

const GEN_MIN_VALUE: u32 = 100;
const GEN_MAX_VALUE: u32 = 1000;

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        error!("Not enough arguments");
        return;
    }

    if !Path::new(&args[3]).exists() {
        error!("File not found: {}", args[3]);
        return;
    }

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let listener = match get_listener(&args) {
        Ok(listener) => listener,
        Err(e) => {
            error!("Impossible open listener, reason: {}", e);
            return;
        }
    };

    match ctrlc::set_handler(move || {
        info!("Received Ctrl+C! Shutting down...");
        r.store(false, Ordering::SeqCst);
    }) {
        Ok(_) => (),
        Err(err) => {
            error!("Impossible set crtl+c signal handler, reason: {}", err);
            return;
        }
    }

    let loader = JsonLoader {
        filepath: args[3].clone(),
    };
    let generator = Arc::new(Mutex::new(QuoteGeneratorPublisher::new(loader)));

    {
        if let Ok(mut gen_lock) = generator.lock() {
            let _ = gen_lock
                .load_data()
                .map_err(|e| error!("Load error: {}", e));
        } else {
            error!("Failed to lock generator: mutex poisoned")
        }
    }

    let gen_clone = Arc::clone(&generator);
    let running_update = running.clone();

    let quote_handle = thread::spawn(move || quote_thread(running_update, gen_clone));

    let mut client_handles: Vec<thread::JoinHandle<()>> = vec![];

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let local_generator = Arc::clone(&generator);
                let clone_running = running.clone();
                client_handles.push(thread::spawn(move || {
                    handle_client(stream, local_generator, clone_running);
                }));
            }
            Err(_) if !running.load(Ordering::SeqCst) => break,
            Err(e) => error!("Connection failed: {}", e),
        }
    }

    for handle in client_handles {
        let _ = handle.join();
    }

    let _ = quote_handle.join();
}
