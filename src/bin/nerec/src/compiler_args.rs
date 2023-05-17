use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    author = "https://github.com/Jshuk-7",
    version = "0.1.0",
    about = "nere programming language compiler"
)]
pub struct CompilerArgs {
    /// The source file to be compiled
    pub input: String,

    /// The output file of the compiler
    pub output: Option<String>,

    /// Display language tokens during compilation
    #[arg(short = 't', long = "tokens")]
    pub display_tokens: bool,

    /// Override the default output file 'a.out'
    #[arg(short = 'o', long = "output")]
    pub override_output: bool,

    /// Show a breakdown of the bytecode after compilation
    #[arg(short = 'd', long = "disassemble")]
    pub disassemble: bool,
}
