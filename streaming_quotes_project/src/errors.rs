//! # Доступные ошибки для библиотеки stream_quotes

use std::error::Error;
use std::fmt;

impl Error for ArgumentError {}

/// Ошибка обработки аргументов
#[derive(Debug)]
pub struct ArgumentError {
    pub name: String,
    pub reason: String,
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error in arguments handeling, name: {}, reason: {}",
            self.name, self.reason
        )
    }
}

impl Error for FileReadError {}

/// Ошибка обработки аргументов при запуске клиента/сервера
#[derive(Debug)]
pub struct FileReadError {
    pub path: String,
    pub reason: String,
}

impl fmt::Display for FileReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error in file handeling, name: {}, reason: {}",
            self.path, self.reason
        )
    }
}

impl Error for RequestParamError {}

/// Ошибка при обработке параметров, переданных при запросе
#[derive(Debug)]
pub struct RequestParamError {
    pub name: String,
    pub value: String,
}

impl fmt::Display for RequestParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error param in request, name: {}, reason: {}",
            self.name, self.value
        )
    }
}

impl Error for ReplyError {}

/// Ошибка при ответе
#[derive(Debug)]
pub struct ReplyError {
    pub name: String,
    pub value: String,
}

impl fmt::Display for ReplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "error param in request, name: {}, reason: {}",
            self.name, self.value
        )
    }
}
