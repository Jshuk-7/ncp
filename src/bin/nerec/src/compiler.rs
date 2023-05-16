use std::{fmt::Display, path::Path};

use colored::Colorize;
use nere_internal::{lexer::Lexer, utils, TokenType};

use crate::compiler_args::CompilerArgs;

pub enum Error {
    ParseError(String),
    CompileError(String),
    InvalidFilepath(String),
}

pub type CompileResult<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(err) => write!(f, "{err}"),
            Error::CompileError(err) => write!(f, "{}: {err}", "compile error".red()),
            Error::InvalidFilepath(err) => write!(f, "{}: {err}", "invalid filepath".red()),
        }
    }
}

pub struct Compiler {}

impl Compiler {
    pub fn compile(args: &CompilerArgs) -> CompileResult<()> {
        let input = utils::filename_from_path(&args.input);

        if !Path::new(&input).exists() {
            return Err(Error::InvalidFilepath(input));
        }

        let out = args.output.clone();
        let output = utils::filename_from_path(out.as_deref().unwrap_or("a.out"));

        println!("{} '{input}' -> '{output}'", "Compiling".green(),);

        let mut lexer = Lexer::new(input.clone());
        let tokens = lexer.scan_tokens();

        let error_tokens = tokens
            .iter()
            .filter(|t| matches!(t.typ3, TokenType::Error))
            .collect::<Vec<_>>();

        if !error_tokens.is_empty() {
            let mut err_str = String::new();

            let err_type = format!("{}: ", "parse error".red());

            for token in error_tokens.iter() {
                err_str.push_str(&err_type);
                err_str.push_str(&token.lexeme);
                err_str.push('\n');
            }

            return Err(Error::ParseError(err_str));
        }

        let mut it = tokens.iter();

        while let Some(token) = it.next() {
            if args.display_tokens {
                println!("{token}");
            }
        }

        println!(
            "{} '{}' {}",
            "Compiled".green(),
            input,
            "successfully".green()
        );

        Ok(())
    }
}
