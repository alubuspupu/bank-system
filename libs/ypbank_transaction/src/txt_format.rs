use std::io::{BufRead, BufReader, Read, Write};

use crate::{
    BaseReader, BaseWriter, DataValues,
    errors::{ReadError, WriteError},
};

#[derive(Default, Clone)]
struct TxtDataValues {
    tx_id: Option<String>,
    tx_type: Option<String>,
    from_user_id: Option<String>,
    to_user_id: Option<String>,
    amount: Option<String>,
    timestamp: Option<String>,
    status: Option<String>,
    description: Option<String>,
}

impl TxtDataValues {
    fn check_empty_disc(s: &str) -> Option<String> {
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }

    pub fn new(data: &DataValues) -> Self {
        let record = data.as_record();

        Self {
            tx_id: Some(record[0].to_string()),
            tx_type: Some(record[1].to_string()),
            from_user_id: Some(record[2].to_string()),
            to_user_id: Some(record[3].to_string()),
            amount: Some(record[4].to_string()),
            timestamp: Some(record[5].to_string()),
            status: Some(record[6].to_string()),
            description: Self::check_empty_disc(record[7]),
        }
    }

    fn has_missing_fields(&self) -> bool {
        self.tx_id.is_none()
            || self.tx_type.is_none()
            || self.from_user_id.is_none()
            || self.to_user_id.is_none()
            || self.amount.is_none()
            || self.timestamp.is_none()
            || self.status.is_none()
    }

    fn convert_from_txt(&self) -> Result<DataValues, ReadError> {
        let tx_id = self.tx_id.clone().ok_or(ReadError)?;
        let tx_type = self.tx_type.clone().ok_or(ReadError)?;
        let from_user_id = self.from_user_id.clone().ok_or(ReadError)?;
        let to_user_id = self.to_user_id.clone().ok_or(ReadError)?;
        let amount = self.amount.clone().ok_or(ReadError)?;
        let timestamp = self.timestamp.clone().ok_or(ReadError)?;
        let status = self.status.clone().ok_or(ReadError)?;

        Ok(DataValues::new(
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            self.description.clone(),
        ))
    }

    fn into_result(self) -> Result<Option<DataValues>, ReadError> {
        if self.has_missing_fields() {
            Err(ReadError)
        } else {
            Ok(Some(self.convert_from_txt()?))
        }
    }

    fn analyze_field(&mut self, field: &str) -> Result<(), ReadError> {
        if let Some(idx) = field.find(": ") {
            let value = &field[idx + 2..].trim();

            if field.starts_with("DESCRIPTION") {
                if value.starts_with('"') && value.ends_with('"') {
                    let inner = &value[1..value.len() - 1];
                    self.description = Some(inner.to_string());
                } else {
                    return Err(ReadError);
                }
            } else {
                let parse_value = |f: &str| -> Option<String> {
                    f.split_whitespace().last().map(|s| s.to_owned())
                };

                if field.starts_with("TX_ID") {
                    self.tx_id = parse_value(field);
                } else if field.starts_with("TX_TYPE") {
                    self.tx_type = parse_value(field);
                } else if field.starts_with("FROM_USER_ID") {
                    self.from_user_id = parse_value(field);
                } else if field.starts_with("TO_USER_ID") {
                    self.to_user_id = parse_value(field);
                } else if field.starts_with("AMOUNT") {
                    self.amount = parse_value(field);
                } else if field.starts_with("TIMESTAMP") {
                    self.timestamp = parse_value(field);
                } else if field.starts_with("STATUS") {
                    self.status = parse_value(field);
                }
            }
        }
        Ok(())
    }

    pub fn to_txt_format(&self) -> String {
        let mut lines = vec![
            format!("TX_ID: {}", self.tx_id.as_deref().unwrap_or("")),
            format!("TX_TYPE: {}", self.tx_type.as_deref().unwrap_or("")),
            format!(
                "FROM_USER_ID: {}",
                self.from_user_id.as_deref().unwrap_or("")
            ),
            format!("TO_USER_ID: {}", self.to_user_id.as_deref().unwrap_or("")),
            format!("AMOUNT: {}", self.amount.as_deref().unwrap_or("")),
            format!("TIMESTAMP: {}", self.timestamp.as_deref().unwrap_or("")),
            format!("STATUS: {}", self.status.as_deref().unwrap_or("")),
        ];

        if let Some(ref desc) = self.description
            && !desc.is_empty()
        {
            lines.push(format!("DESCRIPTION: \"{}\"", desc));
        }

        lines.join("\n") + "\n"
    }
}

