use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{BufReader, Read, Write};

use crate::{
    BaseReader, BaseWriter, DataValues,
    errors::{ConvertFromStrToU8Error, ReadError, WriteError},
};

#[derive(Default, Clone)]
#[repr(u8)]
enum BinDataTypes {
    #[default]
    Deposit = 0,
    Transfer = 1,
    Withdrawal = 2,
}

impl BinDataTypes {
    pub fn from_u8(value: u8) -> Result<Self, &'static str> {
        match value {
            0 => Ok(BinDataTypes::Deposit),
            1 => Ok(BinDataTypes::Transfer),
            2 => Ok(BinDataTypes::Withdrawal),
            _ => Err("Invalid value for BinDataTypes"),
        }
    }

    pub fn to_string(&self) -> Result<String, ConvertFromStrToU8Error> {
        match self {
            BinDataTypes::Deposit => Ok("DEPOSIT".to_string()),
            BinDataTypes::Transfer => Ok("TRANSFER".to_string()),
            BinDataTypes::Withdrawal => Ok("WITHDRAWAL".to_string()),
        }
    }
}

#[derive(Clone, Default)]
#[repr(u8)]
enum BinDataStatus {
    #[default]
    Success = 0,
    Failure = 1,
    Pending = 2,
}

impl BinDataStatus {
    pub fn from_u8(value: u8) -> Result<Self, &'static str> {
        match value {
            0 => Ok(BinDataStatus::Success),
            1 => Ok(BinDataStatus::Failure),
            2 => Ok(BinDataStatus::Pending),
            _ => Err("Invalid value for BinDataTypes"),
        }
    }

    pub fn to_string(&self) -> Result<String, ConvertFromStrToU8Error> {
        match self {
            BinDataStatus::Success => Ok("SUCCESS".to_string()),
            BinDataStatus::Failure => Ok("FAILURE".to_string()),
            BinDataStatus::Pending => Ok("PENDING".to_string()),
        }
    }
}

#[derive(Default, Clone)]
struct BinDataValues {
    tx_id: u64,
    tx_type: BinDataTypes,
    from_user_id: u64,
    to_user_id: u64,
    amount: i64,
    timestamp: u64,
    status: BinDataStatus,
    description: Vec<u8>,
}

impl BinDataValues {
    pub fn from_data_values(data: &DataValues) -> Result<Self, ConvertFromStrToU8Error> {
        let record = data.as_record();

        let tx_id = record[0]
            .parse::<u64>()
            .map_err(|_| ConvertFromStrToU8Error {
                value: record[0].to_owned(),
                which_struct: "BinDataValues".to_string(),
            })?;
        let from_user_id = record[2]
            .parse::<u64>()
            .map_err(|_| ConvertFromStrToU8Error {
                value: record[0].to_owned(),
                which_struct: "BinDataValues".to_string(),
            })?;
        let to_user_id = record[3]
            .parse::<u64>()
            .map_err(|_| ConvertFromStrToU8Error {
                value: record[0].to_owned(),
                which_struct: "BinDataValues".to_string(),
            })?;
        let amount = record[4]
            .parse::<i64>()
            .map_err(|_| ConvertFromStrToU8Error {
                value: record[0].to_owned(),
                which_struct: "BinDataValues".to_string(),
            })?;
        let timestamp = record[5]
            .parse::<u64>()
            .map_err(|_| ConvertFromStrToU8Error {
                value: record[0].to_owned(),
                which_struct: "BinDataValues".to_string(),
            })?;

        let tx_type = match record[1] {
            "DEPOSIT" => BinDataTypes::Deposit,
            "TRANSFER" => BinDataTypes::Transfer,
            "WITHDRAWAL" => BinDataTypes::Withdrawal,
            _ => {
                return Err(ConvertFromStrToU8Error {
                    value: record[1].to_owned(),
                    which_struct: "BinDataValues".to_string(),
                });
            }
        };

        let status = match record[6] {
            "SUCCESS" => BinDataStatus::Success,
            "FAILURE" => BinDataStatus::Failure,
            "PENDING" => BinDataStatus::Pending,
            _ => {
                return Err(ConvertFromStrToU8Error {
                    value: record[6].to_owned(),
                    which_struct: "BinDataValues".to_string(),
                });
            }
        };

        let description = record[7].as_bytes().to_vec();

        Ok(Self {
            tx_id,
            tx_type,
            from_user_id,
            to_user_id,
            amount,
            timestamp,
            status,
            description,
        })
    }
}

pub struct YPBankBinReader<R: Read> {
    reader: Option<BufReader<R>>,
}
pub struct YPBankBinWriter<W: Write> {
    writer: Option<W>,
}

impl<R: Read> YPBankBinReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: Some(BufReader::new(reader)),
        }
    }
}

impl<W: Write> YPBankBinWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: Some(writer),
        }
    }
}

