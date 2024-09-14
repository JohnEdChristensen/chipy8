#![allow(arithmetic_overflow)]
use std::{fmt, fs::read};

use drawille::Canvas;
/// The first 512 bytes are resevered for the interpreter
const PROGRAM_START: usize = 0x200;
const MEMORY_SIZE: usize = 4096;

pub const WIDTH_PIX: usize = 64;
pub const HEIGHT_PIX: usize = 32;
const WIDTH_BYTE: usize = 8;
const HEIGHT_BYTE: usize = 32;

#[derive(Clone, PartialEq)]
pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    registers: [u8; 16],
    /// register for storing memory addresses
    i: u16,
    input: u8,
    /// these two registers are auto decremented at 60hz
    delay: u8,
    sound: u8,

    program_counter: u16,
    /// the stack stores the address that should be returned to
    stack: [u16; 16],
    stack_pointer: u8,

    pub display: [u8; WIDTH_BYTE * HEIGHT_BYTE],
    canvas: Canvas,
}

impl Chip8 {
    pub fn new(path: String) -> Chip8 {
        let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
        println!("Reading ROM: {}", path);

        let rom = read(path).unwrap();
        let rom: &[u8] = rom.as_slice();
        memory[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(rom);

        #[rustfmt::skip]
        let characters = [
            //0
            0xF0, 0x90, 0x90, 0x90, 0xF0,
            //1
            0x20, 0x60, 0x20, 0x20, 0x70,
            //2
            0xF0, 0x10, 0xF0, 0x80, 0xF0,
            //3
            0xF0, 0x10, 0xF0, 0x10, 0xF0,
            //4
            0x90, 0x90, 0xF0, 0x10, 0x10,
            //5
            0xF0, 0x80, 0xF0, 0x10, 0xF0,
            //6
            0xF0, 0x80, 0xF0, 0x90, 0xF0,
            //7
            0xF0, 0x10, 0x20, 0x40, 0x40,
            //8
            0xF0, 0x90, 0xF0, 0x90, 0xF0,
            //9
            0xF0, 0x90, 0xF0, 0x10, 0xF0,
            //A
            0xF0, 0x90, 0xF0, 0x90, 0x90,
            //B
            0xE0, 0x90, 0xE0, 0x90, 0xE0,
            //C
            0xF0, 0x80, 0x80, 0x80, 0xF0,
            //D
            0xE0, 0x90, 0x90, 0x90, 0xE0,
            //E
            0xF0, 0x80, 0xF0, 0x80, 0xF0,
            //F
            0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];
        memory[0..characters.len()].copy_from_slice(&characters);

        Chip8 {
            memory,
            registers: [0; 16],
            i: 0,
            input: 0,
            delay: 0,
            sound: 0,
            program_counter: PROGRAM_START as u16,
            stack: [0; 16],
            stack_pointer: 0,
            display: [0; WIDTH_BYTE * HEIGHT_BYTE],
            canvas: Canvas::new(WIDTH_PIX as u32, HEIGHT_PIX as u32),
        }
    }
    fn set_addr(&mut self, a1: u8, a2: u8, a3: u8) {
        self.program_counter = assemble_addr(a1, a2, a3) - 2;
    }
    pub fn step(&mut self) -> &mut Chip8 {
        let byte_1 = self.memory[self.program_counter as usize];
        let n1 = (byte_1 & 0xF0) >> 4;
        let n2 = byte_1 & 0x0F;
        let byte_2 = self.memory[(self.program_counter + 1) as usize];
        let n3 = (byte_2 & 0xF0) >> 4;
        let n4 = byte_2 & 0x0F;
        //println!("{:#x},{:#x}", byte_1, byte_2);
        //println!("{:#x}, {:#x}, {:#x}, {:#x}", n1, n2, n3, n4);
        //println!("{:?}", state);
        match (n1, n2, n3, n4) {
            //// CLS
            (0, 0, 0x0E, 0x00) => self.display.fill(0),
            //// RET
            (0, 0, 0x0E, 0x0E) => {
                // pop sp
                self.program_counter = self.stack[self.stack_pointer as usize];
                self.stack_pointer -= 1;
            }
            //// JP addr
            (0x01, a1, a2, a3) => self.set_addr(a1, a2, a3),
            //// CALL addr
            (0x02, a1, a2, a3) => {
                // push sp
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.set_addr(a1, a2, a3);
            }
            //// Eq Vx, NN
            (0x03, x, n1, n2) => {
                if self.registers[x as usize] == (n1 << 4 | n2) {
                    self.program_counter += 2;
                }
            }
            //// Neq Vx,NN
            (0x04, x, n1, n2) => {
                if self.registers[x as usize] != (n1 << 4 | n2) {
                    self.program_counter += 2;
                }
            }
            //// Eq Vx, Vy
            (0x05, x, y, _) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.program_counter += 2;
                }
            }
            //// Set Vx=NN
            (0x06, x, n1, n2) => self.registers[x as usize] = n1 << 4 | n2,
            (0x07, x, n1, n2) => self.registers[x as usize] += n1 << 4 | n2,
            (0x08, x, y, 0) => self.registers[x as usize] += self.registers[y as usize],
            (0x08, x, y, 1) => self.registers[x as usize] |= self.registers[y as usize],
            (0x08, x, y, 2) => self.registers[x as usize] &= self.registers[y as usize],
            (0x08, x, y, 3) => self.registers[x as usize] ^= self.registers[y as usize],
            (0x08, x, y, 4) => {
                let (value, overflow) =
                    self.registers[x as usize].overflowing_add(self.registers[y as usize]);
                self.registers[x as usize] = value;
                self.registers[15] = overflow as u8;
            }
            (0x08, x, y, 5) => {
                let (value, overflow) =
                    self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
                self.registers[x as usize] = value;
                self.registers[15] = (!overflow) as u8;
            }
            (0x08, x, _, 6) => {
                self.registers[15] = self.registers[x as usize] | 1;
                self.registers[x as usize] >>= 1
            }
            (0x08, x, y, 7) => {
                let (value, overflow) =
                    self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
                self.registers[x as usize] = value;
                self.registers[15] = (!overflow) as u8;
            }
            (0x08, x, _, 0xE) => {
                self.registers[15] = self.registers[x as usize] | 0x8;
                self.registers[x as usize] <<= 1
            }
            (0x09, x, y, 0) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.program_counter += 2;
                }
            }
            (0x0A, n1, n2, n3) => self.i = assemble_addr(n1, n2, n3),
            (0x0B, n1, n2, n3) => {
                self.program_counter = self.registers[0] as u16 + assemble_addr(n1, n2, n3)
            }
            (0x0C, x, n1, n2) => {
                self.registers[x as usize] = rand::random::<u8>() & ((n1 << 4) | n2)
            }
            //// Draw
            (0x0D, x, y, n) => {
                let mut changed = false;
                for i in 0..n {
                    let px = self.registers[x as usize];
                    let py = self.registers[y as usize];
                    let current_screen = &mut self.display[(px / 8 + ((py + i) * 8)) as usize];
                    let current_sprite = self.memory[self.i as usize + i as usize];
                    let fin = *current_screen ^ current_sprite;
                    changed |= (*current_screen & !fin) != 0;
                    *current_screen = fin;
                    let pixel_string = format!("{:08b}", fin);
                    pixel_string.chars().enumerate().for_each(|(j, p)| {
                        match p {
                            '0' => self
                                .canvas
                                .unset((j + (px) as usize) as u32, (py + i) as u32),
                            '1' => self.canvas.set((j + (px) as usize) as u32, (py + i) as u32),
                            e => panic!("unexpected char:{}", e),
                        };
                    });
                }
                self.registers[15] = changed as u8;
            }
            (0x0E, x, 9, 0x0E) => {
                if self.input == self.registers[x as usize] {
                    self.program_counter += 2;
                }
            }
            (0x0E, x, 0x0A, 1) => {
                if self.input != self.registers[x as usize] {
                    self.program_counter += 2;
                }
            }
            (0x0F, x, 0, 7) => self.registers[x as usize] = self.delay,
            (0x0F, x, 0, 0x0A) => self.registers[x as usize] = self.input,
            (0x0F, x, 1, 5) => self.delay = self.registers[x as usize],
            (0x0F, x, 1, 8) => self.sound = self.registers[x as usize],
            (0x0F, x, 1, 0x0E) => self.i += self.registers[x as usize] as u16,
            (0x0F, x, 2, 0x09) => self.i = x as u16 * 5,
            (0x0F, x, 3, 3) => {
                let val = self.registers[x as usize];
                self.memory[self.i as usize] = val / 100;
                self.memory[self.i as usize + 1] = (val % 100) / 10;
                self.memory[self.i as usize + 2] = val % 10;
            }
            (0x0F, x, 5, 5) => {
                for i in 0..=x {
                    self.memory[self.i as usize + i as usize] = self.registers[i as usize]
                }
            }
            (0x0F, x, 6, 5) => {
                for i in 0..=x {
                    self.registers[i as usize] = self.memory[self.i as usize + i as usize]
                }
            }

            _ => {
                println!(
                    " Unknown command: {:#x}, {:#x}, {:#x}, {:#x}",
                    n1, n2, n3, n4
                );
                todo!("Not all opcodes implemented!")
            }
        }
        //each instruction is 2 bytes
        self.program_counter += 2;
        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.sound > 0 {
            self.sound -= 1;
        }
        self
    }
}