pub struct YPBankTxtReader<R: Read> {
    reader: Option<BufReader<R>>,
}
pub struct YPBankTxtWriter<W: Write> {
    writer: Option<W>,
}

impl<R: Read> YPBankTxtReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: Some(BufReader::new(reader)),
        }
    }
}

impl<W: Write> YPBankTxtWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: Some(writer),
        }
    }
}

impl<R: Read> BaseReader for YPBankTxtReader<R> {
    fn read(&mut self) -> Result<Option<DataValues>, ReadError> {
        let reader = self.reader.as_mut().ok_or(ReadError)?;

        let mut txt_data_values = TxtDataValues::default();

        loop {
            let mut line = String::new();
            let result = reader.read_line(&mut line);

            match result {
                Err(_) => return Err(ReadError),
                Ok(0) => return txt_data_values.into_result(),
                _ => {
                    if line == "\n" {
                        return txt_data_values.into_result();
                    }

                    if line.starts_with("#") {
                        continue;
                    }

                    txt_data_values.analyze_field(&line)?;
                }
            }
        }
    }
}

impl<W: Write> BaseWriter for YPBankTxtWriter<W> {
    fn write(&mut self, data_values: &DataValues) -> Result<(), WriteError> {
        let writer = self.writer.as_mut().ok_or(WriteError)?;

        let txt_data_values = TxtDataValues::new(data_values);

        let txt = txt_data_values.to_txt_format();
        writeln!(writer, "{}", txt).map_err(|_| WriteError)?;
        writer.flush().map_err(|_| WriteError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_txt_cursor() {
        let data = b"\
# Record 1 (Deposit)
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
TIMESTAMP: 1633036800000
STATUS: SUCCESS
DESCRIPTION: \"Terminal deposit\"
";
        let cursor = Cursor::new(data);
        let mut reader = YPBankTxtReader::new(cursor);
        let result = reader.read().unwrap();
        let data = result.unwrap();
        assert_eq!(data.as_record()[0], "1234567890123456");
        assert_eq!(data.as_record()[1], "DEPOSIT");
        assert_eq!(data.as_record()[2], "0");
        assert_eq!(data.as_record()[3], "9876543210987654");
        assert_eq!(data.as_record()[4], "10000");
        assert_eq!(data.as_record()[5], "1633036800000");
        assert_eq!(data.as_record()[6], "SUCCESS");
        assert_eq!(data.as_record()[7], "Terminal deposit");
    }

    #[test]
    fn test_read_txt_inordered_lines_cursor() {
        let data = b"\
# Record 1 (Deposit)
TIMESTAMP: 1633036800000
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9876543210987654
AMOUNT: 10000
STATUS: SUCCESS
DESCRIPTION: \"Terminal deposit\"
";
        let cursor = Cursor::new(data);
        let mut reader = YPBankTxtReader::new(cursor);
        let result = reader.read().unwrap();
        let data = result.unwrap();
        assert_eq!(data.as_record()[0], "1234567890123456");
        assert_eq!(data.as_record()[1], "DEPOSIT");
        assert_eq!(data.as_record()[2], "0");
        assert_eq!(data.as_record()[3], "9876543210987654");
        assert_eq!(data.as_record()[4], "10000");
        assert_eq!(data.as_record()[5], "1633036800000");
        assert_eq!(data.as_record()[6], "SUCCESS");
        assert_eq!(data.as_record()[7], "Terminal deposit");
    }

    #[test]
    fn test_read_txt_comments_cursor() {
        let data = b"\
# Record 1 (Deposit)
# Record 1 (Deposit)
TIMESTAMP: 1633036800000
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
# Record 1 (Deposit)
TO_USER_ID: 9876543210987654
AMOUNT: 10000
STATUS: SUCCESS
DESCRIPTION: \"Terminal deposit\"
# Record 1 (Deposit)
";
        let cursor = Cursor::new(data);
        let mut reader = YPBankTxtReader::new(cursor);
        let result = reader.read().unwrap();
        let data = result.unwrap();
        assert_eq!(data.as_record()[0], "1234567890123456");
        assert_eq!(data.as_record()[1], "DEPOSIT");
        assert_eq!(data.as_record()[2], "0");
        assert_eq!(data.as_record()[3], "9876543210987654");
        assert_eq!(data.as_record()[4], "10000");
        assert_eq!(data.as_record()[5], "1633036800000");
        assert_eq!(data.as_record()[6], "SUCCESS");
        assert_eq!(data.as_record()[7], "Terminal deposit");
    }

    #[test]
    fn test_read_txt_skiped_string_cursor() {
        let data = b"\
\
TIMESTAMP: 1633036800000
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
# Record 1 (Deposit)
TO_USER_ID: 9876543210987654
AMOUNT: 10000
STATUS: SUCCESS
DESCRIPTION: \"Terminal deposit\"
# Record 1 (Deposit)
";
        let cursor = Cursor::new(data);
        let mut reader = YPBankTxtReader::new(cursor);
        let result = reader.read().unwrap();
        let data = result.unwrap();
        assert_eq!(data.as_record()[0], "1234567890123456");
        assert_eq!(data.as_record()[1], "DEPOSIT");
        assert_eq!(data.as_record()[2], "0");
        assert_eq!(data.as_record()[3], "9876543210987654");
        assert_eq!(data.as_record()[4], "10000");
        assert_eq!(data.as_record()[5], "1633036800000");
        assert_eq!(data.as_record()[6], "SUCCESS");
        assert_eq!(data.as_record()[7], "Terminal deposit");
    }

    #[test]
    fn test_read_txt_few_records_cursor() {
        let data = b"\
\
TIMESTAMP: 1633036800000
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
# Record 1 (Deposit)
TO_USER_ID: 9876543210987654
AMOUNT: 10000
STATUS: SUCCESS
DESCRIPTION: \"Terminal deposit\"
# Record 1 (Deposit)

TIMESTAMP: 1633036800001
TX_ID: 1234567890123457
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
# Record 1 (Deposit)
TO_USER_ID: 9876543210987654
AMOUNT: 10000
STATUS: SUCCESS
DESCRIPTION: \"Terminal deposit\"
# Record 1 (Deposit)
";
        let cursor = Cursor::new(data);
        let mut reader = YPBankTxtReader::new(cursor);
        reader.read().unwrap();
        let result2 = reader.read().unwrap();
        let data = result2.unwrap();
        assert_eq!(data.as_record()[0], "1234567890123457");
    }

    #[test]
    fn test_write_txt_format() {
        let buffer = Vec::new(); // Буфер для записи
        let mut writer = YPBankTxtWriter::new(buffer);

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

        let output = String::from_utf8(writer.writer.unwrap()).expect("Not UTF-8");
        let expected = "\
TX_ID: 1000000000000000
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9223372036854780000
AMOUNT: 100
TIMESTAMP: 1633036860000
STATUS: FAILURE
DESCRIPTION: \"Record number 1\"\
";
        assert_eq!(output.trim(), expected);
    }

    #[test]
    fn test_write_no_desc_txt_format() {
        let buffer = Vec::new(); // Буфер для записи
        let mut writer = YPBankTxtWriter::new(buffer);

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

        let output = String::from_utf8(writer.writer.unwrap()).expect("Not UTF-8");
        let expected = "\
TX_ID: 1000000000000000
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 9223372036854780000
AMOUNT: 100
TIMESTAMP: 1633036860000
STATUS: FAILURE";
        assert_eq!(output.trim(), expected);
    }

    #[test]
    fn test_read_txt_no_desc_cursor() {
        let data = b"\
# Record 1 (Deposit)
# Record 1 (Deposit)
TIMESTAMP: 1633036800000
TX_ID: 1234567890123456
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
# Record 1 (Deposit)
TO_USER_ID: 9876543210987654
AMOUNT: 10000
STATUS: SUCCESS
# Record 1 (Deposit)
";
        let cursor = Cursor::new(data);
        let mut reader = YPBankTxtReader::new(cursor);
        let result = reader.read().unwrap();
        let data = result.unwrap();
        assert_eq!(data.as_record()[0], "1234567890123456");
        assert_eq!(data.as_record()[1], "DEPOSIT");
        assert_eq!(data.as_record()[2], "0");
        assert_eq!(data.as_record()[3], "9876543210987654");
        assert_eq!(data.as_record()[4], "10000");
        assert_eq!(data.as_record()[5], "1633036800000");
        assert_eq!(data.as_record()[6], "SUCCESS");
    }
}
