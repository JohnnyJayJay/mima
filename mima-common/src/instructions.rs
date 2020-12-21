use std::convert::TryFrom;
use crate::types::{MimaAddress, MimaValue, ADDRESS_BITS, coerce_mima_value, MAX_ADDRESS};
use strum_macros::{ToString, EnumString};
use enum_repr::EnumRepr;

#[derive(Debug, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub arg: MimaAddress
}

#[EnumRepr(type = "u8")]
#[derive(Debug, PartialEq, Eq, EnumString, ToString)]
pub enum Opcode {
    LDC = 0x00,
    LDV = 0x01,
    STV = 0x02,
    ADD = 0x03,
    AND = 0x04,
    OR = 0x05,
    XOR = 0x06,
    EQL = 0x07,
    JMP = 0x08,
    JMN = 0x09,
    LDIV = 0x0a,
    STIV = 0x0b,
    HALT = 0xf0,
    NOT = 0xf1,
    RAR = 0xf2
}

impl Opcode {

    pub fn has_arg(&self) -> bool {
        ((self.repr() >> 4) & 0xf) != 0xf
    }

}

impl Instruction {
    pub fn opcode_bits(instr: MimaValue) -> u8 {
        let prefix = instr >> ADDRESS_BITS;
        (if prefix == 0x0f {
            instr >> (ADDRESS_BITS - 4)
        } else {
            prefix
        }) as u8
    }

    pub fn arg_bits(instr: MimaValue) -> MimaAddress {
        instr & MAX_ADDRESS
    }
}

/*impl TryFrom<u8> for Opcode {
    type Error = String;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            0x00 => Ok(Self::LDC),
            0x01 => Ok(Self::LDV),
            0x02 => Ok(Self::STV),
            0x03 => Ok(Self::ADD),
            0x04 => Ok(Self::AND),
            0x05 => Ok(Self::OR),
            0x06 => Ok(Self::XOR),
            0x07 => Ok(Self::EQL),
            0x08 => Ok(Self::JMP),
            0x09 => Ok(Self::JMN),
            0x0a => Ok(Self::LDIV),
            0x0b => Ok(Self::STIV),
            0xf0 => Ok(Self::HALT),
            0xf1 => Ok(Self::NOT),
            0xf2 => Ok(Self::RAR),
            _ => Err()
        }
    }
}*/

impl ToString for Instruction {
    fn to_string(&self) -> String {
        if self.opcode.has_arg() {
            format!("{:4} {:#x}", self.opcode.to_string(), self.arg)
        } else {
            self.opcode.to_string()
        }
    }
}

impl TryFrom<MimaValue> for Instruction {
    type Error = String;

    fn try_from(instr: MimaValue) -> Result<Self, Self::Error> {
        let opcode_bits = Self::opcode_bits(instr);
        Opcode::from_repr(opcode_bits)
            .ok_or(format!("Instruction {:#x} uses unrecognized opcode: {:#x}", instr, opcode_bits))
            .map(|opcode| Self { opcode, arg: Self::arg_bits(instr)} )
    }
}

impl From<&Instruction> for MimaValue {
    fn from(instr: &Instruction) -> Self {
        coerce_mima_value(if instr.opcode.has_arg() {
            ((instr.opcode.repr() as u32) << ADDRESS_BITS) | instr.arg
        } else {
            (instr.opcode.repr() as u32) << (ADDRESS_BITS - 4)
        })
    }
}

