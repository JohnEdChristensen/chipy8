use std::io::{Bytes, Read};

use chipy8::chip8::Chip8;
use chipy8::cli::Cli;
use chipy8::rom::Rom;
use clap::Parser;
use iced::widget::{canvas, column, container, image, text, Container};
use iced::Length::Fill;
use iced::{mouse, Center, Color, Rectangle, Renderer, Task, Theme};

pub fn main() -> iced::Result {
    let cli = Cli::parse();

    let rom = Rom::new(cli.rom_path).unwrap();
    iced::application("Chippy-8", Chippy8::update, Chippy8::view)
        .theme(|_| Theme::Ferra)
        .run_with(|| {
            (
                Chippy8 {
                    chip8: Chip8::new(rom),
                    mode: Mode::Running,
                },
                Task::done(Message::Tick),
            )
        })
}

struct Chippy8 {
    chip8: Chip8,
    mode: Mode,
}

enum Mode {
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ToggleMode,
    Tick,
}

impl Chippy8 {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleMode => {
                self.mode = match self.mode {
                    Mode::Running => Mode::Paused,
                    Mode::Paused => Mode::Running,
                };
                Task::none()
            }
            Message::Tick => {
                println!("{:?}", self.chip8);
                self.chip8.step();
                Task::done(Message::Tick)
            }
        }
    }

    fn view(&self) -> Container<Message> {
        container(
            column![
                text(self.chip8.rom.name()).size(50),
                canvas(Circle {
                    radius: 50.0,
                    chip8: &self.chip8
                })
            ]
            .padding(20)
            .align_x(Center),
        )
        .center_x(Fill)
        .center_y(Fill)
        .into()
    }
}

// First, we define the data we need for drawing
#[derive(Debug)]
struct Circle<'a> {
    radius: f32,
    chip8: &'a Chip8,
}

// Then, we implement the `Program` trait
impl<'a, Message> canvas::Program<Message> for Circle<'a> {
    // No internal state
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // We prepare a new `Frame`
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        // We create a `Path` representing a simple circle
        //let circle = canvas::Path::circle(frame.center(), self.radius);
        //let quad =
        //    canvas::Path::rectangle(iced::Point { x: 10., y: 20. }, iced::Size::new(64., 32.));
        //// canvas::Image::from(self.chip8.display);
        //// And fill it with some color
        //let bit_image = self.chip8.display;
        //let img = image::Handle::from_path("ferris.png");
        //let img_bytes = self.chip8.display.iter().flat_map(|p|[0xFF,])

        let pixel_string = &self
            .chip8
            .display
            .iter()
            .map(|r| format!("{:08b}", r))
            .collect::<Vec<_>>()
            .join("");
        let img_bits: Vec<u8> = pixel_string
            .chars()
            .into_iter()
            .flat_map(|c| match c {
                '0' => [0x00, 0x00, 0x00, 0xFF],
                '1' => [0xFF, 0xFF, 0xFF, 0xFF],
                _ => [0xFF, 0x00, 0x00, 0xFF],
            })
            .collect();
        println!("{}", img_bits.len());
        let img = image::Handle::from_rgba(64, 32, img_bits);
        frame.draw_image(
            Rectangle::new(iced::Point { x: 0., y: 0. }, iced::Size::new(64., 32.)),
            &img,
        );

        // Then, we produce the geometry
        vec![frame.into_geometry()]
    }
}

//// Finally, we simply use our `Circle` to create the `Canvas`!
//fn view<'a, Message: 'a>(_state: &'a State) -> Element<'a, Message> {
//    canvas(Circle { radius: 50.0 }).into()
//}
