use compiler::Compiler;
use compiler_args::CompilerArgs;

use clap::Parser;
use colored::Colorize;

pub mod compiler;
pub mod compiler_args;

fn main() {
    let args = CompilerArgs::parse();

    if let Err(err) = Compiler::compile(&args) {
        eprintln!("{err}");
        eprintln!(
            "{}: failed to compile program due to previous error",
            "error".red()
        );
    }
}
