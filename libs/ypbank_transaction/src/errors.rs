//! # Доступные ошибки для библиотеки ypbank

use std::error::Error;
use std::fmt;

impl Error for ReadError {}

/// Ошибка чтения данных из файла
#[derive(Debug)]
pub struct ReadError {
    pub reason: String,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to read information from read buffer, reason: {}",
            self.reason
        )
    }
}

impl Error for WriteError {}

/// Ошибка записи данных в файл
#[derive(Debug)]
pub struct WriteError {
    pub reason: String,
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to write information from file, reason: {}",
            self.reason
        )
    }
}

impl Error for ConvertFromStrToU8Error {}

/// Ошибка конвертации строки в u8
#[derive(Debug)]
pub struct ConvertFromStrToU8Error {
    pub value: String,
    pub which_struct: String,
}

impl fmt::Display for ConvertFromStrToU8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to convert value {} to compatible value of {}",
            self.value, self.which_struct
        )
    }
}

impl Error for GetReaderError {}

/// Ошибка получения ридера
#[derive(Debug)]
pub struct GetReaderError {
    pub value: String,
}

impl fmt::Display for GetReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to get reader for format {}", self.value)
    }
}

impl Error for GetWriterError {}

#[derive(Debug)]
/// Ошибка получения врайтера
pub struct GetWriterError {
    pub value: String,
}

impl fmt::Display for GetWriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unable to get writer for format {}", self.value)
    }
}
