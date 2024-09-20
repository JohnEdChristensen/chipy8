#![allow(arithmetic_overflow)]
use std::fmt;

use drawille::Canvas;

use crate::rom::Rom;
/// The first 512 bytes are resevered for the interpreter
const PROGRAM_START: usize = 0x200;
const MEMORY_SIZE: usize = 4096;

pub const WIDTH_PIX: usize = 64;
pub const HEIGHT_PIX: usize = 32;
const WIDTH_BYTE: usize = 8;
const HEIGHT_BYTE: usize = 32;

/// characters 0..f
/// 5 row tall, 8 pixles wide 
#[rustfmt::skip]
const CHARACTERS:[u8;5*16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //a
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //b
    0xF0, 0x80, 0x80, 0x80, 0xF0, //c
    0xE0, 0x90, 0x90, 0x90, 0xE0, //d
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //e
    0xF0, 0x80, 0xF0, 0x80, 0x80, //f
];

enum Register {
    Main(u8),
    ProgramCounter,
    Sprite,
    Delay,
    Sound,
}

/// Chip 8 emulator state
#[derive(Clone, PartialEq)]
pub struct Chip8 {
    pub memory: [u8; MEMORY_SIZE],
    pub registers: [u8; 16],
    /// register for storing memory addresses
    pub i: u16,
    pub input: u8,
    /// these two registers are auto decremented at 60hz
    pub delay: u8,
    pub sound: u8,

    pub program_counter: u16,
    /// the stack stores the address that should be returned to
    pub stack: [u16; 16],
    pub stack_pointer: u8,

    pub display: [u8; WIDTH_BYTE * HEIGHT_BYTE],
    pub rom: Rom,
    canvas: Canvas,
}

struct Instruction {
    n1: u8,
    n2: u8,
    n3: u8,
    n4: u8,
}

impl Instruction {
    fn exec(self, state: &mut Chip8) {}
    fn display(self, state: Chip8) {}
}
impl Chip8 {
    ///
    pub fn new(rom: Rom) -> Chip8 {
        let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];

        let rom_slice: &[u8] = rom.contents.as_slice();
        memory[PROGRAM_START..PROGRAM_START + rom_slice.len()].copy_from_slice(rom_slice);

