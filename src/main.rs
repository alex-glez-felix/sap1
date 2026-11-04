use ratatui::{
    crossterm::event::{self, Event},
    widgets::{Block, Paragraph},
};
use sap1::{RAM_SIZE, address::Address, cpu::Cpu, instruction::Instruction};

fn main() -> std::io::Result<()> {
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

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, cpu);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, mut cpu: Cpu) -> std::io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let text = match cpu.current_instruction() {
                Some(instruction) => instruction.to_string(),
                None => "???".to_string(),
            };

            let texto = format!(
                "{0:<6}   PC={1:>2}  A={2:#010b} {2:#04x}  OUT={3:#010b} {3:#04x}",
                text,
                cpu.pc().get(),
                cpu.a(),
                cpu.out(),
            );

            let widget = Paragraph::new(texto).block(Block::bordered().title("Registros"));
            frame.render_widget(widget, frame.area());
        })?;

        let e = event::read()?;

        if let Event::Key(key) = e {
            match key.code {
                event::KeyCode::Char(' ') => cpu.step(),
                event::KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
        }
    }
}
