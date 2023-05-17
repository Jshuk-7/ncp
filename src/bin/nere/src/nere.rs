use virtual_machine::VirtualMachine;
use runtime_args::RuntimeArgs;

use clap::Parser;
use colored::Colorize;

pub mod virtual_machine;
pub mod runtime_args;

fn main() {
    let args = RuntimeArgs::parse();

    let mut vm = VirtualMachine::default();

    if let Err(err) = vm.load_binary(args.binary) {
        eprintln!("{err}");
        eprintln!(
            "{}: failed to load binary due to previous error",
            "error".red()
        );
    }
}
