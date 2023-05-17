pub mod disassembler;
pub mod lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpCode {
    Push,
    Halt,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        use OpCode::*;
        match value {
            0b00 => Push,
            0b01 => Halt,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Int32(i32),
}

impl Value {
    pub fn constant_type(&self) -> u8 {
        match self {
            Value::Int32(..) => 0b00,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int32(int32) => write!(f, "{int32}"),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ByteCode {
    pub bytes: Vec<u8>,
    pub constants: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenType {
    Instruction(OpCode),
    Value(Value),
    Error,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub path: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}:{}:{}>", self.path, self.line, self.column)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ3: TokenType,
    pub lexeme: String,
    pub location: Location,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{:?} {}]", self.location, self.typ3, self.lexeme)
    }
}

pub mod utils {
    use std::{collections::HashMap, path::Path};

    use crate::OpCode;

    pub fn get_instruction_set() -> HashMap<String, OpCode> {
        vec![("".to_string(), OpCode::Push)]
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }

    pub fn filename_from_path(path: &str) -> String {
        let p = Path::new(path);
        p.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn extension_from_path(path: &str) -> String {
        let p = Path::new(path);
        p.extension().unwrap().to_str().unwrap().to_string()
    }
}
