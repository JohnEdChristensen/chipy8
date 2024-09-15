use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct HexInput {
    pub input: u8,
}
//// Displays all 16 possible input keys, 0..F
/// The selected input is highlighted
impl Widget for HexInput {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let keys = (0..15).fold("".to_owned(), |acc, x| format!("{acc}{:#1x}", x));
        let keys = [0..15].chunks(4).for_each(|f| );

        buf.set_string(
            area.left(),
            area.top(),
            keys,
            Style::default().fg(Color::Green),
        );
        buf.set_string(
            area.left(),
            area.top() + 1,
            self.input.to_string(),
            Style::default().fg(Color::Green),
        );
    }
}
