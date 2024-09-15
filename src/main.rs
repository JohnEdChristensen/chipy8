use std::{
    env::args,
    time::{Duration, Instant},
};

use chip8::{Chip8, WIDTH_PIX};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    prelude::*,
    widgets::{
        canvas::{Canvas, Shape},
        Block, Borders, Paragraph,
    },
    DefaultTerminal,
};
use symbols::Marker;
use widget::HexInput;

mod chip8;
mod widget;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

struct App {
    chip8: Chip8,
    tick_count: u64,
}

impl App {
    fn new() -> Self {
        let mut cli_params = args();
        let _ = cli_params.next();
        let path = cli_params.next().unwrap_or("./ROMS/INVADERS".to_string());
        Self {
            chip8: Chip8::new(&path),
            tick_count: 0,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(4);
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => break Ok(()),
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
        self.chip8.step();
    }

    fn draw(&self, frame: &mut Frame) {
        let horizontal = Layout::horizontal([Constraint::Length(66), Constraint::Min(1)]);
        let left_vertical = Layout::vertical([Constraint::Length(18), Constraint::Min(1)]);
        let right_vertical = Layout::vertical([Constraint::Min(1), Constraint::Min(1)]);
        let [left, right] = horizontal.areas(frame.area());
        let [n1, n2] = right_vertical.areas(right);
        let [display, n3] = left_vertical.areas(left);
        let surrounding_block = Block::default().borders(Borders::ALL).title("data");
        frame.render_widget(self.display(), display);
        frame.render_widget(
            HexInput {
                input: self.chip8.input,
            },
            n1,
        );
        frame.render_widget(Paragraph::new("n2").block(surrounding_block.clone()), n2);
        frame.render_widget(Paragraph::new("n3").block(surrounding_block), n3);
    }

    fn display(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("ROM"))
            .marker(Marker::HalfBlock)
            .paint(|ctx| {
                ctx.draw(&self.chip8);
            })
    }
}
impl Shape for Chip8 {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        let pixel_string = &self
            .display
            .iter()
            .map(|r| format!("{:08b}", r))
            .collect::<Vec<_>>()
            .join("");
        pixel_string
            .chars()
            .into_iter()
            .enumerate()
            .for_each(|(i, p)| match p {
                '1' => painter.paint(i % WIDTH_PIX, i / WIDTH_PIX, Color::White),
                '0' => painter.paint(i % WIDTH_PIX, i / WIDTH_PIX, Color::Black),
                _ => panic!("unexpected display value"),
            });
    }
}
