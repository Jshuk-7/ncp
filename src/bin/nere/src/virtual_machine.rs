use std::{fmt::Display, path::Path};

use colored::Colorize;

use nere_internal::{utils, ByteCode, Value, disassembler::Disassembler};

pub enum Error {
    CorruptedBinary,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CorruptedBinary => write!(f, "{}", "corrupted binary".red()),
        }
    }
}

pub type RuntimeResult<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct VirtualMachine {
    byte_code: ByteCode,
}

impl VirtualMachine {
    pub fn load_binary(&mut self, binary: String) -> RuntimeResult<()> {
        if !Path::new(&binary).exists() {
            return Err(Error::CorruptedBinary);
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

    fn load_constants(&mut self, constant_bytes: &mut Vec<u8>) -> RuntimeResult<()> {
        let mut offset = 0;

        loop {
            if offset >= constant_bytes.len() {
                break;
            }

            let constant_type = constant_bytes[offset];

            let constant = match constant_type {
                0b00 => {
                    assert!(constant_bytes.len() >= 4);
                    let bytes: [u8; 4] = constant_bytes
                        .drain(offset + 1..=offset + 4)
                        .collect::<Vec<u8>>()
                        .try_into()
                        .unwrap();
                    let int32 = i32::from_ne_bytes(bytes);
                    offset += 5;
                    Value::Int32(int32)
                }
                _ => unreachable!(),
            };

            self.byte_code.constants.push(constant);
        }

        Ok(())
    }
}
