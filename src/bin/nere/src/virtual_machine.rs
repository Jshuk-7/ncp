use std::{fmt::Display, path::Path};

use colored::Colorize;

use nere_internal::{utils, ByteCode};

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
            Ok(bytes) => {
                self.byte_code.bytes = bytes;
            }
            Err(..) => return Err(Error::CorruptedBinary),
        }

        Ok(())
    }
}
