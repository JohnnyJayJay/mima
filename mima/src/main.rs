mod debugger;
mod disassembly;
mod assembly;
mod cli;

use std::path::PathBuf;
use clap::Clap;
use mima_common::types::{WriteMimaExt, ReadMimaExt, MimaValue};
use std::fs::File;
use std::io;
use mima_common::runtime::Runtime;
use crate::debugger::Debugger;
use std::io::{Write, Read};
use crate::disassembly::disassemble;
use crate::assembly::assemble;
use crate::cli::{MainOpts, SubCommand, AsmOpts, RunOpts};


fn main() -> Result<(), String> {
    let opts: MainOpts = MainOpts::parse();
    let input = File::open(&opts.file())
        .map_err(|e| format!("Could not open input file: {}", e.to_string()))?;

    match &opts.cmd {
        SubCommand::Asm(asm_opts) => run_asm(input, asm_opts),
        SubCommand::Run(run_opts) => run_run(input, run_opts)
    }?;
    Ok(())
}

fn run_asm(mut input: File, opts: &AsmOpts) -> Result<(), String> {
    let output = || {
        if let Some(path) = &opts.output {
            File::create(path)
                .map_err(|e| format!("Could not open output file: {}", e.to_string()))
                .map(|f| Some(f))
        } else {
            Ok(None)
        }
    };
    if opts.disassemble {
        let values = read_mima_file(&mut input)?;
        let asm = disassemble(values)?;
        write_to_output(output()?.as_mut(), |w| w.write_all(asm.as_bytes()))?;
    } else {
        let mut content = String::new();
        input.read_to_string(&mut content).map_err(|e| e.to_string())?;
        let instructions: Vec<MimaValue> = assemble(content, opts.absolute)?
            .iter()
            .map(|i| MimaValue::from(i))
            .collect();
        write_mima_file(&mut output()?.unwrap(), instructions.as_slice())?;
    }
    Ok(())
}

fn run_run(mut input: File, opts: &RunOpts) -> Result<(), String> {
    let instructions = read_mima_file(&mut input)?;
    let mut runtime = Runtime::with_instructions(&instructions);
    if opts.debug {
        Debugger::from(&mut runtime).run()
    } else {
        runtime.run()
    }?;

    let mut addresses = opts.abs_output.clone()
        .unwrap_or_else(|| Vec::new());
    addresses.extend(opts.rel_output.clone().unwrap_or_else(|| Vec::new()).iter()
        .map(|addr| *addr + instructions.len() as u32 + 1));
    addresses.iter()
        .map(|addr| runtime.read_mem(*addr))
        .for_each(|v| println!("{}", v));


    if let Some(ref path) = &opts.memdump {
        create_memdump(path, &runtime)?;
    }

    Ok(())

}

fn read_mima_file(file: &mut File) -> Result<Vec<MimaValue>, String> {
    file.read_all_mima_vals()
        .map_err(|e| format!("Failed to parse mima file: {}", e.to_string()))
}

fn create_memdump(path: &PathBuf, runtime: &Runtime) -> Result<(), String> {
    File::create(path)
        .map_err(|e| format!("Could not open memdump file: {}", e.to_string()))
        .and_then(|mut file| write_mima_file(&mut file, runtime.mem_iter().as_slice()))
}

fn write_mima_file(file: &mut File, vals: &[MimaValue]) -> Result<(), String> {
    file.write_all_mima_vals(vals)
        .map_err(|e| format!("Could not write mima file: {}", e.to_string()))
}

fn write_to_output<O: FnOnce(&mut dyn Write) -> io::Result<()>>(file: Option<&mut File>, op: O)
    -> Result<(), String> {
    (if let Some(file) = file {
        op(file)
    } else {
        op(&mut io::stdout())
    }).map_err(|e| format!("Could not write to output: {}", e.to_string()))
}
