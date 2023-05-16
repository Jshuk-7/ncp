use compiler::Compiler;

use clap::Parser;

use colored::Colorize;
use compiler_args::CompilerArgs;

pub mod compiler;
pub mod compiler_args;

fn main() {
    let args = CompilerArgs::parse();

    if let Err(err) = Compiler::compile(&args) {
        println!("{err}");
        println!(
            "{}: failed to compile program due to previous error",
            "error".red()
        );
    }
}
