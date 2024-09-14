//! # [Ratatui] Canvas example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

use std::{
    env::args,
    time::{Duration, Instant},
};

use chip8::{Chip8, WIDTH_PIX};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Shape},
        Block, Borders, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};
mod chip8;
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
        let path = cli_params.next().unwrap();
        Self {
            chip8: Chip8::new(path),
            tick_count: 0,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(16);
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break Ok(()),
                        //KeyCode::Down | KeyCode::Char('j') => self.y += 1.0,
                        //KeyCode::Up | KeyCode::Char('k') => self.y -= 1.0,
                        //KeyCode::Right | KeyCode::Char('l') => self.x += 1.0,
                        //KeyCode::Left | KeyCode::Char('h') => self.x -= 1.0,
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
        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let vertical = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [display, right] = horizontal.areas(frame.area());
        let [n1, n2] = vertical.areas(right);

        let surrounding_block = Block::default().borders(Borders::ALL).title("data");
        frame.render_widget(self.display(), display);
        frame.render_widget(Paragraph::new("n1").block(surrounding_block.clone()), n1);
        frame.render_widget(Paragraph::new("n2").block(surrounding_block), n2);
    }

    fn display(&self) -> impl Widget + '_ {
        Canvas::default()
            .block(Block::bordered().title("Rects"))
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
        //painter.paint(0, 0, Color::Green);
        //painter.paint(31, 0, Color::Green);
        //painter.paint(31, 1, Color::Green);
        //
        //painter.paint(63, 0, Color::Green);
        //painter.paint(0, 31, Color::Green);
        //painter.paint(63, 31, Color::Green);
    }
}
