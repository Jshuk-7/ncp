use std::path::Path;

use colored::Colorize;

use nere_internal::{disassembler::Disassembler, utils, ByteCode, Error, OpCode, Value};

use crate::runtime_args::RuntimeArgs;

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

            if args.stack_trace {
                if !self.stack.is_empty() {
                    for value in self.stack.iter() {
                        print!("[ {value} ] ");
                    }
                    println!();
                }
            }

            match opcode {
                OpCode::Push => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                OpCode::Halt => {
                    break;
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
        println!("{} '{path}'", "Loading binary".green());

        match std::fs::read(&binary) {
            Ok(mut bytes) => {
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

    fn is_at_end(&self) -> bool {
        self.ip >= self.byte_code.bytes.len()
    }

    fn load_constants(&mut self, constant_bytes: &mut Vec<u8>) -> RuntimeResult<()> {
        let mut offset = 0;

        loop {
            if offset >= constant_bytes.len() {
                break;
            }

            let constant_type = constant_bytes[offset];
            constant_bytes.remove(0);

            let constant = match constant_type {
                0b00 => {
                    let bytes: [u8; 4] = constant_bytes
                        .drain(0..=3)
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap();

                    let int32 = i32::from_ne_bytes(bytes);
                    offset += 4;
                    Value::Int32(int32)
                }
                _ => unreachable!(),
            };

            self.byte_code.constants.push(constant);
        }

        Ok(())
    }

    fn read_constant(&mut self) -> Value {
        let bytes: [u8; 8] = self.byte_code.bytes[self.ip..self.ip + 8].try_into().unwrap();
        let constant_index = usize::from_ne_bytes(bytes);
        self.ip += 8;
        self.byte_code.constants[constant_index].clone()
    }
}