impl<R: Read> BaseReader for YPBankBinReader<R> {
    fn read(&mut self) -> Result<Option<DataValues>, ReadError> {
        let reader = self.reader.as_mut().ok_or(ReadError {
            reason: "reader".to_owned(),
        })?;

        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })
            .and_then(|_| {
                if &magic != b"YPBN" {
                    return Err(ReadError {
                        reason: "reader".to_owned(),
                    });
                };

                Ok(())
            })?;

        let size = reader
            .read_u32::<BigEndian>()
            .map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })
            .and_then(|val| {
                if val < 46 {
                    return Err(ReadError {
                        reason: "reader".to_owned(),
                    });
                };
                Ok(val)
            })?;

        let tx_id = reader.read_u64::<BigEndian>().map_err(|_| ReadError {
            reason: "reader".to_owned(),
        })?;

        let type_val = {
            let val = reader.read_u8().map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })?;

            if val > 2 {
                return Err(ReadError {
                    reason: "reader".to_owned(),
                });
            }

            let bin_type = BinDataTypes::from_u8(val).map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })?;
            bin_type.to_string().map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })?
        };

        let from_user_id = reader.read_u64::<BigEndian>().map_err(|_| ReadError {
            reason: "reader".to_owned(),
        })?;
        let to_user_id = reader.read_u64::<BigEndian>().map_err(|_| ReadError {
            reason: "reader".to_owned(),
        })?;
        let amount = reader.read_i64::<BigEndian>().map_err(|_| ReadError {
            reason: "reader".to_owned(),
        })?;
        let timestamp = reader.read_u64::<BigEndian>().map_err(|_| ReadError {
            reason: "reader".to_owned(),
        })?;

        let status = {
            let val = reader.read_u8().map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })?;

            if val > 2 {
                return Err(ReadError {
                    reason: "reader".to_owned(),
                });
            }

            let bin_type = BinDataStatus::from_u8(val).map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })?;
            bin_type.to_string().map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })?
        };

        let desc_size = reader.read_u32::<BigEndian>().map_err(|_| ReadError {
            reason: "reader".to_owned(),
        })?;

        if desc_size + 46 != size {
            return Err(ReadError {
                reason: "reader".to_owned(),
            });
        }

        let mut s: Option<String> = None;

        if desc_size != 0 {
            let mut description = vec![0u8; desc_size as usize];
            reader.read(&mut description).map_err(|_| ReadError {
                reason: "reader".to_owned(),
            })?;
            s = Some(String::from_utf8_lossy(&description).into_owned());
        }

        Ok(Some(DataValues::new(
            tx_id.to_string(),
            type_val,
            from_user_id.to_string(),
            to_user_id.to_string(),
            amount.to_string(),
            timestamp.to_string(),
            status,
            s,
        )))
    }
}

impl<W: Write> BaseWriter for YPBankBinWriter<W> {
    fn write(&mut self, data_values: &DataValues) -> Result<(), WriteError> {
        let writer = self.writer.as_mut().ok_or(WriteError {
            reason: "writer".to_owned(),
        })?;

        let bin_data = BinDataValues::from_data_values(data_values).map_err(|_| WriteError {
            reason: "writer".to_owned(),
        })?;

        writer.write_all(b"YPBN").map_err(|_| WriteError {
            reason: "writer".to_owned(),
        })?;
        writer
            .write_u32::<BigEndian>(46 + bin_data.description.len() as u32)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;
        writer
            .write_u64::<BigEndian>(bin_data.tx_id)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;
        writer
            .write_u8(bin_data.tx_type as u8)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;
        writer
            .write_u64::<BigEndian>(bin_data.from_user_id)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;
        writer
            .write_u64::<BigEndian>(bin_data.to_user_id)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;
        writer
            .write_i64::<BigEndian>(bin_data.amount)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;
        writer
            .write_u64::<BigEndian>(bin_data.timestamp)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;
        writer
            .write_u8(bin_data.status as u8)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;

        let desc_len = bin_data.description.len() as u32;
        writer
            .write_u32::<BigEndian>(desc_len)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;
        writer
            .write_all(&bin_data.description)
            .map_err(|_| WriteError {
                reason: "writer".to_owned(),
            })?;

        writer.flush().map_err(|_| WriteError {
            reason: "writer".to_owned(),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_write_and_read_bin_format() {
        let original_data = DataValues::new(
            "1234567890123456".to_string(),
            "DEPOSIT".to_string(),
            "0".to_string(),
            "9876543210987654".to_string(),
            "10000".to_string(),
            "1633036800000".to_string(),
            "SUCCESS".to_string(),
            Some("Test deposit transaction".to_string()),
        );

        let mut buffer = Vec::new();
        {
            let mut writer = YPBankBinWriter::new(&mut buffer);
            writer.write(&original_data).unwrap();
        }

        let mut reader = YPBankBinReader::new(Cursor::new(&buffer));
        let result = reader.read().unwrap().unwrap();
        assert_eq!(result.as_record(), original_data.as_record());
    }

    #[test]
    fn test_write_and_read_no_desc_bin_format() {
        let original_data = DataValues::new(
            "1234567890123456".to_string(),
            "DEPOSIT".to_string(),
            "0".to_string(),
            "9876543210987654".to_string(),
            "10000".to_string(),
            "1633036800000".to_string(),
            "SUCCESS".to_string(),
            None,
        );

        let mut buffer = Vec::new();
        {
            let mut writer = YPBankBinWriter::new(&mut buffer);
            writer.write(&original_data).unwrap();
        }

        let mut reader = YPBankBinReader::new(Cursor::new(&buffer));
        let result = reader.read().unwrap().unwrap();
        assert_eq!(result.as_record(), original_data.as_record());
    }
}
