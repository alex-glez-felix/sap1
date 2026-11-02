use sap1::{RAM_SIZE, address::Address, cpu::Cpu, instruction::Instruction};

fn main() {
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

    let outcome = cpu.run_with(100, |cpu, instruction| {
        let text = match instruction {
            Some(i) => i.to_string(),
            None => "???".to_string(),
        };

        println!(
            "{0:<6}   PC={1:>2}  A={2:#010b} {2:#04x}  OUT={3:#010b} {3:#04x}",
            text,
            cpu.pc().get(),
            cpu.a(),
            cpu.out(),
        )
    });

    println!("{outcome:?}");
}
