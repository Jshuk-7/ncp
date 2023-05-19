use std::collections::HashMap;

use crate::{utils, Location, OpCode, Token, TokenType, Value};

pub struct Lexer {
    path: String,
    source: String,
    chars: Vec<char>,
    cursor: usize,
    start: usize,
    line_start: usize,
    line: usize,
    instruction_set: HashMap<String, OpCode>,
}

impl Lexer {
    pub fn new(path: String) -> Self {
        let source = match std::fs::read_to_string(path.clone()) {
            Ok(s) => s,
            Err(err) => panic!("{err}"),
        };

        Self {
            path,
            source: source.clone(),
            chars: source.chars().collect(),
            cursor: 0,
            start: 0,
            line_start: 0,
            line: 1,
            instruction_set: utils::get_instruction_set(),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = vec![];

        while !self.is_at_end() {
            self.start = self.cursor;
            let c = self.advance();

            if c.is_ascii_digit() {
                while self.peek().is_ascii_digit() {
                    self.advance();
                }

                let lexeme = self.current_lexeme();
                let value = Value::Int32(lexeme.parse::<i32>().unwrap());
                let number = self.make_token(TokenType::Value(value), lexeme);
                tokens.push(number);
                continue;
            } else if c.is_alphabetic() {
                while self.peek().is_alphabetic() && !self.is_at_end() {
                    self.advance();
                }

                let lexeme = self.current_lexeme();
                if self.instruction_set.contains_key(&lexeme) {
                    let opcode = self.instruction_set.get(&lexeme).unwrap();
                    let instruction = self.make_token(TokenType::Instruction(*opcode), lexeme);
                    tokens.push(instruction);
                } else {
                    let err = self.error_token(format!("unknown identifier '{lexeme}'"));
                    tokens.push(err);
                }

                continue;
            }

            match c {
                '{' => {
                    let lexeme = self.current_lexeme();
                    let lbrace = self.make_token(TokenType::LBrace, lexeme);
                    tokens.push(lbrace);
                }
                '}' => {
                    let lexeme = self.current_lexeme();
                    let rbrace = self.make_token(TokenType::RBrace, lexeme);
                    tokens.push(rbrace);
                }
                '+' => {
                    let lexeme = self.current_lexeme();
                    let add = self.make_token(TokenType::Instruction(OpCode::Add), lexeme);
                    tokens.push(add);
                }
                '-' => {
                    let lexeme = self.current_lexeme();
                    let sub = self.make_token(TokenType::Instruction(OpCode::Sub), lexeme);
                    tokens.push(sub);
                }
                '*' => {
                    let lexeme = self.current_lexeme();
                    let mul = self.make_token(TokenType::Instruction(OpCode::Mul), lexeme);
                    tokens.push(mul);
                }
                '/' => {
                    if self.matches('/') {
                        // comments
                    } else {
                        let lexeme = self.current_lexeme();
                        let div = self.make_token(TokenType::Instruction(OpCode::Div), lexeme);
                        tokens.push(div);
                    }
                }
                '<' => {
                    if self.matches('=') {
                        let lexeme = self.current_lexeme();
                        let lte = self.make_token(TokenType::Instruction(OpCode::Lte), lexeme);
                        tokens.push(lte);
                    } else {
                        let lexeme = self.current_lexeme();
                        let lt = self.make_token(TokenType::Instruction(OpCode::Lt), lexeme);
                        tokens.push(lt);
                    }
                }
                '>' => {
                    if self.matches('=') {
                        let lexeme = self.current_lexeme();
                        let gte = self.make_token(TokenType::Instruction(OpCode::Gte), lexeme);
                        tokens.push(gte);
                    } else {
                        let lexeme = self.current_lexeme();
                        let gt = self.make_token(TokenType::Instruction(OpCode::Gt), lexeme);
                        tokens.push(gt);
                    }
                }
                '=' => {
                    let lexeme = self.current_lexeme();
                    let eq = self.make_token(TokenType::Instruction(OpCode::Eq), lexeme);
                    tokens.push(eq);
                }
                '!' => {
                    let lexeme = self.current_lexeme();
                    let ne = self.make_token(TokenType::Instruction(OpCode::Ne), lexeme);
                    tokens.push(ne);
                }
                '.' => {
                    let lexeme = self.current_lexeme();
                    let dump = self.make_token(TokenType::Instruction(OpCode::Dump), lexeme);
                    tokens.push(dump);
                }
                '"' => {
                    while !self.is_at_end() && self.peek() != '"' {
                        if self.peek() == '\n' {
                            self.line += 1;
                            self.line_start = self.cursor;
                        }

                        self.advance();
                    }

                    if self.peek() != '"' {
                        let err = self.error_token("unterminated string literal".to_string());
                        tokens.push(err);
                        continue;
                    }

                    self.advance();

                    let mut lexeme = self.current_lexeme();
                    lexeme.remove(0);
                    lexeme.remove(lexeme.len() - 1);
                    let value = Value::String(lexeme.clone());
                    let string = self.make_token(TokenType::Value(value), lexeme);
                    tokens.push(string);
                }
                '\r' | '\t' | ' ' => (),
                '\n' => {
                    self.line += 1;
                    self.line_start = self.cursor;
                }
                _ => {
                    let error = self.error_token(format!("unexpected character '{c}'"));
                    tokens.push(error);
                }
            }
        }

        let eof = self.make_token(TokenType::Eof, self.current_lexeme());
        tokens.push(eof);

        tokens
    }

    fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.cursor += 1;
        self.chars[self.cursor - 1]
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.chars[self.cursor]
    }

    fn matches(&mut self, c: char) -> bool {
        if self.peek() == c {
            self.advance();
            return true;
        }

        false
    }

    fn current_lexeme(&self) -> String {
        String::from(&self.source[self.start..self.cursor])
    }

    fn cursor_location(&self) -> Location {
        Location {
            path: self.path.clone(),
            line: self.line,
            column: self.cursor - self.line_start,
        }
    }

    fn error_token(&self, msg: String) -> Token {
        Token {
            typ3: TokenType::Error,
            lexeme: msg,
            location: self.cursor_location(),
        }
    }

    fn make_token(&self, typ3: TokenType, lexeme: String) -> Token {
        Token {
            typ3,
            lexeme,
            location: self.cursor_location(),
        }
    }

    fn is_at_end(&self) -> bool {
        self.cursor >= self.source.len()
    }
}
