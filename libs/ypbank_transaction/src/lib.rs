//! # ypbank_transaction библиотека
//! *Предназначена для чтения, парсинга и анализа данных файлов типа txt, bin, csv*

mod bin_format;
mod csv_format;
pub mod errors;
mod txt_format;

use core::fmt;
use std::io::Read;
use std::io::Write;

use errors::ReadError;
use errors::WriteError;

/// ## Стурктура данных
/// Описывает общий вид транкзакции
#[derive(PartialEq, Eq)]
pub struct DataValues {
    /// Идентификатор транзакции
    tx_id: String,
    /// Тип транзакции
    tx_type: String,
    /// Идентификатор отправителя
    from_user_id: String,
    /// Идентификатор получателя
    to_user_id: String,
    /// Сумма
    amount: String,
    /// Время транзакции
    timestamp: String,
    /// Статус
    status: String,
    /// Описание
    description: Option<String>,
}

impl DataValues {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        tx_id: String,
        tx_type: String,
        from_user_id: String,
        to_user_id: String,
        amount: String,
        timestamp: String,
        status: String,
        description: Option<String>,
    ) -> Self {
        Self {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description,
        }
    }

    /// # Метод для конвертации в Vec<&str> структуры
    pub fn as_record(&self) -> Vec<&str> {
        vec![
            &self.tx_id,
            &self.tx_type,
            &self.from_user_id,
            &self.to_user_id,
            &self.amount,
            &self.timestamp,
            &self.status,
            if self.description.is_none() {
                ""
            } else {
                self.description.as_deref().unwrap()
            },
        ]
    }
}

impl fmt::Display for DataValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lines = vec![
            format!("TX_ID: {}", self.tx_id),
            format!("TX_TYPE: {}", self.tx_type),
            format!("FROM_USER_ID: {}", self.from_user_id),
            format!("TO_USER_ID: {}", self.to_user_id),
            format!("AMOUNT: {}", self.amount),
            format!("TIMESTAMP: {}", self.timestamp),
            format!("STATUS: {}", self.status),
        ];

        if let Some(ref desc) = self.description
            && !desc.is_empty()
        {
            lines.push(format!("DESCRIPTION: \"{}\"", desc));
        }

        write!(f, "{}", lines.join(","))
    }
}

/// Трейт для чтения данных из файла
pub trait BaseReader {
    fn read(&mut self) -> Result<Option<DataValues>, ReadError>;
}

/// Трейт для записи данных в файл
pub trait BaseWriter {
    fn write(&mut self, data_values: &DataValues) -> Result<(), WriteError>;
}

/// Единое перечисление со всеми доступными типами файлов для дайльнейшего поулчения нужного ридера
#[allow(clippy::large_enum_variant)]
pub enum YPBankReader<R: Read> {
    CSV(csv_format::YPBankCsvReader<R>),
    TXT(txt_format::YPBankTxtReader<R>),
    BIN(bin_format::YPBankBinReader<R>),
}

impl<R: Read> YPBankReader<R> {
    /// Метод получения ридера для нужного формата
    pub fn get_reader(
        format: String,
        reader: R,
    ) -> Result<YPBankReader<R>, errors::GetReaderError> {
        match format.as_str() {
            "csv" => Ok(YPBankReader::CSV(csv_format::YPBankCsvReader::new(reader))),
            "txt" => Ok(YPBankReader::TXT(txt_format::YPBankTxtReader::new(reader))),
            "bin" => Ok(YPBankReader::BIN(bin_format::YPBankBinReader::new(reader))),
            _ => Err(errors::GetReaderError { value: format }),
        }
    }
}

impl<R: Read> BaseReader for YPBankReader<R> {
    /// Метод для чтения данных из файла
    fn read(&mut self) -> Result<Option<DataValues>, errors::ReadError> {
        match self {
            YPBankReader::CSV(reader) => reader.read(),
            YPBankReader::TXT(reader) => reader.read(),
            YPBankReader::BIN(reader) => reader.read(),
        }
    }
}

