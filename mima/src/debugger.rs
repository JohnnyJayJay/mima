use mima_common::runtime::Runtime;
use rustyline::Editor;
use std::collections::HashSet;
use mima_common::types::{MimaAddress, parse_mima_addr, parse_mima_value, ADDRESS_SPACE};
use mima_common::instructions::Instruction;
use rustyline::config::Configurer;
use std::path::PathBuf;
use std::str::FromStr;
use crate::{create_memdump};
use std::convert::TryFrom;

const HELP_MESSAGE: &'static str =
    "`state` - print the current state of the machine
`step` - run the next instruction and immediately brak again
`continue` - continue execution until the next breakpoint
`break <addr>` - toggle breakpoint at the specified address
`read <addr>` - print value at the given address
`write <addr> <val>` - write value to the given address
`dump <file>` - dump the machine's memory to the specified file
`halt` - stop execution
`?` - display this help message";

pub struct Debugger<'a> {
    runtime: &'a mut Runtime,
    editor: Editor<()>,
    breakpoints: HashSet<MimaAddress>,
    break_next: bool,
    break_state: bool
}

impl<'a> From<&'a mut Runtime> for Debugger<'a> {
    fn from(runtime: &'a mut Runtime) -> Self {
        let mut editor = Editor::<()>::new();
        editor.set_auto_add_history(true);
        Debugger {
            runtime,
            editor,
            breakpoints: HashSet::new(),
            break_next: true,
            break_state: false
        }
    }
}

impl Debugger<'_> {

    pub fn run(&mut self) -> Result<(), String> {
        while !self.runtime.halt {
            let instr_addr = self.runtime.read_iar();
            if self.break_next || self.breakpoints.contains(&instr_addr) {
                self.break_next = false;
                self.print_state();
                self.break_state = true;
                while self.break_state {
                    let input = self.editor.readline(">")
                        .map_err(|kind| kind.to_string())?;
                    let args: Vec<&str> = input.split(' ').collect();
                    match args.as_slice() {
                        ["state"] => self.print_state(),
                        ["step"] => self.step(),
                        ["continue"] => self.continue_run(),
                        ["break", addr] => self.toggle_breakpoint(addr),
                        ["read", addr] => self.print_mem(addr),
                        ["write", addr, val] => self.write_mem(addr, val),
                        ["dump", path] => self.make_dump(path),
                        ["halt"] => self.stop(),
                        ["?"] => println!("{}", HELP_MESSAGE),
                        _ => {
                            eprintln!("Unknown command. Type '?' for help");
                        }
                    }
                }
            }
            if !self.runtime.halt {
                self.runtime.step()?;
            }
        }
        Ok(())
    }

    fn print_state(&self) {
        let instr_addr = self.runtime.read_iar();
        println!("Accumulator: {val} {val:#07x} {val:#022b}", val = self.runtime.read_accu());

        if instr_addr > 0 {
            println!("   {}", stringify_instr(self.runtime, instr_addr - 1));
        }
        println!("-> {}", stringify_instr(self.runtime, instr_addr));
        if instr_addr < ADDRESS_SPACE - 1 {
            println!("   {}", stringify_instr(self.runtime, instr_addr + 1));
        }
    }

    fn step(&mut self) {
        self.break_state = false;
        self.break_next = true;
    }

    fn continue_run(&mut self) {
        self.break_state = false;
    }

    fn toggle_breakpoint(&mut self, addr: &str) {
        if let Ok(addr) = parse_mima_addr(addr) {
            if self.breakpoints.contains(&addr) {
                self.breakpoints.remove(&addr);
                println!("Breakpoint at address {:#x} removed", addr);
            } else {
                self.breakpoints.insert(addr);
                println!("Breakpoint set at address {:#x}", addr);
            }
        } else {
            eprintln!("Invalid address {}", addr);
        }
    }

    fn print_mem(&self, addr: &str) {
        if let Ok(addr) = parse_mima_addr(addr) {
            println!("{:#07x}: {val} {val:#07x} {val:#022b}",
                     addr, val = self.runtime.read_mem(addr));
        } else {
            eprintln!("Invalid address {}", addr);
        }
    }

    fn write_mem(&mut self, addr: &str, val: &str) {
        if let [Ok(addr), Ok(val)] = [parse_mima_addr(addr), parse_mima_value(val)] {
            self.runtime.write_mem(addr, val);
            println!("Wrote {} to address {}", addr, val);
        } else {
            eprintln!("Invalid address {} or value {}", addr, val);
        }
    }

    fn make_dump(&self, path: &str) {
        match PathBuf::from_str(path) {
            Ok(buf) => {
                if let Err(error) = create_memdump(&buf, self.runtime) {
                    eprintln!("Unable to create memory dump at {}: {}",
                              path, error.to_string());
                } else {
                    println!("Created memory dump at {}", path);
                }
            }
            Err(error) => {
                eprintln!("Could not read path: {}", error.to_string());
            }
        }
    }

    fn stop(&mut self) {
        self.runtime.halt = true;
        self.break_state = false;
    }

}

fn stringify_instr(runtime: &Runtime, instr_addr: MimaAddress) -> String {
    let instr_str = Instruction::try_from(runtime.read_mem(instr_addr))
        .map_or("???".to_owned(), |i| i.to_string());
    format!("{:#07x}: {}", instr_addr, instr_str)
}