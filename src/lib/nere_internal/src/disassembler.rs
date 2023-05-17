use crate::{ByteCode, OpCode};

pub struct Disassembler {}

impl Disassembler {
    pub fn disassemble_byte_code(byte_code: &ByteCode) {
        // we start 8 bytes deep because we need to skip
        // over the halt index
        let mut offset = 8;

        loop {
            let byte = byte_code.bytes[offset];

            let opcode = OpCode::from(byte);
            print!("{:04} [{opcode:?}] ", offset - 8);
            match opcode {
                OpCode::Push => {
                    let bytes = [
                        byte_code.bytes[offset + 1],
                        byte_code.bytes[offset + 2],
                        byte_code.bytes[offset + 3],
                        byte_code.bytes[offset + 4],
                        byte_code.bytes[offset + 5],
                        byte_code.bytes[offset + 6],
                        byte_code.bytes[offset + 7],
                        byte_code.bytes[offset + 8],
                    ];
                    let constant_index = usize::from_ne_bytes(bytes);
                    let constant = byte_code.constants[constant_index].clone();
                    println!("{constant_index:04} '{constant}'");
                    offset += 9;
                }
                OpCode::Halt => {
                    println!();
                    break;
                }
            }
        }
    }
}
