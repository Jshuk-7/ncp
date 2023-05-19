use std::path::Path;

use colored::Colorize;

use crate::runtime_args::RuntimeArgs;
use nere_internal::{disassembler::Disassembler, utils, ByteCode, Error, OpCode, Value};

pub type RuntimeResult<T> = std::result::Result<T, Error>;

const STACK_CAPACITY_START: usize = 256;

pub struct VirtualMachine {
    stack: Vec<Value>,
    byte_code: ByteCode,
    ip: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(STACK_CAPACITY_START),
            byte_code: ByteCode::default(),
            ip: 0,
        }
    }

    pub fn execute(&mut self, args: &RuntimeArgs) -> RuntimeResult<()> {
        let mut jmp_to_end = false;

        loop {
            if self.is_at_end() {
                break;
            }

            let ip = self.advance();

            let byte = self.byte_code.bytes[ip];
            let opcode = OpCode::from(byte);

            if args.disassemble {
                let mut offset = ip;
                Disassembler::disassemble_instruction(&self.byte_code, opcode, &mut offset)
            }

            match opcode {
                OpCode::Push => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                OpCode::Add => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(lhs + rhs);
                }
                OpCode::Sub => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(lhs - rhs);
                }
                OpCode::Mul => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(lhs * rhs);
                }
                OpCode::Div => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(lhs / rhs);
                }
                OpCode::Lt => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(Value::Int32((lhs < rhs) as i32));
                }
                OpCode::Lte => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(Value::Int32((lhs <= rhs) as i32));
                }
                OpCode::Gt => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(Value::Int32((lhs > rhs) as i32));
                }
                OpCode::Gte => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(Value::Int32((lhs >= rhs) as i32));
                }
                OpCode::Eq => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(Value::Int32((lhs == rhs) as i32));
                }
                OpCode::Ne => {
                    let rhs = self.stack.pop().unwrap();
                    let lhs = self.stack.pop().unwrap();
                    self.stack.push(Value::Int32((lhs != rhs) as i32));
                }
                OpCode::If(..) => {
                    let value = self.stack.pop().unwrap();

                    let return_addr = self.read_isize();

                    if value.as_i32_implicit() == 1 {
                        jmp_to_end = true;
                    } else {
                        self.jmp(return_addr as usize)?;
                    }
                }
                OpCode::Else(..) => {
                    let return_addr = self.read_isize();

                    if jmp_to_end {
                        self.jmp(return_addr as usize)?;
                    }
                }
                OpCode::Dump => {
                    let value = self.stack.pop().unwrap();
                    println!("{value}");
                }
                OpCode::Halt => {
                    break;
                }
            }

            if args.stack_trace {
                if !self.stack.is_empty() {
                    for value in self.stack.iter() {
                        print!("[ {value} ] ");
                    }
                    println!();
                } else {
                    println!("[ ]");
                }
            }
        }

        Ok(())
    }

    pub fn load_binary(&mut self, binary: String) -> RuntimeResult<()> {
        if !Path::new(&binary).exists() {
            return Err(Error::InvalidFilepath(binary));
        }

        if !binary.ends_with(".nar") && !binary.ends_with(".out") {
            let ext = utils::extension_from_path(&binary);
            return Err(Error::InvalidExtension(ext));
        }

        let path = utils::filename_from_path(binary.as_str());
        println!("{} '{path}'", "Loading Binary".green());

        match std::fs::read(&binary) {
            Ok(mut bytes) => {
                if bytes.is_empty() {
                    return Err(Error::CorruptedBinary);
                }

                let halt_index_bytes: [u8; 8] =
                    bytes.drain(0..=7).collect::<Vec<u8>>().try_into().unwrap();
                let halt_index = usize::from_ne_bytes(halt_index_bytes);

                let mut constant_bytes = bytes.drain(halt_index + 1..).collect::<Vec<u8>>();

                self.byte_code.bytes = bytes;

                self.load_constants(&mut constant_bytes)?;
            }
            Err(..) => return Err(Error::CorruptedBinary),
        }

        Ok(())
    }

    fn advance(&mut self) -> usize {
        self.ip += 1;
        self.ip - 1
    }

    fn jmp(&mut self, to: usize) -> RuntimeResult<()> {
        let len = self.byte_code.bytes.len();
        if to >= len {
            return Err(Error::SegFault(
                self.ip,
                format!("attempting to access restricted memory"),
            ));
        }

        self.ip = to;
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.ip >= self.byte_code.bytes.len()
    }

    fn load_constants(&mut self, constant_bytes: &mut Vec<u8>) -> RuntimeResult<()> {
        loop {
            if constant_bytes.is_empty() {
                break;
            }

            let constant_type = constant_bytes[0];
            constant_bytes.remove(0);

            let constant = match constant_type {
                0 => {
                    let bytes: [u8; 4] = constant_bytes
                        .drain(0..=3)
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap();

                    let int32 = i32::from_ne_bytes(bytes);
                    Ok(Value::Int32(int32))
                }
                1 => {
                    let bytes: [u8; 4] = constant_bytes
                        .drain(0..=3)
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap();

                    let uint32 = u32::from_ne_bytes(bytes);
                    Ok(Value::UInt32(uint32))
                }
                2 => {
                    let len_bytes: [u8; 8] = constant_bytes
                        .drain(0..=7)
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap();

                    let len = usize::from_ne_bytes(len_bytes);

                    let str_bytes = constant_bytes.drain(0..len).collect::<Vec<u8>>();

                    match String::from_utf8(str_bytes) {
                        Ok(string) => Ok(Value::String(string)),
                        Err(..) => Err(Error::InvalidUTF8String),
                    }
                }
                _ => unreachable!(),
            }?;

            self.byte_code.constants.push(constant);
        }

        Ok(())
    }

    fn read_constant(&mut self) -> Value {
        let bytes: [u8; 8] = self.byte_code.bytes[self.ip..self.ip + 8]
            .try_into()
            .unwrap();
        let constant_index = usize::from_ne_bytes(bytes);
        self.ip += 8;
        self.byte_code.constants[constant_index].clone()
    }

    fn read_isize(&mut self) -> isize {
        let bytes: [u8; 8] = self.byte_code.bytes[self.ip..self.ip + 8]
            .try_into()
            .unwrap();
        let value = isize::from_ne_bytes(bytes);
        self.ip += 8;
        value
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}
