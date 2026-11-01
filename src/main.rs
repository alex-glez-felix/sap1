type Address = u8;
type OpCode = u8;
type Word = u8;

const RAM_SIZE: usize = 16;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Instruction {
    Lda(Address),
    Add(Address),
    Sub(Address),
    Out,
    Hlt,
}

impl Instruction {
    #[expect(clippy::identity_op, reason = "simetría con los demás opcodes")]
    fn encode(self) -> Word {
        match self {
            Instruction::Lda(operand) => 0x00 | operand,
            Instruction::Add(operand) => 0x10 | operand,
            Instruction::Sub(operand) => 0x20 | operand,
            Instruction::Out => 0xE0,
            Instruction::Hlt => 0xF0,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Default)]
struct Cpu {
    pc: Address,
    a: Word,
    out: Word,
    ram: [Word; RAM_SIZE],
    halted: bool,
}

impl Cpu {
    fn new(program: [Word; RAM_SIZE]) -> Self {
        Self {
            pc: 0x0,
            a: 0x00,
            out: 0x00,
            ram: program,
            halted: false,
        }
    }

    fn step(&mut self) {
        if self.halted {
            return;
        }

        let word = self.ram[self.pc as usize];
        self.pc = (self.pc + 1) & 0x0F;

        let Some(instruction) = decode(word) else {
            self.halted = true;
            return;
        };

        match instruction {
            Instruction::Lda(operand) => self.a = self.ram[operand as usize],
            Instruction::Add(operand) => self.a = self.a.wrapping_add(self.ram[operand as usize]),
            Instruction::Sub(operand) => self.a = self.a.wrapping_sub(self.ram[operand as usize]),
            Instruction::Out => self.out = self.a,
            Instruction::Hlt => self.halted = true,
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    type InstructionConstructor = fn(Address) -> Instruction;

    fn cpu_with(instruction: InstructionConstructor, address: Address, value: Word) -> Cpu {
        let mut program = [0x00; RAM_SIZE];
        program[0] = instruction(address).encode();
        program[address as usize] = value;
        Cpu::new(program)
    }

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
                assert_eq!(decode(word), Some(expected(operand)), "word = {word:#04x}")
            }
        }
    }

    #[test]
    fn new_cpu_starts_in_reset_state() {
        let program = [0xF1; RAM_SIZE];
        let cpu = Cpu::new(program);
        assert_eq!(cpu.pc, 0x0);
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.out, 0x00);
        assert!(!cpu.halted);
        assert_eq!(cpu.ram, program);
    }

    #[test]
    fn step_does_nothing_when_halted() {
        let mut cpu = Cpu::default();
        cpu.halted = true;
        let before = cpu.clone();
        cpu.step();
        assert_eq!(cpu, before);
    }

    #[test]
    fn step_advances_pc() {
        let mut cpu = Cpu::default();
        cpu.step();
        assert_eq!(cpu.pc, 1);
    }

    #[test]
    fn pc_wraps_after_last_address() {
        let mut cpu = Cpu::default();
        cpu.pc = 15;
        cpu.step();
        assert_eq!(cpu.pc, 0);
    }

    #[test]
    fn lda_loads_value_from_memory() {
        let value = 0xA1;

        let mut cpu = cpu_with(Instruction::Lda, 14, value);
        cpu.step();
        assert_eq!(cpu.a, value);
    }

    #[test]
    fn add_adds_memory_to_a() {
        let value = 12;
        let mut cpu = cpu_with(Instruction::Add, 10, value);

        cpu.a = 88;
        cpu.step();
        assert_eq!(cpu.a, 88 + value);
    }

    #[test]
    fn sub_subtracts_memory_from_a() {
        let value = 15;
        let mut cpu = cpu_with(Instruction::Sub, 5, value);
        cpu.a = 120;
        cpu.step();
        assert_eq!(cpu.a, 120 - value);
    }

    #[test]
    fn out_copies_a_to_output() {
        let mut program = [0x00; RAM_SIZE];
        program[0] = Instruction::Out.encode();

        let mut cpu = Cpu::new(program);
        cpu.a = 34;
        cpu.step();
        assert_eq!(cpu.out, 34);
    }

    #[test]
    fn hlt_halts_cpu() {
        let mut program = [0x00; RAM_SIZE];
        program[0] = Instruction::Hlt.encode();

        let mut cpu = Cpu::new(program);
        cpu.step();
        assert!(cpu.halted);
    }

    #[test]
    fn add_wraps_on_overflow() {
        let value = 250;
        let mut cpu = cpu_with(Instruction::Add, 9, value);

        cpu.a = 10;
        cpu.step();
        // 10 + 250 = 260 -> 260 - 256 = 4
        assert_eq!(cpu.a, 4);
    }

    #[test]
    fn sub_wraps_on_underflow() {
        let value = 3;
        let mut cpu = cpu_with(Instruction::Sub, 9, value);

        cpu.a = 2;
        cpu.step();
        // 2 - 3 = -1 -> -1 + 256 = 255
        assert_eq!(cpu.a, 255);
    }

    #[test]
    fn invalid_opcode_halts() {
        let mut program = [0x00; RAM_SIZE];
        program[0] = 0xC0;

        let mut cpu = Cpu::new(program);
        cpu.step();

        assert!(cpu.halted);
    }

    #[test]
    fn runs_sample_program() {
        let mut program = [0x00; RAM_SIZE];

        program[0] = Instruction::Lda(8).encode();
        program[8] = 100;
        program[1] = Instruction::Add(9).encode();
        program[9] = 7;
        program[2] = Instruction::Sub(10).encode();
        program[10] = 58;
        program[3] = Instruction::Out.encode();
        program[4] = Instruction::Add(11).encode();
        program[11] = 1;
        program[5] = Instruction::Hlt.encode();
        let mut cpu = Cpu::new(program);

        cpu.step();
        cpu.step();
        cpu.step();
        cpu.step();
        cpu.step();
        cpu.step();

        assert!(cpu.halted);
        assert_eq!(cpu.out, 49);
        assert_eq!(cpu.a, 50);
    }
}
