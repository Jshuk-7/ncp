pub mod disassembler;
pub mod lexer;
pub mod timer;

use std::ops::{Add, Div, Mul, Sub};

use colored::Colorize;

pub enum Error {
    RuntimeError(String),
    SegFault(usize, String),
    ParseError(String),
    CompileError(String, Location),
    InvalidFilepath(String),
    InvalidExtension(String),
    FailedToCreateFile(String),
    InvalidUTF8String,
    CorruptedBinary,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RuntimeError(err) => write!(f, "{}: {err}", "uncaught runtime error".red()),
            Error::SegFault(ip, err) => {
                write!(f, "{}: at ip: {ip}, {err}", "segmentation fault".red())
            }
            Error::ParseError(err) => write!(f, "{err}"),
            Error::CompileError(err, loc) => write!(f, "{loc} {}: {err}", "compile error".red()),
            Error::InvalidFilepath(err) => write!(f, "{}: '{err}'", "invalid filepath".red()),
            Error::InvalidExtension(err) => write!(f, "{}: '{err}'", "invalid extension".red()),
            Error::FailedToCreateFile(err) => {
                write!(f, "{}: '{err}'", "failed to create file".red())
            }
            Error::InvalidUTF8String => {
                write!(
                    f,
                    "{}: failed to read binary string",
                    "invalid utf-8 string".red()
                )
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
    Dup,
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Ne,
    If(isize),
    Else(isize),
    While,
    Do(isize),
    Dump,
    Halt,
    LBrace,
    RBrace(isize),
}

impl OpCode {
    pub fn as_byte(&self) -> u8 {
        use OpCode::*;
        match self {
            Push => 0,
            Dup => 1,
            Add => 2,
            Sub => 3,
            Mul => 4,
            Div => 5,
            Lt => 6,
            Lte => 7,
            Gt => 8,
            Gte => 9,
            Eq => 10,
            Ne => 11,
            If(..) => 12,
            Else(..) => 13,
            While => 14,
            Do(..) => 15,
            Dump => 16,
            Halt => 17,
            LBrace => 18,
            RBrace(..) => 19,
        }
    }
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        use OpCode::*;
        match value {
            0 => Push,
            1 => Dup,
            2 => Add,
            3 => Sub,
            4 => Mul,
            5 => Div,
            6 => Lt,
            7 => Lte,
            8 => Gt,
            9 => Gte,
            10 => Eq,
            11 => Ne,
            12 => If(-1),
            13 => Else(-1),
            14 => While,
            15 => Do(-1),
            16 => Dump,
            17 => Halt,
            18 => LBrace,
            19 => RBrace(-1),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Int32(i32),
    UInt32(u32),
    String(String),
}

impl Value {
    pub fn constant_type(&self) -> u8 {
        match self {
            Value::Int32(..) => 0,
            Value::UInt32(..) => 1,
            Value::String(..) => 2,
        }
    }

    pub fn as_i32(&self) -> i32 {
        debug_assert!(self.constant_type() == 0);
        if let Value::Int32(int32) = self {
            return *int32;
        }
        0
    }

    pub fn as_i32_implicit(&self) -> i32 {
        match self {
            Value::Int32(..) => self.as_i32(),
            Value::UInt32(..) => self.as_u32() as i32,
            Value::String(..) => unreachable!(),
        }
    }

    pub fn as_u32(&self) -> u32 {
        debug_assert!(self.constant_type() == 0);
        if let Value::UInt32(uint32) = self {
            return *uint32;
        }
        0
    }

    pub fn as_string(&self) -> String {
        debug_assert!(self.constant_type() == 1);
        if let Value::String(string) = self {
            return string.clone();
        }
        "".to_string()
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        debug_assert!(self.constant_type() == rhs.constant_type());

        match self {
            Value::Int32(lhs) => Value::Int32(lhs + rhs.as_i32_implicit()),
            Value::UInt32(lhs) => Value::UInt32(lhs + rhs.as_u32()),
            Value::String(lhs) => match rhs {
                Value::Int32(int32) => Value::String(lhs + &int32.to_string()),
                Value::UInt32(uint32) => Value::String(lhs + &uint32.to_string()),
                Value::String(rhs) => Value::String(lhs + &rhs),
            },
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        debug_assert!(self.constant_type() == rhs.constant_type());

        match self {
            Value::Int32(lhs) => Value::Int32(lhs - rhs.as_i32_implicit()),
            Value::UInt32(lhs) => Value::UInt32(lhs - rhs.as_u32()),
            Value::String(..) => todo!(),
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert!(self.constant_type() == rhs.constant_type());

        match self {
            Value::Int32(lhs) => Value::Int32(lhs * rhs.as_i32_implicit()),
            Value::UInt32(lhs) => Value::UInt32(lhs * rhs.as_u32()),
            Value::String(..) => todo!(),
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        debug_assert!(self.constant_type() == rhs.constant_type());

        match self {
            Value::Int32(lhs) => {
                let rhs = rhs.as_i32_implicit();
                debug_assert!(rhs != 0);
                Value::Int32(lhs / rhs)
            }
            Value::UInt32(lhs) => {
                let rhs = rhs.as_u32();
                debug_assert!(rhs != 0);
                Value::UInt32(lhs / rhs)
            }
            Value::String(..) => todo!(),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int32(int32) => write!(f, "{int32}"),
            Value::UInt32(uint32) => write!(f, "{uint32}"),
            Value::String(string) => write!(f, "{string}"),
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
        vec![
            ("dup", OpCode::Dup),
            ("if", OpCode::If(-1)),
            ("else", OpCode::Else(-1)),
            ("while", OpCode::While),
            ("do", OpCode::Do(-1)),
        ]
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
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
