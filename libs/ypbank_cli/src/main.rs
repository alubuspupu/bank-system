use regex::Regex;
use std::collections::HashMap;
use std::env;
use ypbank_transaction::{BaseReader, DataValues, YPBankReader};

use std::fs::File;

fn get_reader(format_name: &String, file_name: &String) -> Result<YPBankReader<File>, String> {
    let file =
        File::open(file_name).map_err(|_| format!("Can't open file via path {}", file_name))?;

    match ypbank_transaction::YPBankReader::get_reader(format_name.to_string(), file) {
        Err(e) => Err(format!("{}", e)),
        Ok(r) => Ok(r),
    }
}

fn get_readers(args: &[String]) -> Result<HashMap<&String, YPBankReader<File>>, String> {
    if args.len() < 5 || !(args.len() - 1).is_multiple_of(4) {
        return Err("Usage: --file1 <path> --format1 <fmt> [--file2 ...]".to_string());
    }

    let reg_file = Regex::new(r"^--file\d+$").map_err(|_| "Invalid regex")?;
    let reg_format = Regex::new(r"^--format\d+$").map_err(|_| "Invalid regex")?;

    let mut readers = HashMap::new();
    let mut file_paths = Vec::new();
    let mut formats = Vec::new();

    for i in 1..args.len() {
        if reg_file.is_match(&args[i]) {
            if i + 1 >= args.len() {
                return Err(format!("No path provided for {}", args[i]));
            }
            file_paths.push((&args[i], &args[i + 1]));
        } else if reg_format.is_match(&args[i]) {
            if i + 1 >= args.len() {
                return Err(format!("No format provided for {}", args[i]));
            }
            formats.push((&args[i], &args[i + 1]));
        }
    }

    file_paths.sort_by_key(|(arg, _)| extract_id(arg));
    formats.sort_by_key(|(arg, _)| extract_id(arg));

    if file_paths.len() != formats.len() {
        return Err("Mismatched number of --file and --format arguments".to_string());
    }

    for ((file_arg, file_path), (format_arg, format)) in file_paths.into_iter().zip(formats) {
        let file_id = extract_id(file_arg);
        let format_id = extract_id(format_arg);

        if file_id != format_id {
            return Err(format!(
                "File {} has no corresponding format or mismatched IDs",
                file_path
            ));
        }

        let reader = get_reader(format, file_path)?;
        readers.insert(file_path, reader);
    }

    Ok(readers)
}

fn extract_id(s: &str) -> usize {
    let re = Regex::new(r"\d+").expect("Valid regex pattern");
    re.find(s)
        .and_then(|m| m.as_str().parse::<usize>().ok())
        .unwrap_or(0)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut readers: HashMap<&String, YPBankReader<File>> = match get_readers(args.as_slice()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    let readers_len = readers.len();
    let zero_sample: Vec<Option<DataValues>> = (0..readers_len).map(|_| None).collect();

    loop {
        let mut current_data_name = Vec::new();
        let mut current_data = Vec::new();
        let mut errors = Vec::new();

        for (name, reader) in readers.iter_mut() {
            match reader.read() {
                Ok(None) => {
                    errors.push(name);
                }
                Ok(data) => {
                    current_data.push(data);
                    current_data_name.push(name);
                }
                Err(_) => {
                    errors.push(name);
                }
            }
        }

        if errors.len() == readers_len || current_data == zero_sample {
            break;
        }

        if !errors.is_empty() {
            eprintln!("Error reading from files: {:?}", errors);
            return;
        }

        if let (Some(first), Some(name_first)) = (current_data.first(), current_data_name.first()) {
            for (data, file_name) in current_data[1..].iter().zip(&current_data_name[1..]) {
                if data != first {
                    let val_str = match &data {
                        None => "None".to_string(),
                        Some(vals) => vals.as_record().join(","),
                    };
                    println!(
                        "File {} has incompatible data with first file {}: value {}",
                        file_name, name_first, val_str
                    );
                    return;
                }
            }
        }
    }

    let names: Vec<String> = readers.keys().map(|s| s.to_string()).collect();
    let names_joined = names.join(" and ");
    println!(
        "The transaction records in {}  are identical.",
        names_joined
    );
}