fn assemble_addr(a1: u8, a2: u8, a3: u8) -> u16 {
    let a_low: u16 = ((a2 << 4) | a3) as u16;
    ((a1 as u16) << 8) | a_low
}

#[test]
fn cls() {
    let mut state = Chip8::new();
    let mut expected_state = state.clone();
    state.display.fill(1);

    assert_ne!(state, expected_state);
    state = step(state, 0x00, 0x00, 0x0E, 0x00);
    expected_state.program_counter += 2;

    assert_eq!(state, expected_state)
}
#[test]
fn ret() {
    let mut state = Chip8::new();
    state.stack_pointer = 3;
    state.stack[3] = 0xF0;
    state.stack[2] = 0xF1;
    state.stack[1] = 0xF2;
    state.stack[0] = 0xF3;
    let mut expected_state = state.clone();

    state = step(state, 0x00, 0x00, 0x0E, 0x0E);
    expected_state.stack_pointer = 2;
    expected_state.program_counter = 0xF0 + 2;

    assert_eq!(state, expected_state);

    state = step(state, 0x00, 0x00, 0x0E, 0x0E);
    expected_state.stack_pointer = 1;
    expected_state.program_counter = 0xF1 + 2;

    assert_eq!(state, expected_state);

    state = step(state, 0x00, 0x00, 0x0E, 0x0E);
    expected_state.stack_pointer = 0;
    expected_state.program_counter = 0xF2 + 2;

    assert_eq!(state, expected_state);

    state = step(state, 0x00, 0x00, 0x0E, 0x0E);
    expected_state.stack_pointer = 0xFF;
    expected_state.program_counter = 0xF3 + 2;

    assert_eq!(state, expected_state);
}

