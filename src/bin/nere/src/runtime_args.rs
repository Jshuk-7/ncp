use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author = "https://github.com/Jshuk-7",
    version = "0.1.0",
    about = "nere programming language runtime"
)]
pub struct RuntimeArgs {
    /// The compiled program to run
    pub binary: String,

    /// Show a breakdown of the bytecode during runtime
    #[arg(short = 'd', long = "disassemble")]
    pub show_bytecode: bool,
}
