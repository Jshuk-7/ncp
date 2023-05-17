use crate::{ByteCode, OpCode};

pub struct Disassembler {}

impl Disassembler {
    pub fn disassemble_byte_code(byte_code: &ByteCode) {
        // we start 8 bytes deep because we need to skip
        // over the halt index
        let mut offset = 8;

        loop {
            if offset >= byte_code.bytes.len() {
                break;
            }

            let byte = byte_code.bytes[offset];
            let opcode = OpCode::from(byte);
            let mut o = offset - 8;
            Disassembler::disassemble_instruction(byte_code, opcode, &mut o);
            offset = o + 8;
        }
    }

    pub fn disassemble_instruction(byte_code: &ByteCode, opcode: OpCode, offset: &mut usize) {
        print!("{:04} [{opcode:?}] ", *offset);

        match opcode {
            OpCode::Push => {
                let bytes: [u8; 8] = byte_code.bytes[(*offset + 1)..=(*offset + 8)]
                    .try_into()
                    .unwrap();
                let constant_index = usize::from_ne_bytes(bytes);
                let constant = byte_code.constants[constant_index].clone();
                println!("{constant_index:04} '{constant}'");
                *offset += 9;
            }
            OpCode::Halt => {
                println!();
                *offset += 1;
            }
        }
    }
}
