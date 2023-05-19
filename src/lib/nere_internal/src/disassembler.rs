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
            let adjusted = offset - 8;
            Disassembler::disassemble_instruction_internal(
                byte_code,
                opcode,
                &mut offset,
                adjusted,
            );
        }
    }

    pub fn disassemble_instruction(byte_code: &ByteCode, opcode: OpCode, offset: &mut usize) {
        let adjusted = *offset;
        Disassembler::disassemble_instruction_internal(byte_code, opcode, offset, adjusted);
    }

    fn disassemble_instruction_internal(
        byte_code: &ByteCode,
        opcode: OpCode,
        offset: &mut usize,
        adjusted: usize,
    ) {
        print!("{:04} [{opcode:?}] ", adjusted);

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
            OpCode::If(..) => {
                let return_addr = Disassembler::read_isize(byte_code, offset);
                println!("{return_addr:04}");
                *offset += 9;
            }
            OpCode::Else(..) => {
                let return_addr = Disassembler::read_isize(byte_code, offset);
                println!("{return_addr:04}");
                *offset += 9;
            }
            OpCode::Lt
            | OpCode::Lte
            | OpCode::Gt
            | OpCode::Gte
            | OpCode::Eq
            | OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Dump
            | OpCode::Halt => {
                println!();
                *offset += 1;
            }
        }
    }

    fn read_isize(byte_code: &ByteCode, offset: &mut usize) -> isize {
        let bytes: [u8; 8] = byte_code.bytes[(*offset + 1)..=(*offset + 8)]
            .try_into()
            .unwrap();
        isize::from_ne_bytes(bytes)
    }
}