        memory[0..CHARACTERS.len()].copy_from_slice(&CHARACTERS);

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
            rom,
            canvas: Canvas::new(WIDTH_PIX as u32, HEIGHT_PIX as u32),
        }
    }
    fn set_addr(&mut self, a1: u8, a2: u8, a3: u8) {
        self.program_counter = assemble_addr(a1, a2, a3) - 2;
    }

    #[allow(dead_code)]
    fn set_memory(&mut self, start_location: u16, data: Vec<u8>) {
        self.memory[start_location as usize..start_location as usize + data.len() as usize]
            .copy_from_slice(&data);
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
        //match (n1, n2, n3, n4) {
        //    (0, 0, 0xE, 0x0) => self.op_00e0(),
        //    (0, 0, 0xE, 0xE) => self.op_00ee(),
        //    (0x1, n1, n2, n3) => self.op_1nnn(n1, n2, n3),
        //    (0x2, n1, n2, n3) => self.op_2nnn(n1, n2, n3),
        //    (0x3, x, k1, k2) => self.op_3xkk(x, k1, k2),
        //    (0x4, x, k1, k2) => self.op_4xkk(x, k1, k2),
        //    (0x5, x, y, _) => self.op_5xy(x, y),
        //    (0x6, x, k1, k2) => self.op_6xkk(x, k1, k2),
        //    (0x7, x, k1, k2) => self.op_7xkk(x, k1, k2),
        //    (0x8, x, y, 0x0) => self.op_8xy0(x, y),
        //    (0x8, x, y, 0x1) => self.op_8xy1(x, y),
        //    (0x8, x, y, 0x2) => self.op_8xy2(x, y),
        //    (0x8, x, y, 0x3) => self.op_8xy3(x, y),
        //    (0x8, x, y, 0x4) => self.op_8xy4(x, y),
        //    (0x8, x, y, 0x5) => self.op_8xy5(x, y),
        //    (0x8, x, _, 0x6) => self.op_8xy6(x),
        //    (0x8, x, y, 0x7) => self.op_8xy7(x, y),
        //    (0x8, x, _, 0xE) => self.op_8xyE(x),
        //    (0x9, x, y, 0x0) => self.op_9xy0(x, y),
        //    (0xA, n1, n2, n3) => self.op_Annn(n1, n2, n3),
        //    (0xB, n1, n2, n3) => self.op_Bnnn(n1, n2, n3),
        //    (0xC, x, n1, n2) => self.op_Cxnn(x, n1, n2),
        //    (0xD, x, y, n) => self.op_Dxyn(x, y, n),
        //    (0xE, x, 0x9, 0xE) => self.opEx9E(x),
        //    (0xE, x, 0xA, 0x1) => self.opExA1(x),
        //    (0xF, x, 0x0, 0x7) => self.opFx07(x),
        //    (0xF, x, 0x0, 0xA) => self.opFx0A(x),
        //    (0xF, x, 0x1, 0x5) => self.opFx15(x),
        //    (0xF, x, 0x1, 0x8) => self.opFx18(x),
        //    (0xF, x, 0x1, 0xE) => self.opFx1E(x),
        //    (0xF, x, 0x2, 0x9) => self.opFx29(x),
        //    (0xF, x, 0x3, 0x3) => self.opFx33(x),
        //    (0xF, x, 0x5, 0x5) => self.opFx55(x),
        //    (0xF, x, 0x6, 0x5) => self.opFx65(x),
        //    _ => todo!(),
        //}
        //each instruction is 2 bytes
        self.program_counter += 2;
        if self.delay > 0 {
            self.delay -= 1;
        }
        if self.sound > 0 {
            self.sound -= 1;
        }
        //self.input = 0;
        self
    }

    fn op_00e0(&self) {
        todo!()
    }

    fn op_00ee(&self) {
        todo!()
    }

    fn op_1nnn(&self, n1: u8, n2: u8, n3: u8) {
        todo!()
    }

    fn op_2nnn(&self, n1: u8, n2: u8, n3: u8) {
        todo!()
    }

    fn op_3xkk(&self, x: u8, k1: u8, k2: u8) {
        todo!()
    }

    fn op_4xkk(&self, x: u8, k1: u8, k2: u8) {
        todo!()
    }

    fn op_5xy(&self, x: u8, y: u8) {
        todo!()
    }

    fn op_6xkk(&self, x: u8, k1: u8, k2: u8) {
        todo!()
    }

    fn op_7xkk(&self, x: u8, k1: u8, k2: u8) {
        todo!()
    }

    fn op_8xy0(&self, x: u8, y: u8) {
        todo!()
    }

    fn op_8xy1(&self, x: u8, y: u8) {
        todo!()
    }

    fn op_8xy2(&self, x: u8, y: u8) {
        todo!()
    }

    fn op_8xy3(&self, x: u8, y: u8) {
        todo!()
    }

    fn op_8xy4(&self, x: u8, y: u8) {
        todo!()
    }

    fn op_8xy5(&self, x: u8, y: u8) {
        todo!()
    }
    fn op_8xy6(&self, x: u8) {
        todo!()
    }

    fn op_8xy7(&self, x: u8, y: u8) {
        todo!()
    }

    fn op_8xyE(&self, x: u8) {
        todo!()
    }

    fn op_9xy0(&self, x: u8, y: u8) {
        todo!()
    }

    fn op_Annn(&self, n1: u8, n2: u8, n3: u8) {
        todo!()
    }

    fn op_Bnnn(&self, n1: u8, n2: u8, n3: u8) {
        todo!()
    }

    fn op_Cxnn(&self, x: u8, n1: u8, n2: u8) {
        todo!()
    }

    fn op_Dxyn(&self, x: u8, y: u8, n2: u8) {
        todo!()
    }

    fn opEx9E(&self, x: u8) {
        todo!()
    }

    fn opExA1(&self, x: u8) {
        todo!()
    }

    fn opFx07(&self, x: u8) {
        todo!()
    }

    fn opFx0A(&self, x: u8) {
        todo!()
    }

    fn opFx15(&self, x: u8) {
        todo!()
    }

    fn opFx18(&self, x: u8) {
        todo!()
    }

    fn opFx1E(&self, x: u8) {
        todo!()
    }

    fn opFx29(&self, x: u8) {
        todo!()
    }

    fn opFx33(&self, x: u8) {
        todo!()
    }

    fn opFx55(&self, x: u8) {
        todo!()
    }

    fn opFx65(&self, x: u8) {
        todo!()
    }
}

fn assemble_addr(a1: u8, a2: u8, a3: u8) -> u16 {
    let a_low: u16 = ((a2 << 4) | a3) as u16;
    ((a1 as u16) << 8) | a_low
}

#[test]
fn cls() {
    let mut state = Chip8::new("");
    let mut expected_state = state.clone();
    state.display.fill(1);

    assert_ne!(state, expected_state);
    state.set_memory(state.program_counter, vec![0x00, 0x00, 0x0E, 0x00]);
    state.step();
    expected_state.program_counter += 2;

    assert_eq!(state, expected_state)
}
#[test]
fn ret() {
    let mut state = Chip8::new("");
    state.stack_pointer = 3;
    state.stack[3] = 0xF0;
    state.stack[2] = 0xF1;
    state.stack[1] = 0xF2;
    state.stack[0] = 0xF3;
    #[rustfmt::skip]
    state.set_memory(
        state.program_counter,
        vec![
            0x00, 0x00, 0x0E, 0x0E,
            0x00, 0x00, 0x0E, 0x0E,
            0x00, 0x00, 0x0E, 0x0E,
            0x00, 0x00, 0x0E, 0x0E,
            0x00, 0x00, 0x0E, 0x0E,
        ],
    );
    let mut expected_state = state.clone();

    state.step();
    expected_state.stack_pointer = 2;
    expected_state.program_counter = 0xF0 + 2;

    assert_eq!(state, expected_state);

    state.step();
    expected_state.stack_pointer = 1;
    expected_state.program_counter = 0xF1 + 2;

    assert_eq!(state, expected_state);

    state.step();
    expected_state.stack_pointer = 0;
    expected_state.program_counter = 0xF2 + 2;

    assert_eq!(state, expected_state);

    state.step();
    expected_state.stack_pointer = 0xFF;
    expected_state.program_counter = 0xF3 + 2;

    assert_eq!(state, expected_state);
}

#[test]
fn jump() {
    let mut state = Chip8::new("".into());
    state.set_memory(state.program_counter, vec![0x01, 0x01, 0x02, 0x03]);
    let mut expected_state = state.clone();

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
