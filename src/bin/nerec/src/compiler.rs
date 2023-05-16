use std::{fmt::Display, fs::File, io::Write, path::Path};

use colored::Colorize;

use crate::compiler_args::CompilerArgs;
use nere_internal::{
    disassembler::Disassembler, lexer::Lexer, utils, ByteCode, OpCode, Token, TokenType,
};

pub enum Error {
    ParseError(String),
    CompileError(String),
    InvalidFilepath(String),
    InvalidExtension(String),
    FailedToCreateFile(String),
}

pub type CompileResult<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(err) => write!(f, "{err}"),
            Error::CompileError(err) => write!(f, "{}: {err}", "compile error".red()),
            Error::InvalidFilepath(err) => write!(f, "{}: {err}", "invalid filepath".red()),
            Error::InvalidExtension(err) => write!(f, "{}: {err}", "invalid extension".red()),
            Error::FailedToCreateFile(err) => write!(f, "{}: {err}", "failed to create file".red()),
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

        if !input.ends_with(".nere") {
            let ext = utils::extension_from_path(&input);
            return Err(Error::InvalidExtension(ext));
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

        let mut byte_code = ByteCode::default();

        for token in tokens.iter() {
            if args.display_tokens {
                println!("{token}");
            }

            Compiler::bytes_from_token(&mut byte_code, token);
        }

        if args.show_bytecode {
            Disassembler::disassemble_byte_code(&byte_code);
        }

        match File::create(output.clone()) {
            Ok(mut file) => {
                file.write_all(&byte_code.bytes).unwrap();
            }
            Err(..) => {
                return Err(Error::FailedToCreateFile(output));
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

    fn bytes_from_token(byte_code: &mut ByteCode, token: &Token) {
        match &token.typ3 {
            TokenType::Instruction(opcode) => {
                byte_code.bytes.push(*opcode as u8);
            }
            TokenType::Value(value) => {
                byte_code.bytes.push(OpCode::Push as u8);
                byte_code.constants.push(value.clone());
                let constant_index = byte_code.constants.len() - 1;
                let constant_bytes: [u8; 8] = constant_index.to_ne_bytes();

                byte_code.bytes.extend_from_slice(&constant_bytes);
            }
            TokenType::Error => unreachable!(),
            TokenType::Eof => {
                byte_code.bytes.push(OpCode::Halt as u8);
            }
        }
    }
}
