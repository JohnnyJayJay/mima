use mima_common::types::MimaValue;
use mima_common::instructions::Instruction;
use std::convert::TryFrom;

pub fn disassemble(instructions: Vec<MimaValue>) -> Result<String, String> {
    let mut output = String::new();
    for instr_value in instructions {
        let instr = Instruction::try_from(instr_value)?;
        output.push_str(&*instr.to_string());
        output.push('\n');
    }
    Ok(output)
}