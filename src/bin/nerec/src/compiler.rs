use std::{fs::File, io::Write, path::Path};

use colored::Colorize;

use crate::compiler_args::CompilerArgs;
use nere_internal::{
    disassembler::Disassembler, lexer::Lexer, timer::Timer, utils, ByteCode, Error, OpCode, Token,
    TokenType, Value,
};

pub type CompileResult<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct Compiler {}

impl Compiler {
    pub fn compile(&self, args: &CompilerArgs) -> CompileResult<()> {
        let timer = Timer::new();
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

            for (i, token) in error_tokens.iter().enumerate() {
                let err_type = format!("{} {}: ", token.location, "parse error".red());
                err_str.push_str(&err_type);
                err_str.push_str(&token.lexeme);

                if i != error_tokens.len() - 1 {
                    err_str.push('\n');
                }
            }

            return Err(Error::ParseError(err_str));
        }

        let mut byte_code = ByteCode::default();

        for token in tokens.iter() {
            if args.display_tokens {
                println!("{token}");
            }

            self.bytes_from_token(&mut byte_code, token);
        }

        if args.disassemble {
            Disassembler::disassemble_byte_code(&byte_code);
        }

        match File::create(output.clone()) {
            Ok(mut file) => {
                file.write_all(&byte_code.bytes).unwrap();
                let constant_bytes = self.constants_to_bytes(&byte_code.constants);
                file.write_all(&constant_bytes).unwrap();
            }
            Err(..) => {
                return Err(Error::FailedToCreateFile(output));
            }
        }

        println!("{} '{}' in {}s", "Finished".green(), input, timer.elapsed());

        Ok(())
    }

    fn bytes_from_token(&self, byte_code: &mut ByteCode, token: &Token) {
        match &token.typ3 {
            TokenType::Instruction(opcode) => {
                byte_code.bytes.push(*opcode as u8);
            }
            TokenType::Value(value) => {
                byte_code.bytes.push(OpCode::Push as u8);
                byte_code.constants.push(value.clone());
                let constant_index = byte_code.constants.len() - 1;
                let bytes: [u8; 8] = constant_index.to_ne_bytes();

                byte_code.bytes.extend_from_slice(&bytes);
            }
            TokenType::Error => unreachable!(),
            TokenType::Eof => {
                byte_code.bytes.push(OpCode::Halt as u8);
                let halt_index = byte_code.bytes.len() - 1;
                let bytes: [u8; 8] = halt_index.to_ne_bytes();

                byte_code.bytes.splice(0..0, bytes);
            }
        }
    }

    fn constants_to_bytes(&self, constants: &[Value]) -> Vec<u8> {
        let mut result = vec![];

        for constant in constants.iter() {
            match constant {
                Value::Int32(int32) => {
                    result.push(constant.constant_type());
                    let bytes: [u8; 4] = int32.to_ne_bytes();
                    result.extend_from_slice(&bytes);
                }
                Value::String(string) => {
                    result.push(constant.constant_type());
                    let len = string.len();
                    let len_as_bytes: [u8; 8] = len.to_ne_bytes();
                    result.extend_from_slice(&len_as_bytes);
                    result.extend_from_slice(string.as_bytes());
                }
            }
        }

        result
    }
}
