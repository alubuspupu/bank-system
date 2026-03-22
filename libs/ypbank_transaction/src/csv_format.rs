use std::{io::Read, io::Write};

use crate::{
    BaseReader, BaseWriter, DataValues,
    errors::{ReadError, WriteError},
};

#[derive(serde::Deserialize)]
struct CsvDataValues {
    #[serde(rename = "TX_ID")]
    tx_id: String,
    #[serde(rename = "TX_TYPE")]
    tx_type: String,
    #[serde(rename = "FROM_USER_ID")]
    from_user_id: String,
    #[serde(rename = "TO_USER_ID")]
    to_user_id: String,
    #[serde(rename = "AMOUNT")]
    amount: String,
    #[serde(rename = "TIMESTAMP")]
    timestamp: String,
    #[serde(rename = "STATUS")]
    status: String,
    #[serde(rename = "DESCRIPTION")]
    description: Option<String>,
}

impl From<CsvDataValues> for DataValues {
    fn from(csv: CsvDataValues) -> Self {
        DataValues::new(
            csv.tx_id,
            csv.tx_type,
            csv.from_user_id,
            csv.to_user_id,
            csv.amount,
            csv.timestamp,
            csv.status,
            csv.description,
        )
    }
}

pub struct YPBankCsvReader<R: Read> {
    reader: Option<csv::Reader<R>>,
}
pub struct YPBankCsvWriter<W: Write> {
    writer: Option<csv::Writer<W>>,
}

impl<R: Read> YPBankCsvReader<R> {
    pub fn new(reader: R) -> Self {
        let csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(reader);

        Self {
            reader: Some(csv_reader),
        }
    }
}

impl<W: Write> YPBankCsvWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: Some(csv::Writer::from_writer(writer)),
        }
    }
}

impl<R: Read> BaseReader for YPBankCsvReader<R> {
    fn read(&mut self) -> Result<Option<DataValues>, ReadError> {
        let reader = self.reader.as_mut().ok_or(ReadError)?;

        let mut deserializer = reader.deserialize();

        loop {
            let result: Option<Result<CsvDataValues, _>> = deserializer.next();

            match result {
                Some(Ok(csv_data)) => {
                    if csv_data.tx_id.is_empty() {
                        continue;
                    }

                    let data: DataValues = csv_data.into();

                    return Ok(Some(data));
                }
                Some(Err(_)) => return Err(ReadError),
                None => return Ok(None),
            }
        }
    }
}

impl<W: Write> BaseWriter for YPBankCsvWriter<W> {
    fn write(&mut self, data_values: &DataValues) -> Result<(), WriteError> {
        let writer = self.writer.as_mut().ok_or(WriteError)?;

        writer
            .write_record(data_values.as_record())
            .map_err(|_| WriteError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_csv_cursor() {
        let data = b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n1000000000000000,DEPOSIT,0,9223372036854780000,100,1633036860000,FAILURE,Record number 1";
        let cursor = Cursor::new(data);
        let mut reader = YPBankCsvReader::new(cursor);
        let result = reader.read().unwrap();
        let data = result.unwrap();
        assert_eq!(data.as_record()[0], "1000000000000000");
    }

    #[test]
    fn test_read_no_desc_csv_cursor() {
        let data = b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n1000000000000000,DEPOSIT,0,9223372036854780000,100,1633036860000,FAILURE,";
        let cursor = Cursor::new(data);
        let mut reader = YPBankCsvReader::new(cursor);
        let result = reader.read().unwrap();
        let data = result.unwrap();
        assert_eq!(data.as_record()[7], "");
    }

    #[test]
    fn test_write_csv_cursor() {
        let buffer = Vec::new();
        let mut writer = YPBankCsvWriter::new(buffer);

        let data = DataValues::new(
            "1000000000000000".to_string(),
            "DEPOSIT".to_string(),
            "0".to_string(),
            "9223372036854780000".to_string(),
            "100".to_string(),
            "1633036860000".to_string(),
            "FAILURE".to_string(),
            Some("Record number 1".to_string()),
        );

        writer.write(&data).unwrap();
        let output = String::from_utf8(writer.writer.unwrap().into_inner().unwrap()).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(
            lines[0],
            "1000000000000000,DEPOSIT,0,9223372036854780000,100,1633036860000,FAILURE,Record number 1"
        );
    }

    #[test]
    fn test_write_no_desc_csv_cursor() {
        let buffer = Vec::new();
        let mut writer = YPBankCsvWriter::new(buffer);

        let data = DataValues::new(
            "1000000000000000".to_string(),
            "DEPOSIT".to_string(),
            "0".to_string(),
            "9223372036854780000".to_string(),
            "100".to_string(),
            "1633036860000".to_string(),
            "FAILURE".to_string(),
            None,
        );

        writer.write(&data).unwrap();
        let output = String::from_utf8(writer.writer.unwrap().into_inner().unwrap()).unwrap();
        let lines: Vec<&str> = output.trim().lines().collect();

        assert_eq!(
            lines[0],
            "1000000000000000,DEPOSIT,0,9223372036854780000,100,1633036860000,FAILURE,"
        );
    }

    #[test]
    fn test_spaces_between_lines_csc_cursor() {
        let data = b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n\n1000000000000000,DEPOSIT,0,9223372036854780000,100,1633036860000,FAILURE,Record number 1";
        let cursor = Cursor::new(data);
        let mut reader = YPBankCsvReader::new(cursor);
        let result = reader.read().unwrap();
        let data = result.unwrap();
        assert_eq!(data.as_record()[0], "1000000000000000");
    }

    #[test]
    fn test_not_enough_data_csv_cursor() {
        let data = b"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n1000000000000000,DEPOSIT,0,9223372036854780000,1633036860000,FAILURE,Record number 1";
        let cursor = Cursor::new(data);
        let mut reader = YPBankCsvReader::new(cursor);
        assert!(
            reader.read().is_err(),
            "unable to read information from read buffer"
        );
    }
}
