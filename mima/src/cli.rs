use std::path::PathBuf;
use mima_common::types::MimaAddress;
use clap::Clap;


/// mimavm is an emulator of the "minimal machine" (mima) used in various
/// universities to teach low level computing.
///
/// This tool lets you assemble, disassemble, run and debug mima instructions.
#[derive(Clap)]
#[clap(version = "0.1.0", author = "JohnnyJayJay")]
pub struct MainOpts {

    #[clap(subcommand)]
    pub cmd: SubCommand,

}

impl MainOpts {
    pub fn file(&self) -> &PathBuf {
        match &self.cmd {
            SubCommand::Asm(opts) => &opts.file,
            SubCommand::Run(opts) => &opts.file
        }
    }
}

#[derive(Clap)]
pub enum SubCommand {
    /// Assemble/Disassemble mima instructions
    Asm(AsmOpts),
    /// Run/Debug mima instructions
    Run(RunOpts)
}

#[derive(Clap)]
pub struct AsmOpts {
    /// Disassemble input file
    #[clap(short, long)]
    pub disassemble: bool,
    /// Do not relativize addresses used in assembly
    #[clap(short, long)]
    pub absolute: bool,

    /// File to output the result of the operation to.
    #[clap(short, long, value_name = "FILE", required_unless_present = "disassemble")]
    pub output: Option<PathBuf>,

    /// The file to assemble/disassemble
    file: PathBuf
}

#[derive(Clap)]
pub struct RunOpts {
    /// Enables debug mode
    #[clap(short, long)]
    pub debug: bool,
    /// Outputs the values at the given addresses in decimal format
    /// to the console upon termination
    #[clap(short, long = "--print-absolute-addresses", value_name = "ADDR")]
    pub abs_output: Option<Vec<MimaAddress>>,
    /// Like -a, but interprets the given addresses as
    /// relative to the last input instruction address.
    /// These addresses will be printed after the absolute ones, if any
    #[clap(short, long = "--print-relative-addresses", value_name = "ADDR")]
    pub rel_output: Option<Vec<MimaAddress>>,

    /// Dumps the VM's memory to the specified file upon termination
    #[clap(short, long, value_name = "FILE")]
    pub memdump: Option<PathBuf>,

    /// The binary to run
    file: PathBuf
}