#[test]
fn jump() {
    let mut state = Chip8::new();
    let mut expected_state = state.clone();

    state = step(state, 0x01, 0x01, 0x02, 0x03);

    expected_state.program_counter = 0x0123 + 2;
    assert_eq!(state, expected_state);
    expected_state.program_counter = 0x0456 + 2;
    assert_ne!(state, expected_state);
}

// Implement Debug manually
impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display only a small part of memory for brevity
        let memory_preview = &self.memory[0..8]; // First 8 bytes of memory

        // Display registers as a simple array
        let registers_display = self
            .registers
            .iter()
            .map(|r| format!("{:#04x}", r))
            .collect::<Vec<_>>()
            .join(", ");

        let memory_pointer = &self.memory
            [self.program_counter as usize..self.program_counter as usize + 8]
            .iter()
            .map(|r| format!("{:#04x}", r))
            .collect::<Vec<_>>()
            .join(", ");

        // Display only a few entries from the stack
        let stack_preview = &self.stack[0..4]
            .iter()
            .map(|r| format!("{:#04x}", r))
            .collect::<Vec<_>>()
            .join(", ");

        //// Display part of the display array, e.g., a small block or first few pixels
        //let pixel_strings = &self
        //    .display
        //    .iter()
        //    .map(|r| format!("{:08b}", r))
        //    .collect::<Vec<_>>();

        //let display_preview: String =
        //    pixel_strings
        //        .into_iter()
        //        .enumerate()
        //        .fold("".to_owned(), |acc, (i, bit)| {
        //            if i % 16 == 0 {
        //                acc + "\n" + bit
        //            } else {
        //                acc + bit
        //            }
        //        });

        // Use `{:#?}` for debug formatting of arrays
        write!(
            f,
            "\x1B[2J\x1B[1;1H State {{
    Memory (first 8 bytes): {:?}
    Registers: [{}]
    I Register: {:#06x}
    Delay Timer: {}
    Sound Timer: {}
    Program Counter: {:#06x}
    Memory At Program Counter (next 8 bytes): {:?}
    Stack (first 4 entries): {:?}
    Stack Pointer: {:#x}
{}
}}",
            memory_preview,
            registers_display,
            self.i,
            self.delay,
            self.sound,
            self.program_counter,
            memory_pointer,
            stack_preview,
            self.stack_pointer,
            self.canvas.frame()
        )
    }
}
