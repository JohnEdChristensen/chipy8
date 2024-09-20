use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::BlockExt,
    style::{Color, Stylize},
    text::Span,
    widgets::{Block, Widget},
};

pub struct HexInput<'a> {
    pub input: u8,
    block: Option<Block<'a>>,
}
impl<'a> HexInput<'a> {
    pub fn new(input: u8) -> Self {
        HexInput { input, block: None }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}
//// Displays all 16 possible input keys, 0..F
/// The selected input is highlighted
impl Widget for HexInput<'_> {
    fn render(self, container_area: Rect, buf: &mut Buffer) {
        self.block.render(container_area, buf);
        let widget_area = self.block.inner_if_some(container_area);
        if widget_area.is_empty() {
            return;
        }

        let keys = "1234qwerasdfzxcv".chars();

        let spans = keys.enumerate().map(|(i, k)| {
            let span = Span::default().content(k.to_string());
            if i == self.input as usize {
                span.fg(Color::Green)
            } else {
                span.fg(Color::Blue)
            }
        });

        spans.enumerate().for_each(|(i, span)| {
            //.fold("".to_owned(), |acc, x| format!("{acc}{:#1x}", x));
            let x = widget_area.left() + (i as u16 % 4) * 3;
            let y = widget_area.top() + (i / 4) as u16;
            buf.set_span(x, y, &span, 8);
        });
    }
}
