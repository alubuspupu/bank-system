use std::{env, io::StdoutLock};

use ypbank_transaction::{BaseReader, BaseWriter, YPBankWriter};

use std::{fs::File, io::BufWriter};

use ypbank_transaction::YPBankReader;

fn get_reader(args: Vec<String>) -> Result<YPBankReader<File>, String> {
    if args.len() != 7 {
        return Err("Not enough arguments".to_string());
    }

    let mut i = match "--input" {
        val if val == args[1].as_str() => 2,
        val if val == args[3].as_str() => 4,
        val if val == args[5].as_str() => 6,
        _ => return Err("No reader's argument".to_string()),
    };

    let file = File::open(&args[i]).map_err(|_| format!("Can't open file via path {}", args[1]))?;

    i = match "--in-format" {
        val if val == args[1].as_str() => 2,
        val if val == args[3].as_str() => 4,
        val if val == args[5].as_str() => 6,
        _ => return Err("No reader's argument".to_string()),
    };

    match ypbank_transaction::YPBankReader::get_reader(args[i].to_string(), file) {
        Err(e) => Err(format!("{}", e)),
        Ok(r) => Ok(r),
    }
}

fn get_writer(
    args: Vec<String>,
    writer: BufWriter<StdoutLock<'_>>,
) -> Result<YPBankWriter<BufWriter<StdoutLock<'_>>>, String> {
    if args.len() != 7 {
        return Err("Not enough arguments".to_string());
    }
    let i = match "--output-format" {
        val if val == args[1].as_str() => 2,
        val if val == args[3].as_str() => 4,
        val if val == args[5].as_str() => 6,
        _ => return Err("No reader's argument".to_string()),
    };

    let output_format = Some(&args[i]);

    let format = output_format.ok_or("No --output-format argument")?;

    match ypbank_transaction::YPBankWriter::get_writer(format.clone(), writer) {
        Err(e) => {
            return Err(format!("Impossibe to get writer for format {}", e))?;
        }
        Ok(w) => Ok(w),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut reader = match get_reader(args.clone()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    let stdout = std::io::stdout();
    let locked_stdout = stdout.lock();
    let buffered_stdout = BufWriter::new(locked_stdout);

    let mut writer = match get_writer(args, buffered_stdout) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    while let Ok(Some(data)) = reader.read() {
        let res = writer.write(&data);

        if res.is_err() {
            println!("Can't write to buffer");
        }
    }
}
