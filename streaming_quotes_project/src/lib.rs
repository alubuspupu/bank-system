use serde::{Deserialize, Serialize};
use std::{fmt, fs, path::Path};
pub mod errors;
use errors::FileReadError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

impl fmt::Display for StockQuote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}",
            self.ticker, self.price, self.volume, self.timestamp
        )
    }
}

impl StockQuote {
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() == 4 {
            Some(StockQuote {
                ticker: parts[0].to_string(),
                price: parts[1].parse().ok()?,
                volume: parts[2].parse().ok()?,
                timestamp: parts[3].parse().ok()?,
            })
        } else {
            None
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.ticker.as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.price.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.volume.to_string().as_bytes());
        bytes.push(b'|');
        bytes.extend_from_slice(self.timestamp.to_string().as_bytes());
        bytes
    }

    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(_) => "".to_string(),
        }
    }
}
pub struct Validators {}

impl Validators {
    pub fn validate_ip(ip: &str) -> bool {
        match regex::Regex::new(
            r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$",
        ) {
            Ok(re) => re.is_match(ip),
            Err(_) => false,
        }
    }

    pub fn validate_port(port: &str) -> bool {
        match regex::Regex::new(
            r"^(?:[1-9]\d{0,3}|[1-5]\d{4}|6[0-4]\d{3}|65[0-4]\d{2}|655[0-2]\d|6553[0-5])$",
        ) {
            Ok(re) => re.is_match(port),
            Err(_) => false,
        }
    }

    pub fn validate_addr(addr: &str) -> bool {
        let re = regex::Regex::new(r"^([^:\s]+):(\d+)$").expect("Invalid regex");
        let caps = match re.captures(addr) {
            Some(caps) => caps,
            None => return false,
        };

        let host = match caps.get(1) {
            Some(m) => m.as_str(),
            None => return false,
        };
        let port = match caps.get(2) {
            Some(m) => m.as_str(),
            None => return false,
        };

        if !Self::validate_port(port) {
            return false;
        }

        Self::validate_ip(host) || ["localhost", "127.0.0.1", "0.0.0.0"].contains(&host)
    }
}

pub fn read_data_from_file(file_path: &str) -> Result<String, FileReadError> {
    if !Path::new(file_path).exists() {
        return Err(FileReadError {
            path: file_path.to_string(),
            reason: "file not exists".to_string(),
        });
    }

    match fs::read_to_string(file_path) {
        Err(_) => Err(FileReadError {
            path: file_path.to_string(),
            reason: "impossible to read data from file".to_string(),
        }),
        Ok(data) => Ok(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qutote_to_string() {
        let quote = StockQuote {
            ticker: "AAPL".to_string(),
            price: 150.0,
            volume: 1000,
            timestamp: 1625000000,
        };

        assert_eq!(quote.to_string(), "AAPL|150|1000|1625000000");
    }

    #[test]
    fn test_quote_from_string() {
        let quote = StockQuote {
            ticker: "AAPL".to_string(),
            price: 150.0,
            volume: 1000,
            timestamp: 1625000000,
        };

        assert_eq!(
            StockQuote::from_string("AAPL|150|1000|1625000000").unwrap(),
            quote
        );
    }

    #[test]
    fn test_quote_to_bytes() {
        let quote = StockQuote {
            ticker: "AAPL".to_string(),
            price: 150.0,
            volume: 1000,
            timestamp: 1625000000,
        };

        let encoded: Vec<u8> = vec![
            65, 65, 80, 76, 124, 49, 53, 48, 124, 49, 48, 48, 48, 124, 49, 54, 50, 53, 48, 48, 48,
            48, 48, 48,
        ];

        assert_eq!(quote.to_bytes(), encoded);
    }

    #[test]
    fn test_from_json_valid() {
        let json = r#"{"ticker":"AAPL","price":150.0,"volume":1000,"timestamp":1625000000}"#;
        let quote = StockQuote::from_json(json);
        assert!(quote.is_some());
        let quote = quote.unwrap();
        assert_eq!(quote.ticker, "AAPL");
        assert_eq!(quote.price, 150.0);
        assert_eq!(quote.volume, 1000);
        assert_eq!(quote.timestamp, 1625000000);
    }

    #[test]
    fn test_from_json_invalid() {
        let json = r#"{"text": "invalid json"#;
        let quote = StockQuote::from_json(json);
        assert!(quote.is_none());
    }

    #[test]
    fn test_to_json() {
        let quote = StockQuote {
            ticker: "AAPL".to_string(),
            price: 150.0,
            volume: 1000,
            timestamp: 1625000000,
        };
        let json = quote.to_json();
        assert!(!json.is_empty());
        assert!(json.contains(r#""ticker":"AAPL""#));
        assert!(json.contains(r#""price":150.0"#));
        assert!(json.contains(r#""volume":1000"#));
        assert!(json.contains(r#""timestamp":1625000000"#));
    }

    #[test]
    fn test_validate_ip_valid() {
        assert!(Validators::validate_ip("192.168.1.1"));
        assert!(Validators::validate_ip("127.0.0.1"));
        assert!(Validators::validate_ip("255.255.255.255"));
        assert!(Validators::validate_ip("0.0.0.0"));
        assert!(Validators::validate_ip("1.1.1.1"));
    }

    #[test]
    fn test_validate_ip_invalid() {
        assert!(!Validators::validate_ip("256.1.1.1"));
        assert!(!Validators::validate_ip("192.168.0.256"));
        assert!(!Validators::validate_ip("192.168.0.-1"));
        assert!(!Validators::validate_ip("192.168.0"));
        assert!(!Validators::validate_ip("192.168.0."));
        assert!(!Validators::validate_ip("abc.def.ghi.jkl"));
        assert!(!Validators::validate_ip(""));
    }

    #[test]
    fn test_validate_port_valid() {
        assert!(Validators::validate_port("1"));
        assert!(Validators::validate_port("80"));
        assert!(Validators::validate_port("443"));
        assert!(Validators::validate_port("1024"));
        assert!(Validators::validate_port("65535"));
        assert!(Validators::validate_port("3000"));
    }

    #[test]
    fn test_validate_port_invalid() {
        assert!(!Validators::validate_port("65536"));
        assert!(!Validators::validate_port("70000"));
        assert!(!Validators::validate_port("-1"));
        assert!(!Validators::validate_port("abc"));
        assert!(!Validators::validate_port(""));
        assert!(!Validators::validate_port("8080a"));
    }

    #[test]
    fn test_validate_addr_valid() {
        assert!(Validators::validate_addr("192.168.1.1:8080"));
    }

    #[test]
    fn test_validate_addr_invalid() {
        assert!(!Validators::validate_addr("bad.ip.address:8080"));
        assert!(!Validators::validate_addr("192.168.1.1:65536"));
        assert!(!Validators::validate_addr("192.168.1.1"));
        assert!(!Validators::validate_addr("192.168.1.1:"));
        assert!(!Validators::validate_addr(":8080"));
        assert!(!Validators::validate_addr("192.168.1.1:80:extra"));
        assert!(!Validators::validate_addr(""));
        assert!(!Validators::validate_addr("localhost"));
    }
}
