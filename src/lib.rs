use chip8::{Chip8, WIDTH_PIX};
use ratatui::{style::Color, widgets::canvas::Shape};

pub mod chip8;
pub mod cli;
pub mod rom;
pub mod widget;

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
                '0' => (), //painter.paint(i % WIDTH_PIX, i / WIDTH_PIX, Color::bg(self)),
                _ => panic!("unexpected display value"),
            });
    }
}
