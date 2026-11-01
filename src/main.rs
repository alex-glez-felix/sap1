type Address = u8;
type OpCode = u8;
type Word = u8;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Instruction {
    Lda(Address),
    Add(Address),
    Sub(Address),
    Out,
    Hlt,
}

fn opcode(word: Word) -> OpCode {
    word >> 4
}

fn operand(word: Word) -> Address {
    word & 0x0F
}

fn decode(word: Word) -> Option<Instruction> {
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

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    type Constructor = fn(Address) -> Instruction;

    #[test]
    fn decodes_each_opcode() {
        let cases = [
            (0x09, Instruction::Lda(9)),
            (0x1f, Instruction::Add(15)),
            (0x22, Instruction::Sub(2)),
            (0xEF, Instruction::Out),
            (0xF0, Instruction::Hlt),
        ];

        for (word, expected) in cases {
            assert_eq!(decode(word), Some(expected), "word = {word:#04x}")
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
        let instructions: [(u8, Constructor); 3] = [
            (0x00, Instruction::Lda),
            (0x10, Instruction::Add),
            (0x20, Instruction::Sub),
        ];

        for (opcode, expected) in instructions {
            for operand in 0..=0x0F {
                let word = opcode | operand;
                assert_eq!(decode(word), Some(expected(operand)), "word = {word:#04x}")
            }
        }
    }
}
