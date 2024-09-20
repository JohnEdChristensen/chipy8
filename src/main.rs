use chipy8::rom::Rom;
use chipy8::widget::HexInput;
use chipy8::{chip8::Chip8, cli::Cli};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::*,
    widgets::{canvas::Canvas, BarChart, Block, List, Paragraph},
    DefaultTerminal,
};

use std::error::Error;
use std::{
    cmp::Ordering,
    path::PathBuf,
    time::{Duration, Instant},
};
use strum::Display;
use symbols::Marker;

fn main() -> Result<(), Box<dyn Error>> {
    //// Setup

    let cli = Cli::parse();

    let mut terminal = ratatui::init();

    let app = App::new(cli.rom_path, cli.paused);

    // Clean the slate
    terminal.clear()?;
    //// Start!
    let app_result = app.run(terminal);

    //// Cleanup
    ratatui::restore();
    app_result
}

struct App {
    chip8: Chip8,
    tick_count: u64,
    mode: Mode,
}

#[derive(Clone, Copy, Debug, Display)]
enum Mode {
    Running,
    Paused,
}

impl App {
    fn new(path: PathBuf, paused: bool) -> Self {
        let initial_mode = match paused {
            true => Mode::Paused,
            false => Mode::Running,
        };
        let rom = Rom::new(path).unwrap();
        Self {
            chip8: Chip8::new(rom),
            tick_count: 0,
            mode: initial_mode,
        }
    }
    fn toggle_mode(mut self) -> Self {
        self.mode = match self.mode {
            Mode::Running => Mode::Paused,
            Mode::Paused => Mode::Running,
        };
        self
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn Error>> {
        let tick_rate = Duration::from_millis(4);
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => break Ok(()),
                        KeyCode::Char(' ') => self = self.toggle_mode(),
                        KeyCode::Char('1') => self.chip8.input = 0,
                        KeyCode::Char('2') => self.chip8.input = 1,
                        KeyCode::Char('3') => self.chip8.input = 2,
                        KeyCode::Char('4') => self.chip8.input = 3,
                        KeyCode::Char('q') => self.chip8.input = 4,
                        KeyCode::Char('w') => self.chip8.input = 5,
                        KeyCode::Char('e') => self.chip8.input = 6,
                        KeyCode::Char('r') => self.chip8.input = 7,
                        KeyCode::Char('a') => self.chip8.input = 8,
                        KeyCode::Char('s') => self.chip8.input = 9,
                        KeyCode::Char('d') => self.chip8.input = 10,
                        KeyCode::Char('f') => self.chip8.input = 11,
                        KeyCode::Char('z') => self.chip8.input = 12,
                        KeyCode::Char('x') => self.chip8.input = 13,
                        KeyCode::Char('c') => self.chip8.input = 14,
                        KeyCode::Char('v') => self.chip8.input = 15,
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                self.on_tick();
                last_tick = Instant::now();
            }
        }
    }

    fn on_tick(&mut self) {
        self.tick_count += 1;
        match self.mode {
            Mode::Running => {
                self.chip8.step();
            }
            _ => (),
        };
    }

    fn draw(&self, frame: &mut Frame) {
        let horizontal = Layout::horizontal([Constraint::Length(66), Constraint::Min(1)]);
        let [left, right] = horizontal.areas(frame.area());

        let left_vertical = Layout::vertical([Constraint::Length(18), Constraint::Min(6)]);
        let [display, n3] = left_vertical.areas(left);
        frame.render_widget(self.display(), display);
        frame.render_widget(Paragraph::new("n3").block(Block::bordered()), n3);

        let right_vertical = Layout::vertical([Constraint::Min(1), Constraint::Length(7)]);
        let [n1, n2] = right_vertical.areas(right);

        self.render_registers(n3, frame);
        self.render_program(n1, frame);
        frame.render_widget(
            HexInput::new(self.chip8.input).block(Block::bordered().title("Input")),
            n2,
        );
    }
    fn render_program(&self, area: Rect, frame: &mut Frame) {
        let outer_block = Block::bordered().title("Program");
        let inner = outer_block.inner(area);
        frame.render_widget(outer_block, area);

        let memory = self.chip8.memory;
        let pc = self.chip8.program_counter as usize;
        let display_range = pc - 4..pc + 28;
        let program_display = &memory[display_range];
        let lines: Vec<Line> = program_display
            .chunks(2)
            .into_iter()
            .enumerate()
            .map(|(i, b)| style_instruction(pc, i * 2 + (pc - 4), b[0], b[1]))
            .collect();

        let list = List::new(lines);
        frame.render_widget(list, inner);
    }

    fn render_registers(&self, area: Rect, frame: &mut Frame) {
        let outer_block = Block::bordered().title("Registers");
        let content = outer_block.inner(area);
        frame.render_widget(outer_block, area);
        let register_layout = Layout::vertical([Constraint::Length(4), Constraint::Length(1)]);
        let [main_reg, misc_reg] = register_layout.areas(content);

        let labels: Vec<String> = (0..16)
            .collect::<Vec<i32>>()
            .into_iter()
            .map(|i| "v".to_owned() + &format!("{i:1x}").to_uppercase())
            .collect();

        let data: Vec<(&str, u64)> = labels
            .iter()
            .zip(self.chip8.registers)
            .map(|(l, i)| (l.as_str(), i as u64))
            .collect();

        let bar_columns = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);
        let bar_areas: [Rect; 4] = bar_columns.areas(main_reg);
        let _ = &data
            .chunks(4)
            .into_iter()
            .zip(bar_areas)
            .for_each(|(f, a)| {
                frame.render_widget(
                    BarChart::default()
                        .bar_gap(0)
                        .bar_width(1)
                        .bar_style(Style::new().green())
                        .value_style(Style::new().black().on_green())
                        .data(f)
                        .max(255)
                        .direction(Direction::Horizontal),
                    a,
                );
            });
        let bar_columns = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ]);
        //frame.render_
        let bar_areas: [Rect; 3] = bar_columns.areas(misc_reg);
        let _ = &[
            ("delay", self.chip8.delay as u64),
            ("sound", self.chip8.sound as u64),
            ("i", self.chip8.i as u64),
        ]
        .into_iter()
        .zip(bar_areas)
        .for_each(|(f, a)| {
            frame.render_widget(
                BarChart::default()
                    .bar_gap(0)
                    .bar_width(1)
                    .bar_style(Style::new().blue())
                    .value_style(Style::new().black().on_blue())
                    .data(&[f])
                    .max(2000)
                    .direction(Direction::Horizontal),
                a,
            );
        });
    }

    fn display(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(
                Block::bordered()
                    .title(self.chip8.rom.name())
                    .title(self.mode.to_string()),
            )
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                ctx.draw(&self.chip8);
            })
    }
}
fn style_instruction<'a>(pc: usize, addr: usize, b1: u8, b2: u8) -> Line<'a> {
    let line_count = Span::from(format!("{addr:#4x}  ")).dim();

    let instruction = Span::from(format!("{b1:2x} {b2:2x}"));
    let (line_count, instruction) = match addr.cmp(&pc) {
        Ordering::Less => (line_count.dim(), instruction.dim()),
        Ordering::Equal => (line_count.green(), instruction.green()),
        Ordering::Greater => (line_count.dim(), instruction),
    };
    Line::from(vec![line_count, instruction])
}
