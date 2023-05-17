pub mod disassembler;
pub mod lexer;
pub mod timer;

use std::ops::{Add, Div, Mul, Sub};

use colored::Colorize;

pub enum Error {
    RuntimeError(String),
    ParseError(String),
    CompileError(String, Location),
    InvalidFilepath(String),
    InvalidExtension(String),
    FailedToCreateFile(String),
    CorruptedBinary,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RuntimeError(err) => write!(f, "{}: {err}", "uncaught runtime error".red()),
            Error::ParseError(err) => write!(f, "{err}"),
            Error::CompileError(err, loc) => write!(f, "{loc} {}: {err}", "compile error".red()),
            Error::InvalidFilepath(err) => write!(f, "{}: '{err}'", "invalid filepath".red()),
            Error::InvalidExtension(err) => write!(f, "{}: '{err}'", "invalid extension".red()),
            Error::FailedToCreateFile(err) => {
                write!(f, "{}: '{err}'", "failed to create file".red())
            }
            Error::CorruptedBinary => {
                write!(f, "{}: failed to read bytecode", "corrupted binary".red())
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpCode {
    Push,
    Add,
    Sub,
    Mul,
    Div,
    Halt,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        use OpCode::*;
        match value {
            0 => Push,
            1 => Add,
            2 => Sub,
            3 => Mul,
            4 => Div,
            5 => Halt,
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
            Value::Int32(..) => 0,
        }
    }

    pub fn as_i32(&self) -> i32 {
        debug_assert!(self.constant_type() == 0);
        if let Value::Int32(int32) = *self {
            return int32;
        }
        return 0;
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(self.constant_type() == rhs.constant_type());

        match self {
            Value::Int32(lhs) => {
                Value::Int32(lhs + rhs.as_i32())
            },
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(self.constant_type() == rhs.constant_type());

        match self {
            Value::Int32(lhs) => {
                Value::Int32(lhs - rhs.as_i32())
            },
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(self.constant_type() == rhs.constant_type());

        match self {
            Value::Int32(lhs) => {
                Value::Int32(lhs * rhs.as_i32())
            },
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        debug_assert!(self.constant_type() == rhs.constant_type());

        match self {
            Value::Int32(lhs) => {
                let rhs = rhs.as_i32();
                debug_assert!(rhs != 0);
                Value::Int32(lhs / rhs)
            },
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