/// Единое перечисление со всеми доступными типами файлов для дальнейшего получения нужного врайтера
#[allow(clippy::large_enum_variant)]
pub enum YPBankWriter<W: Write> {
    CSV(csv_format::YPBankCsvWriter<W>),
    TXT(txt_format::YPBankTxtWriter<W>),
    BIN(bin_format::YPBankBinWriter<W>),
}

impl<W: Write> YPBankWriter<W> {
    /// Метод получения врайтера для нужного формата
    pub fn get_writer(
        format: String,
        writer: W,
    ) -> Result<YPBankWriter<W>, errors::GetWriterError> {
        match format.as_str() {
            "csv" => Ok(YPBankWriter::CSV(csv_format::YPBankCsvWriter::new(writer))),
            "txt" => Ok(YPBankWriter::TXT(txt_format::YPBankTxtWriter::new(writer))),
            "bin" => Ok(YPBankWriter::BIN(bin_format::YPBankBinWriter::new(writer))),
            _ => Err(errors::GetWriterError { value: format }),
        }
    }
}

impl<W: Write> BaseWriter for YPBankWriter<W> {
    /// Метод для записи данных в файл
    fn write(&mut self, data: &DataValues) -> Result<(), errors::WriteError> {
        match self {
            YPBankWriter::CSV(writer) => writer.write(data),
            YPBankWriter::TXT(writer) => writer.write(data),
            YPBankWriter::BIN(writer) => writer.write(data),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_values_as_record_data_values() {
        let data_values = DataValues::new(
            "1234567890123456".to_string(),
            "DEPOSIT".to_string(),
            "0".to_string(),
            "9876543210987654".to_string(),
            "10000".to_string(),
            "1633036800000".to_string(),
            "SUCCESS".to_string(),
            Some("Test deposit transaction".to_string()),
        );

        let record = data_values.as_record();
        assert_eq!(
            record,
            vec![
                "1234567890123456",
                "DEPOSIT",
                "0",
                "9876543210987654",
                "10000",
                "1633036800000",
                "SUCCESS",
                "Test deposit transaction"
            ]
        );
    }

    #[test]
    fn test_data_values_as_record_without_description() {
        let data = DataValues::new(
            "1234567890123456".to_string(),
            "DEPOSIT".to_string(),
            "0".to_string(),
            "9876543210987654".to_string(),
            "10000".to_string(),
            "1633036800000".to_string(),
            "SUCCESS".to_string(),
            None,
        );

        let record = data.as_record();
        assert_eq!(
            record,
            vec![
                "1234567890123456",
                "DEPOSIT",
                "0",
                "9876543210987654",
                "10000",
                "1633036800000",
                "SUCCESS",
                ""
            ]
        );
    }

    #[test]
    fn test_ypbank_reader_creation() {
        let data = Vec::new();
        let cursor = std::io::Cursor::new(data);

        let reader = YPBankReader::get_reader("csv".to_string(), cursor).unwrap();
        match reader {
            YPBankReader::CSV(_) => {} // OK
            _ => panic!("Expected CSV variant"),
        }
    }

    #[test]
    fn test_ypbank_reader_invalid_format() {
        let data: Vec<u8> = Vec::new();
        let cursor = std::io::Cursor::new(data);
        let result = YPBankReader::get_reader("xml".to_string(), cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_ypbank_writer_creation() {
        let buffer = Vec::new();
        let writer = YPBankWriter::get_writer("txt".to_string(), buffer).unwrap();
        match writer {
            YPBankWriter::TXT(_) => {} // OK
            _ => panic!("Expected TXT variant"),
        }
    }

    #[test]
    fn test_ypbank_writer_invalid_format() {
        let buffer = Vec::new();
        let result = YPBankWriter::get_writer("html".to_string(), buffer);
        assert!(result.is_err());
    }
}
