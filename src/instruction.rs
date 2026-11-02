use crate::{Word, address::Address};
use std::fmt::{self};

type OpCode = u8;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    Lda(Address),
    Add(Address),
    Sub(Address),
    Out,
    Hlt,
}

impl Instruction {
    #[expect(clippy::identity_op, reason = "simetría con los demás opcodes")]
    pub fn encode(self) -> Word {
        match self {
            Instruction::Lda(operand) => 0x00 | operand.get(),
            Instruction::Add(operand) => 0x10 | operand.get(),
            Instruction::Sub(operand) => 0x20 | operand.get(),
            Instruction::Out => 0xE0,
            Instruction::Hlt => 0xF0,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Instruction::Lda(address) => write!(f, "LDA {}", address.get()),
            Instruction::Add(address) => write!(f, "ADD {}", address.get()),
            Instruction::Sub(address) => write!(f, "SUB {}", address.get()),
            Instruction::Out => write!(f, "OUT"),
            Instruction::Hlt => write!(f, "HLT"),
        }
    }
}

fn opcode(word: Word) -> OpCode {
    word >> 4
}

fn operand(word: Word) -> Address {
    Address::from_word(word)
}

pub fn decode(word: Word) -> Option<Instruction> {
    let opcode = opcode(word);
    let operand = operand(word);

    match opcode {
        0x00 => Some(Instruction::Lda(operand)),
        0x01 => Some(Instruction::Add(operand)),
        0x02 => Some(Instruction::Sub(operand)),
        0x0E => Some(Instruction::Out),
        0x0F => Some(Instruction::Hlt),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type InstructionConstructor = fn(Address) -> Instruction;

    #[test]
    fn decodes_each_opcode() {
        let cases = [
            (0x09, Instruction::Lda(Address::new(9).unwrap())),
            (0x1f, Instruction::Add(Address::new(15).unwrap())),
            (0x22, Instruction::Sub(Address::new(2).unwrap())),
            (0xEF, Instruction::Out),
            (0xF0, Instruction::Hlt),
        ];

        for (word, expected) in cases {
            assert_eq!(decode(word), Some(expected), "word = {word:#04x}")
        }
    }

    #[test]
    fn encodes_decodes_symmetry() {
        for word in 0..=0xFF {
            if let Some(instruction) = decode(word) {
                assert_eq!(
                    decode(instruction.encode()),
                    Some(instruction),
                    "word = {word:#04x}"
                )
            }
        }
    }

    #[test]
    fn unknown_opcode_is_none() {
        assert_eq!(decode(0xA0), None)
    }

    #[test]
    fn operandless_instructions_ignore_low_nibble() {
        let instructions = [(0xE0, Instruction::Out), (0xF0, Instruction::Hlt)];

        for (opcode, expected) in instructions {
            for operand in 0..=0x0F {
                let word = opcode | operand;
                assert_eq!(decode(word), Some(expected), "word = {word:#04x}")
            }
        }
    }

    #[test]
    fn operand_instructions_take_low_nibble() {
        let instructions: [(u8, InstructionConstructor); 3] = [
            (0x00, Instruction::Lda),
            (0x10, Instruction::Add),
            (0x20, Instruction::Sub),
        ];

        for (opcode, expected) in instructions {
            for operand in 0..=0x0F {
                let word = opcode | operand;
                assert_eq!(
                    decode(word),
                    Some(expected(Address::from_word(operand))),
                    "word = {word:#04x}"
                )
            }
        }
    }

    #[test]
    fn instruction_display_with_name() {
        let instructions = [
            (Instruction::Lda(Address::new(8).unwrap()), "LDA 8"),
            (Instruction::Add(Address::new(15).unwrap()), "ADD 15"),
            (Instruction::Sub(Address::new(1).unwrap()), "SUB 1"),
            (Instruction::Out, "OUT"),
            (Instruction::Hlt, "HLT"),
        ];

        for (instruction, expected) in instructions {
            assert_eq!(instruction.to_string(), expected, "{expected}")
        }
    }
}
