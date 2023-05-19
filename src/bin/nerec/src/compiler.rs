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
        let timer = Timer::default();
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
        let mut tokens = lexer.scan_tokens();

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

        self.preprocess_program(&mut tokens)?;
        // we must verify that preprocessing went ok otherise we dip
        // ! NOTE: this could be disabled for a release build
        // ! but better safe than segfault
        self.verify_cross_reference_blocks(&tokens)?;

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

        let mut constant_bytes = self.constants_to_bytes(&byte_code.constants);
        byte_code.bytes.append(&mut constant_bytes);

        match File::create(output.clone()) {
            Ok(mut file) => {
                file.write_all(&byte_code.bytes).unwrap();
            }
            Err(..) => {
                return Err(Error::FailedToCreateFile(output));
            }
        }

        println!("{} '{}' in {}s", "Finished".green(), input, timer.elapsed());

        Ok(())
    }

    fn preprocess_program(&self, tokens: &mut [Token]) -> CompileResult<()> {
        let mut stack = vec![];
        let mut count = 0;
        let mut ip = 0;

        let mut hit_else_block = false;

        loop {
            match &tokens[count].typ3 {
                TokenType::Instruction(opcode) => match opcode {
                    OpCode::Push
                    | OpCode::Add
                    | OpCode::Sub
                    | OpCode::Mul
                    | OpCode::Div
                    | OpCode::Lt
                    | OpCode::Lte
                    | OpCode::Gt
                    | OpCode::Gte
                    | OpCode::Eq
                    | OpCode::Ne
                    | OpCode::Dump
                    | OpCode::Halt => {
                        ip += 1;
                        count += 1;
                    }
                    OpCode::If(..) => {
                        stack.push(count);
                        ip += 9;
                        count += 1;
                    }
                    OpCode::Else(..) => {
                        let if_ip = stack.pop().unwrap();
                        match &mut tokens[if_ip].typ3 {
                            TokenType::Instruction(inst) => *inst = OpCode::If(ip as isize),
                            _ => {
                                return Err(Error::CompileError(
                                    "'end' can only close if blocks".to_string(),
                                    tokens[if_ip].location.clone(),
                                ));
                            }
                        }

                        stack.push(count);
                        ip += 9;
                        count += 1;
                        hit_else_block = true;
                    }
                },
                TokenType::Value(..) => {
                    ip += 9;
                    count += 1;
                }
                TokenType::LBrace => {
                    count += 1;
                }
                TokenType::RBrace => {
                    if !stack.is_empty() {
                        if hit_else_block {
                            let else_ip = stack.pop().unwrap();
                            match &mut tokens[else_ip].typ3 {
                                TokenType::Instruction(inst) => *inst = OpCode::Else(ip as isize),
                                _ => {
                                    return Err(Error::CompileError(
                                        "failed to cross reference 'else' token with return address"
                                            .to_string(),
                                        tokens[else_ip].location.clone(),
                                    ));
                                }
                            }
                            hit_else_block = false;
                        } else if !matches!(
                            tokens[count + 1].typ3,
                            TokenType::Instruction(OpCode::Else(..))
                        ) {
                            let if_ip = stack.pop().unwrap();
                            match &mut tokens[if_ip].typ3 {
                                TokenType::Instruction(inst) => *inst = OpCode::If(ip as isize),
                                _ => {
                                    return Err(Error::CompileError(
                                        "failed to cross reference 'if' token with return address"
                                            .to_string(),
                                        tokens[if_ip].location.clone(),
                                    ));
                                }
                            }
                        }
                    }

                    count += 1;
                }
                TokenType::Error => (),
                TokenType::Eof => break,
            }
        }

        Ok(())
    }

    fn verify_cross_reference_blocks(&self, tokens: &[Token]) -> CompileResult<()> {
        for token in tokens.iter() {
            match token.typ3 {
                TokenType::Instruction(opcode) => match opcode {
                    OpCode::If(return_addr) => {
                        if return_addr < 0 {
                            return Err(Error::CompileError(
                                format!(
"invalid return address '{return_addr}',
block was not referenced with end instruction pointer
-----------------------------------
to fix this use '{{' and '}}' to allow the compiler to detect the end of the block"
                                ),
                                token.location.clone(),
                            ));
                        }
                    }
                    OpCode::Else(return_addr) => {
                        if return_addr < 0 {
                            return Err(Error::CompileError(
                                format!(
"invalid return address '{return_addr}'
block was not referenced with end instruction pointer
-----------------------------------
to fix this use '{{' and '}}' to allow the compiler to detect the end of the block"
                                ),
                                token.location.clone(),
                            ));
                        }
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        Ok(())
    }

    fn bytes_from_token(&self, byte_code: &mut ByteCode, token: &Token) {
        match &token.typ3 {
            TokenType::Instruction(opcode) => match opcode {
                OpCode::If(return_addr) => {
                    byte_code.bytes.push(opcode.as_byte());
                    let bytes: [u8; 8] = return_addr.to_ne_bytes();
                    byte_code.bytes.extend_from_slice(&bytes);
                }
                OpCode::Else(return_addr) => {
                    byte_code.bytes.push(opcode.as_byte());
                    let bytes: [u8; 8] = return_addr.to_ne_bytes();
                    byte_code.bytes.extend_from_slice(&bytes);
                }
                _ => {
                    byte_code.bytes.push(opcode.as_byte());
                }
            },
            TokenType::Value(value) => {
                byte_code.bytes.push(OpCode::Push.as_byte());
                byte_code.constants.push(value.clone());
                let constant_index = byte_code.constants.len() - 1;
                let bytes: [u8; 8] = constant_index.to_ne_bytes();
                byte_code.bytes.extend_from_slice(&bytes);
            }
            TokenType::LBrace => (),
            TokenType::RBrace => (),
            TokenType::Error => unreachable!(),
            TokenType::Eof => {
                byte_code.bytes.push(OpCode::Halt.as_byte());
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
                Value::UInt32(uint32) => {
                    result.push(constant.constant_type());
                    let bytes: [u8; 4] = uint32.to_ne_bytes();
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
