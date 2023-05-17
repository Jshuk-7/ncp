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

    /// Show a breakdown of the bytecode during execution
    #[arg(short = 'd', long = "disassemble")]
    pub disassemble: bool,

    /// Show a breakdown of the stack during execution
    #[arg(short = 's', long = "stack-trace")]
    pub stack_trace: bool,
}
