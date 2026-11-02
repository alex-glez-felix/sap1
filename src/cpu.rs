use crate::{
    RAM_SIZE, Word,
    address::Address,
    instruction::{Instruction, decode},
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cpu {
    pc: Address,
    a: Word,
    out: Word,
    ram: [Word; RAM_SIZE],
    halted: bool,
}

#[derive(Debug, PartialEq)]
pub enum RunOutcome {
    Halted,
    StepLimitReached,
}

impl Cpu {
    pub fn new(program: [Word; RAM_SIZE]) -> Self {
        Self {
            pc: Address::from_word(0x00),
            a: 0x00,
            out: 0x00,
            ram: program,
            halted: false,
        }
    }

    pub fn current_instruction(&self) -> Option<Instruction> {
        decode(self.ram[self.pc.index()])
    }

    pub fn a(&self) -> Word {
        self.a
    }

    pub fn pc(&self) -> Address {
        self.pc
    }

    pub fn out(&self) -> Word {
        self.out
    }

    pub fn halted(&self) -> bool {
        self.halted
    }

    pub fn ram(&self) -> &[Word; RAM_SIZE] {
        &self.ram
    }

    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        let word = self.ram[self.pc.index()];
        self.pc = self.pc.next();

        let Some(instruction) = decode(word) else {
            self.halted = true;
            return;
        };

        match instruction {
            Instruction::Lda(operand) => self.a = self.ram[operand.index()],
            Instruction::Add(operand) => self.a = self.a.wrapping_add(self.ram[operand.index()]),
            Instruction::Sub(operand) => self.a = self.a.wrapping_sub(self.ram[operand.index()]),
            Instruction::Out => self.out = self.a,
            Instruction::Hlt => self.halted = true,
        }
    }

    pub fn run_with(
        &mut self,
        max_steps: usize,
        mut on_step: impl FnMut(&Cpu, Option<Instruction>),
    ) -> RunOutcome {
        for _ in 0..max_steps {
            if self.halted {
                break;
            }
            let instruction = self.current_instruction();
            self.step();
            on_step(self, instruction);
        }

        if self.halted {
            return RunOutcome::Halted;
        }
        RunOutcome::StepLimitReached
    }

    pub fn run(&mut self, max_steps: usize) -> RunOutcome {
        self.run_with(max_steps, |_, _| {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type InstructionConstructor = fn(Address) -> Instruction;

    fn cpu_with(instruction: InstructionConstructor, index: u8, value: Word) -> Cpu {
        let mut program = [0x00; RAM_SIZE];
        let address = Address::new(index).unwrap();
        program[0] = instruction(address).encode();
        program[address.index()] = value;
        Cpu::new(program)
    }

    #[test]
    fn new_cpu_starts_in_reset_state() {
        let program = [0xF1; RAM_SIZE];
        let cpu = Cpu::new(program);
        assert_eq!(cpu.pc, Address::new(0).unwrap());
        assert_eq!(cpu.a, 0x00);
        assert_eq!(cpu.out, 0x00);
        assert!(!cpu.halted);
        assert_eq!(cpu.ram, program);
    }

    #[test]
    fn step_does_nothing_when_halted() {
        let mut cpu = Cpu {
            halted: true,
            ..Default::default()
        };
        let before = cpu.clone();
        cpu.step();
        assert_eq!(cpu, before);
    }

    #[test]
    fn step_advances_pc() {
        let mut cpu = Cpu::default();
        cpu.step();
        assert_eq!(cpu.pc.get(), 1);
    }

    #[test]
    fn pc_wraps_after_last_address() {
        let mut cpu = Cpu {
            pc: Address::new(15).unwrap(),
            ..Default::default()
        };
        cpu.step();
        assert_eq!(cpu.pc.get(), 0);
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

        program[0] = Instruction::Lda(Address::new(8).unwrap()).encode();
        program[8] = 100;
        program[1] = Instruction::Add(Address::new(9).unwrap()).encode();
        program[9] = 7;
        program[2] = Instruction::Sub(Address::new(10).unwrap()).encode();
        program[10] = 58;
        program[3] = Instruction::Out.encode();
        program[4] = Instruction::Add(Address::new(11).unwrap()).encode();
        program[11] = 1;
        program[5] = Instruction::Hlt.encode();
        let mut cpu = Cpu::new(program);

        let outcome = cpu.run(10);

        assert!(cpu.halted);
        assert_eq!(cpu.out, 49);
        assert_eq!(cpu.a, 50);
        assert_eq!(outcome, RunOutcome::Halted);
    }

    #[test]
    fn run_stops_after_step_max() {
        let mut cpu = Cpu::default();
        let outcome = cpu.run(9);

        assert_eq!(cpu.pc, Address::new(9).unwrap());
        assert_eq!(outcome, RunOutcome::StepLimitReached);
    }

    #[test]
    fn run_halts_when_hlt_is_on_last_allowed_step() {
        let mut program = [0x00; RAM_SIZE];
        program[2] = Instruction::Hlt.encode();
        let mut cpu = Cpu::new(program);

        let outcome = cpu.run(3);

        assert!(cpu.halted);
        assert_eq!(outcome, RunOutcome::Halted);
    }

    #[test]
    fn run_hits_limit_one_step_before_hlt() {
        let mut program = [0x00; RAM_SIZE];
        program[2] = Instruction::Hlt.encode();
        let mut cpu = Cpu::new(program);

        let outcome = cpu.run(2);

        assert!(!cpu.halted);
        assert_eq!(outcome, RunOutcome::StepLimitReached);
    }
}
