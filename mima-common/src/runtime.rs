use crate::types::{MimaValue, MimaAddress, coerce_mima_value, is_negative, coerce_mima_address, MAX_VALUE};
use crate::instructions::{Instruction, Opcode};
use std::convert::TryFrom;
use std::iter::repeat;
use std::slice::Iter;

pub struct Runtime {
    accu: MimaValue,
    iar: MimaAddress,
    ir: MimaValue,
    memory: Vec<MimaValue>,
    pub halt: bool
}

impl Runtime {
    pub fn with_memory(initial_memory: Vec<MimaValue>) -> Self {
        Self {
            accu: 0,
            iar: 0,
            ir: 0,
            memory: initial_memory,
            halt: false
        }
    }

    pub fn with_instructions(instructions: &Vec<MimaValue>) -> Self {
        let mut memory = Vec::with_capacity(instructions.len());
        memory.extend(instructions);
        Self::with_memory(memory)
    }

    pub fn new() -> Self {
        Self::with_memory(Vec::new())
    }

    pub fn read_accu(&self) -> MimaValue {
        self.accu
    }

    pub fn write_accu(&mut self, val: MimaValue) {
        self.accu = coerce_mima_value(val);
    }

    pub fn read_ir(&self) -> MimaAddress {
        self.ir
    }

    pub fn write_ir(&mut self, instr: MimaValue) {
        self.ir = coerce_mima_value(instr);
    }

    pub fn read_iar(&self) -> MimaAddress {
        self.iar
    }

    pub fn write_iar(&mut self, addr: MimaAddress) {
        self.iar = coerce_mima_address(addr);
    }

    pub fn read_mem(&self, addr: MimaAddress) -> MimaValue {
        let coerced = coerce_mima_address(addr) as usize;
        if coerced < self.memory.len() {
            self.memory[coerced]
        } else {
            0
        }
    }

    pub fn write_mem(&mut self, addr: MimaAddress, val: MimaValue) {
        let coerced = coerce_mima_address(addr) as usize;
        let mem = &mut self.memory;
        if coerced >= mem.len() {
            mem.extend(repeat(0).take(coerced + 1 - mem.len()));
        }
        mem[coerced] = coerce_mima_value(val);
    }

    pub fn mem_iter(&self) -> Iter<'_, MimaValue> {
        self.memory.iter()
    }

    pub fn run(&mut self) -> Result<(), String> {
        while !self.halt {
            self.step()?;
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), String> {
        if self.halt {
            return Err("MIMA is halted".to_owned())
        }
        let instruction = self.next_instruction();
        if instruction.is_err() {
            return Err(format!("Decode failure - {}", instruction.err().unwrap()));
        }
        self.write_iar(self.next_instruction_addr());

        let instr = instruction.unwrap();
        let opcode = instr.opcode;
        let arg = instr.arg;
        match opcode {
            Opcode::LDC => self.ldc(arg),
            Opcode::LDV => self.ldv(arg),
            Opcode::STV => self.stv(arg),
            Opcode::ADD => self.add(arg),
            Opcode::AND => self.and(arg),
            Opcode::OR => self.or(arg),
            Opcode::XOR => self.xor(arg),
            Opcode::EQL => self.eql(arg),
            Opcode::JMP => self.jmp(arg),
            Opcode::JMN => self.jmn(arg),
            Opcode::LDIV => self.ldiv(arg),
            Opcode::STIV => self.stiv(arg),
            Opcode::HALT => self.halt(),
            Opcode::NOT => self.not(),
            Opcode::RAR => self.rar()
        }
        Ok(())

    }

    fn ldc(&mut self, arg: MimaAddress) {
        self.write_accu(coerce_mima_value(arg));
    }

    fn ldv(&mut self, arg: MimaAddress) {
        self.write_accu(self.read_mem(arg));
    }

    fn stv(&mut self, arg: MimaAddress) {
        self.write_mem(arg, self.accu);
    }

    fn add(&mut self, arg: MimaAddress) {
        let result = self.read_accu() + self.read_mem(arg);
        self.write_accu(result);
    }

    fn and(&mut self, arg: MimaAddress) {
        let result = self.read_accu() & self.read_mem(arg);
        self.write_accu(result);
    }

    fn or(&mut self, arg: MimaAddress) {
        let result = self.read_accu() | self.read_mem(arg);
        self.write_accu(result);
    }

    fn xor(&mut self, arg: MimaAddress) {
        let result = self.read_accu() ^ self.read_mem(arg);
        self.write_accu(result);
    }

    fn eql(&mut self, arg: MimaAddress) {
        let result = self.read_accu() == self.read_mem(arg);
        self.write_accu(if result { MAX_VALUE } else { 0 });
    }

    fn jmp(&mut self, arg: MimaAddress) {
        self.write_iar(arg)
    }

    fn jmn(&mut self, arg: MimaAddress) {
        if is_negative(self.read_accu()) {
            self.jmp(arg);
        }
    }

    fn ldiv(&mut self, arg: MimaAddress) {
        self.ldv(self.read_mem(arg));
    }

    fn stiv(&mut self, arg: MimaAddress) {
        self.stv(self.read_mem(arg));
    }

    fn halt(&mut self) {
        self.halt = true;
    }

    fn not(&mut self) {
        self.write_accu(self.read_accu().reverse_bits());
    }

    fn rar(&mut self) {
        self.write_accu(self.read_accu().rotate_right(1));
    }

    pub fn next_instruction(&self) -> Result<Instruction, String> {
        Instruction::try_from(self.read_mem(self.read_iar()))
    }

    pub fn next_instruction_addr(&self) -> MimaAddress {
        coerce_mima_address(self.read_iar() + 1)
    }
}