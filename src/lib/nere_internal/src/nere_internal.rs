pub mod lexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpCode {
    Push,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Int32(i32),
}

#[derive(Debug, Clone)]
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
    path: String,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
pub struct Token {
    typ3: TokenType,
    lexeme: String,
    location: Location,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}:{}:{}> [{:?} {}]",
            self.location.path, self.location.line, self.location.column, self.typ3, self.lexeme
        )
    }
}

pub mod utils {
    use std::collections::HashMap;

    use crate::OpCode;

    pub fn get_instruction_set() -> HashMap<String, OpCode> {
        vec![("".to_string(), OpCode::Push)]
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect()
    }
}
