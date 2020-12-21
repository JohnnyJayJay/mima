use byteorder::{ReadBytesExt, BigEndian, WriteBytesExt};
use std::io;
use std::str::FromStr;
use std::num::ParseIntError;
use std::io::Cursor;

pub const VALUE_BYTES: u8 = 3;
pub const ADDRESS_BITS: u8 = 20;
pub const VALUE_BITS: u8 = 24;
pub const ADDRESS_SPACE: u32 = 1 << ADDRESS_BITS;
pub const VALUE_SPACE: u32 = 1 << VALUE_BITS;
pub const MAX_ADDRESS: MimaAddress = ADDRESS_SPACE - 1;
pub const MAX_VALUE: MimaValue = VALUE_SPACE - 1;

pub type MimaValue = u32;
pub type MimaAddress = u32;

pub trait ReadMimaExt: ReadBytesExt {
    fn read_mima_val(&mut self) -> io::Result<MimaValue> {
        self.read_u24::<BigEndian>()
    }

    fn read_all_mima_vals(&mut self) -> io::Result<Vec<MimaValue>> {
        let mut bytes = Vec::new();
        let byte_count = self.read_to_end(&mut bytes)?;
        if byte_count % (VALUE_BYTES as usize) != 0 {
            return io::Result::Err(
                io::Error::new(io::ErrorKind::InvalidData,
                               "Cannot read mima values: input is malformed"));
        }
        let value_count = byte_count / (VALUE_BYTES as usize);
        let mut cursor = Cursor::new(bytes);
        let mut values = Vec::with_capacity(byte_count / 3);
        for _i in 0..value_count {
            values.push(cursor.read_mima_val().unwrap())
        }
        Ok(values)
    }
}

pub trait WriteMimaExt: WriteBytesExt {
    fn write_mima_val(&mut self, val: MimaValue) -> io::Result<()> {
        self.write_u24::<BigEndian>(val)
    }

    fn write_all_mima_vals(&mut self, vals: &[MimaValue]) -> io::Result<()> {
        for val in vals {
            self.write_mima_val(*val)?;
        }
        Ok(())
    }
}

impl<R: ReadBytesExt> ReadMimaExt for R {}

impl<W: WriteBytesExt> WriteMimaExt for W {}

pub fn parse_mima_addr(s: &str) -> Result<MimaAddress, ParseIntError> {
    (if let Some(hex) = s.strip_prefix("0x") {
        MimaAddress::from_str_radix(hex, 16)
    } else {
        MimaAddress::from_str(s)
    }).map(|v| coerce_mima_address(v))
}

pub fn parse_mima_value(s: &str) -> Result<MimaValue, ParseIntError> {
    (if let Some(hex) = s.strip_prefix("0x") {
        MimaValue::from_str_radix(hex, 16)
    } else {
        MimaValue::from_str(s)
    }).map(|v| coerce_mima_value(v))
}


pub fn coerce_mima_value(num: MimaValue) -> MimaValue {
    num & MAX_VALUE
}

pub fn coerce_mima_address(addr: MimaAddress) -> MimaAddress {
    addr & MAX_ADDRESS
}

pub fn signum(num: MimaValue) -> u8 {
    ((num >> (VALUE_BITS - 1)) & 1) as u8
}

pub fn is_negative(num: MimaValue) -> bool {
    signum(num) == 1
}