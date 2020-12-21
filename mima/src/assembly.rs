use mima_common::instructions::{Instruction, Opcode};
use mima_common::types::{MimaAddress, coerce_mima_address, parse_mima_addr};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

// interim representation of address arguments
#[derive(Debug)]
enum InterimAddr {
    Template(String),
    Real(MimaAddress),
}

pub fn assemble(input: String, absolute_addresses: bool) -> Result<Vec<Instruction>, String> {
    let mut addr_labels = HashMap::<String, MimaAddress>::new();
    let mut addr_templates = HashSet::<String>::new();
    let mut instr_templates = Vec::<(Opcode, Option<InterimAddr>)>::new();
    let mut cur_instr_addr = 0;
    let mut highest_addr_in_use = 0;
    let mut line_num = 1;

    for line in input.lines() {
        // get all tokens until comments, if any
        let mut tokens: Vec<&str> = line.split_whitespace()
            .take_while(|s| !s.starts_with(';'))
            .collect();

        // ignore blank lines
        if tokens.is_empty() {
            continue;
        }

        let token = tokens.remove(0);
        // if token is label declaration, associate it with the current instruction address
        if let Some(label) = token.strip_suffix(':') {
            if !tokens.is_empty() {
                return Err(format!("Line {}: Unexpected token(s) after label '{}'",
                                   line_num, label));
            }
            addr_labels.insert(label.to_owned(), cur_instr_addr);
        } else {
            // else parse instruction
            let opcode_token = token.to_uppercase();
            let opcode = Opcode::from_str(&opcode_token)
                .map_err(|_e| format!("Line {}: Unknown mnemonic opcode '{}'",
                                     line_num, &opcode_token))?;

            let mut possible_arg = None;
            if opcode.has_arg() {
                if tokens.is_empty() {
                    return Err(format!("Line {}: Expected argument after {}",
                                       line_num, &opcode_token));
                }
                let arg = tokens.remove(0);
                if !tokens.is_empty() {
                    return Err(format!("Line {}: Unexpected token(s) after instruction", line_num));
                }
                possible_arg = Some(if let Ok(val) = parse_mima_addr(arg) {
                    // keep track of the highest explicit address in use for address templating
                    if opcode != Opcode::LDC && val > highest_addr_in_use {
                        highest_addr_in_use = val;
                    }
                    InterimAddr::Real(val)
                } else {
                    // if address is not a number, add it as a template
                    addr_templates.insert(arg.to_owned());
                    InterimAddr::Template(arg.to_owned())
                });
            }
            instr_templates.push((opcode, possible_arg));
            cur_instr_addr += 1;
        }
        line_num += 1;

    }

    let instr_count = instr_templates.len();
    assign_template_addresses(
        instr_count, absolute_addresses,
        highest_addr_in_use,
        addr_templates, &mut addr_labels
    );
    let instructions = construct_instructions(
        instr_templates, addr_labels, absolute_addresses
    );
    Ok(instructions)
}

// assigns all uninitialised template addresses an address in the address space,
// avoiding conflicts with other addresses as much as possible.
fn assign_template_addresses(
    instr_count: usize,
    absolute_addresses: bool,
    max_address: MimaAddress,
    templates: HashSet<String>,
    labels: &mut HashMap<String, MimaAddress>
) {
    let mut next_addr = max_address + 1;
    for template in templates {
        if !labels.contains_key(&template) {
            labels.insert(template,
                          add_offset(next_addr, instr_count, absolute_addresses));
            next_addr += 1;
        }

    }
}

fn construct_instructions(
    templates: Vec<(Opcode, Option<InterimAddr>)>,
    labels: HashMap<String, MimaAddress>,
    absolute_addresses: bool
) -> Vec<Instruction> {
    let count = templates.len();
    let mut instructions = Vec::with_capacity(count);
    for (op, arg) in templates {
        let addr = match arg {
            Some(InterimAddr::Template(s)) => labels[&s],
            // add offset only if it's an unadjusted explicit address used in a non-ldc opcode
            Some(InterimAddr::Real(addr)) =>
                add_offset(addr, count,
                           absolute_addresses || !op.has_arg() || op == Opcode::LDC),
            None => 0
        };

        instructions.push(Instruction {
            opcode: op,
            arg: addr
        });
    }
    instructions
}

// adds an appropriate offset to the given address or does nothing when unchanged is true
fn add_offset(addr: MimaAddress, instr_count: usize, unchanged: bool) -> MimaAddress {
    coerce_mima_address(addr + (if unchanged { 0 } else { instr_count as u32 + 1 }))
}
