use core::cmp::min;

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use super::stack::Stack;

/// Number of elements storable in the emulator's stack (at least 16)
const STACK_SIZE: usize = 32;
/// The whole emulator's memory is RAM.
const RAM_SIZE: usize = 0x1000;
const RAM_MASK: u16 = (RAM_SIZE - 1) as u16;
/// Every Program should start at this address.
const PROGRAM_START: usize = 0x200;
/// The maximum size a program can use.
pub const PROGRAM_SIZE: usize = RAM_SIZE - PROGRAM_START;
/// Maximum display width, used by Super-chip and XO-chip.
pub const DISPLAY_WIDTH: usize = 128;
/// Maximum display height, used by Super-chip and XO-chip.
pub const DISPLAY_HEIGHT: usize = 64;
/// Number of registers used by the emulator.
const REGISTERS_COUNT: usize = 16;
/// Numbers of keys used by the system.
const KEYS_COUNT: usize = 16;
/// The location in memory of the font sprite '0'.
const FONT_SPRITES_ADDRESS: usize = 0;
/// The address step between two consecutive font sprites.
const FONT_SPRITES_STEP: usize = 5;
/// The number of font sprites (1 for each character from '0' to 'F').
const FONT_SPRITES_COUNT: usize = 16;
/// The font sprites, from '0' to 'F'.
const FONT_SPRITES: [u8; FONT_SPRITES_STEP * FONT_SPRITES_COUNT] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
/// The location in memory of the high-resolution font sprite '0'.
const FONT_SPRITES_HIGH_ADDRESS: usize = FONT_SPRITES_ADDRESS + FONT_SPRITES.len();
/// The address step between two consecutive high-resolution font sprites.
const FONT_SPRITES_HIGH_STEP: usize = 10;
/// The font sprites, from '0' to 'F'.
// const FONT_SPRITES_HIGH: [u8; FONT_SPRITES_HIGH_STEP * FONT_SPRITES_COUNT] = [];

/// Number of CPU steps executed each second.
pub const STEPS_PER_SECOND: usize = 500;
/// Refresh rate, number of frames per second.
/// Also dictates the decrease rate of the emulator's timers.
pub const REFRESH_RATE_HZ: usize = 60;
/// Number of CPU steps executed between two consecutive frames.
/// Also dictates the number of steps between two timer decreases.
const STEPS_PER_FRAME: usize = STEPS_PER_SECOND / REFRESH_RATE_HZ;

/// The mode in which the emulator runs, affects the display size and the
/// way some instruction are handled.
#[derive(PartialEq)]
pub enum Chirp8Mode {
    /// Original Cosmac VIP chip-8 mode from 1977, uses 64x32 display.
    CosmacChip8,
    /// HP48 Super-Chip extension from 1984, uses 128x64 display.
    SuperChip,
    // TODO : SuperChipLegacy,
    // TODO : XOChip,
}

/// The display size currently used by the emulator.
pub struct DisplaySize {
    pub width: usize,
    pub height: usize,
}

/// Chip-8 Emulator.
pub struct Chirp8 {
    ram: [u8; RAM_SIZE],
    /// Display buffer, true when pixel is on, false otherwise.
    display_buffer: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    /// V0 to VF.
    registers: [u8; REGISTERS_COUNT],
    /// Program counter.
    pc: u16,
    /// Index register, "I".
    index: u16,
    stack: Stack<u16, STACK_SIZE>,
    sound_timer: u8,
    delay_timer: u8,
    /// Each key is set to true whe pressed and false when released.
    keys: [bool; KEYS_COUNT],
    /// On Super Chip 8, true when high-resolution is enabled.
    high_resolution: bool,

    /// The current running mode of the emulator.
    mode: Chirp8Mode,
    /// Number of cpu steps taken since last timer step.
    steps_since_timer: usize,
    /// Meta flag to indicate that the display changed.
    display_changed: bool,
    /// Random numbers generator
    randomizer: SmallRng,
}

impl Default for Chirp8 {
    fn default() -> Self {
        Self::new(Chirp8Mode::CosmacChip8)
    }
}

impl Chirp8 {
    /// Creates a new emulator, which will behave according to given `mode`.
    pub fn new(mode: Chirp8Mode) -> Self {
        // Load font to RAM
        let mut ram = [0u8; RAM_SIZE];
        const FONT_SPRITES_SIZE: usize = FONT_SPRITES_COUNT * FONT_SPRITES_STEP;
        ram[FONT_SPRITES_ADDRESS..FONT_SPRITES_ADDRESS + FONT_SPRITES_SIZE]
            .copy_from_slice(&FONT_SPRITES);
        // TODO copy FONT_HIGH_RES

        // Create emulator
        Self {
            ram: ram,
            display_buffer: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            registers: [0; REGISTERS_COUNT],
            pc: PROGRAM_START as u16,
            index: 0,
            stack: Stack::new(),
            sound_timer: 0,
            delay_timer: 0,
            keys: [false; KEYS_COUNT],
            high_resolution: false,
            mode: mode,
            steps_since_timer: 0,
            display_changed: false,
            randomizer: SmallRng::seed_from_u64(0xDEADCAFEDEADCAFE),
        }
    }

    /// Press the given `key` on the key-pad, between 0 and 15 included.
    pub fn key_press(&mut self, key: usize) {
        if key < KEYS_COUNT {
            self.keys[key] = true;
        }
    }

    /// Release the given `key` on the key-pad, between 0 and 15 included.
    pub fn key_release(&mut self, key: usize) {
        if key < KEYS_COUNT {
            self.keys[key] = false;
        }
    }

    /// Set the given `key` on the key-pad to given `value`, between 0 and 15 included. `pressed` is true when pressed, false when released.
    pub fn key_set(&mut self, key: usize, pressed: bool) {
        if key < KEYS_COUNT {
            self.keys[key] = pressed;
        }
    }

    /// Run as many instruction as necessary to generate a frame.
    pub fn run_frame(&mut self) {
        for _ in 0..STEPS_PER_FRAME {
            self.step();
        }
    }

    fn next_instruction(&self) -> u16 {
        const BITS_IN_BYTE: u16 = 8;
        ((self.ram[self.pc as usize] as u16) << BITS_IN_BYTE)
            + (self.ram[self.pc as usize + 1] as u16)
    }

    /// Execute one machine instruction.
    pub fn step(&mut self) {
        // Big endian instruction
        let instruction = self.next_instruction();
        self.pc = (self.pc + 2) & RAM_MASK;

        // See https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
        let opcode = 0xF & (instruction >> 12) as u8;
        // The second nibble. Used to look up one of the 16 registers (VX) from V0 through VF.
        let x = (0x0F & (instruction >> 8) as u8) as usize;
        // The third nibble. Also used to look up one of the 16 registers (VY) from V0 through VF.
        let y = (0x0F & (instruction >> 4) as u8) as usize;
        // The fourth nibble. A 4-bit number.
        let n = 0x0F & instruction as u8;
        // The second byte (third and fourth nibbles). An 8-bit immediate number.
        let nn = 0xFF & instruction as u8;
        // The second, third and fourth nibbles. A 12-bit immediate memory address.
        let nnn = 0x0FFF & instruction;

        match opcode {
            0x0 => match instruction {
                // Clear screen
                0x00E0 => {
                    for row in &mut self.display_buffer {
                        row.fill(false)
                    }
                }
                // Return from subroutine
                0x00EE => self.pc = self.stack.pop().ok().unwrap(),
                // Exit from interpreter (S-Chip) (do nothing here)
                0x00FD => (),
                // Disable High-res (S-Chip)
                0x00FE => self.high_resolution = false,
                // Enable High-res (S-chip)
                0x00FF => self.high_resolution = true,
                // TODO 0x00CN : scroll down n pixels
                // TODO 0x00FB : scroll right 4 pixels
                // TODO 0x00CN : scroll left 4 pixels
                _ => panic!("Unrecognized 0 instruction {:x}", instruction),
            },
            // Jump
            0x1 => self.pc = nnn,
            // Call subroutine
            0x2 => {
                self.stack.push(self.pc).ok().unwrap();
                self.pc = nnn;
            }
            // Skip
            0x3 => {
                if self.registers[x] == nn {
                    self.pc = (self.pc + 2) & RAM_MASK;
                }
            }
            // Skip
            0x4 => {
                if self.registers[x] != nn {
                    self.pc = (self.pc + 2) & RAM_MASK;
                }
            }
            // Skip
            0x5 => {
                // n should be equal to 0 (0x5XY0), not checked for performance.
                if self.registers[x] == self.registers[y] {
                    self.pc = (self.pc + 2) & RAM_MASK;
                }
            }
            // Skip
            0x9 => {
                // n should be equal to 0 (0x9XY0), not checked for performance.
                if self.registers[x] != self.registers[y] {
                    self.pc = (self.pc + 2) & RAM_MASK
                }
            }
            // Set register
            0x6 => self.registers[x] = nn,
            // Add to register
            0x7 => self.registers[x] = self.registers[x].wrapping_add(nn),
            // Logic and arithmetics
            0x8 => match n {
                // Set
                0x0 => self.registers[x] = self.registers[y],
                // OR
                0x1 => self.registers[x] |= self.registers[y],
                // AND
                0x2 => self.registers[x] &= self.registers[y],
                // XOR
                0x3 => self.registers[x] ^= self.registers[y],
                // ADD
                0x4 => {
                    let flag = if self.registers[x].checked_add(self.registers[y]) == Option::None {
                        1
                    } else {
                        0
                    };
                    self.registers[x] = self.registers[x].wrapping_add(self.registers[y]);
                    self.registers[0xF] = flag;
                }
                // SUB VX - VY
                0x5 => {
                    let flag = if self.registers[x] >= self.registers[y] {
                        1
                    } else {
                        0
                    };
                    self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
                    self.registers[0xF] = flag;
                }
                // Shift VX right
                0x6 => {
                    if self.mode == Chirp8Mode::CosmacChip8 {
                        self.registers[x] = self.registers[y];
                    }
                    let flag = self.registers[x] & 0x1;
                    self.registers[x] >>= 1;
                    self.registers[0xF] = flag;
                }
                // SUB VY - VX
                0x7 => {
                    let flag = if self.registers[y] >= self.registers[x] {
                        1
                    } else {
                        0
                    };
                    self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
                    self.registers[0xF] = flag;
                }
                // Shift VX left
                0xE => {
                    if self.mode == Chirp8Mode::CosmacChip8 {
                        self.registers[x] = self.registers[y];
                    }
                    let flag = (self.registers[x] >> 7) & 0x1;
                    self.registers[x] <<= 1;
                    self.registers[0xF] = flag;
                }
                _ => panic!("Unrecognized logic instruction {:x}", n),
            },
            // Set index
            0xA => self.index = nnn,
            // Jump with offset
            0xB => {
                self.pc = (nnn
                    + self.registers[if self.mode == Chirp8Mode::CosmacChip8 {
                        0
                    } else {
                        x
                    }] as u16)
                    & RAM_MASK
            }
            // Random
            0xC => self.registers[x] = (self.randomizer.next_u32() as u8) & nn,
            // Display
            0xD => self.display((self.registers[x], self.registers[y]), n),
            // Skip if key
            0xE => match nn {
                // Skip if VX pressed
                0x9E => {
                    let key = (0xF & self.registers[x]) as usize;
                    if self.keys[key] {
                        self.pc = (self.pc + 2) & RAM_MASK;
                    }
                }
                // Skip if VX not pressed
                0xA1 => {
                    let key = (0xF & self.registers[x]) as usize;
                    if !self.keys[key] {
                        self.pc = (self.pc + 2) & RAM_MASK;
                    }
                }
                _ => panic!("Unrecognized E instruction {:x}", instruction),
            },
            0xF => {
                match nn {
                    // Timers set VX
                    0x07 => self.registers[x] = self.delay_timer,
                    0x15 => self.delay_timer = self.registers[x],
                    0x18 => self.sound_timer = self.registers[x],
                    // Add to index
                    0x1E => {
                        self.index = self.index + self.registers[x] as u16;
                        if self.index & !RAM_MASK != 0 {
                            self.set_flag();
                            self.index &= RAM_MASK;
                        }
                    }
                    // Get Key
                    0x0A => {
                        if let Option::Some(key) = self.get_first_key() {
                            self.registers[x] = key;
                        } else {
                            self.pc -= 2;
                        }
                    }
                    // FX29: Font character
                    0x29 => {
                        // TODO SuperChip1.0 : Point I to 5-byte font sprite as in CHIP-8, but if the high nibble in VX is 1 (ie. for values between 10 and 19 in hex) it will point I to a 10-byte font sprite for the digit in the lower nibble of VX (only digits 0-9)
                        self.index = FONT_SPRITES_ADDRESS as u16
                            + FONT_SPRITES_STEP as u16 * self.registers[x & 0xF] as u16;
                    }
                    // FX30: Large font character (Super-Chip 1.1)
                    0x30 => {
                        self.index = FONT_SPRITES_HIGH_ADDRESS as u16
                            + FONT_SPRITES_HIGH_STEP as u16 * self.registers[x & 0xF] as u16;
                    }

                    // FX33: Binary-coded decimal conversion
                    0x33 => {
                        let mut value = self.registers[x];
                        self.ram[self.index as usize] = value / 100;
                        value %= 100;
                        self.ram[self.index as usize + 1] = value / 10;
                        value %= 10;
                        self.ram[self.index as usize + 2] = value;
                    }
                    // FX55 : Store
                    0x55 => {
                        let end_index = x + 1;
                        for i in 0..end_index {
                            self.ram[(i + self.index as usize) & RAM_MASK as usize] =
                                self.registers[i];
                        }
                        // if mode == SuperChip1.0 self.index = (self.index + (end_index as u16) - 1) & RAM_MASK;
                        if self.mode == Chirp8Mode::CosmacChip8 {
                            self.index = (self.index + end_index as u16) & RAM_MASK;
                        }
                    }
                    // FX65: Load
                    0x65 => {
                        let end_index = x + 1;
                        for i in 0..end_index {
                            self.registers[i] =
                                self.ram[(i + self.index as usize) & RAM_MASK as usize];
                        }
                        // if mode == SuperChip1.0 self.index = (self.index + (end_index as u16) - 1) & RAM_MASK;
                        if self.mode == Chirp8Mode::CosmacChip8 {
                            self.index = (self.index + end_index as u16) & RAM_MASK;
                        }
                    }
                    // Save flags registers (Super-Chip)
                    0x75 => (), //TODO : not supported at the moment
                    // Load flags registers (Super-Chip)
                    0x85 => (), //TODO : not supported at the moment
                    _ => panic!("Unrecognized E instruction {:x}", instruction),
                }
            }

            _ => panic!("Unrecognized instruction {:x}", instruction),
        }
        // Handle timers
        self.steps_since_timer += 1;
        if self.steps_since_timer == STEPS_PER_FRAME {
            self.steps_since_timer = 0;
            if self.delay_timer != 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer != 0 {
                self.sound_timer -= 1;
            }
        }
    }

    #[inline]
    fn set_flag(&mut self) {
        self.registers[0xF] = 1;
    }

    #[inline]
    fn reset_flag(&mut self) {
        self.registers[0xF] = 0;
    }

    /// Returns the first pressed key index, between 0 and 15 included, or `Option::None` when nothing is pressed.
    fn get_first_key(&self) -> Option<u8> {
        for (index, key) in self.keys.iter().enumerate() {
            if *key {
                return Option::Some(index as u8);
            }
        }
        Option::None
    }

    /// Display `height`-pixel tall sprite pointed by index register at given `x_y_coordinates`.
    /// If `height` is 0 then a large 16x16 sprite is used.
    fn display(&mut self, x_y_coordinates: (u8, u8), height: u8) {
        self.display_changed = true;
        self.reset_flag();
        /// Bits in a byte.
        const BITS: usize = 8;

        let high_resolution = self.mode == Chirp8Mode::SuperChip && self.high_resolution;

        if high_resolution && height == 0 {
            // Handle instruction DXY0 : display 16x16 sprite

            /// Width and Height of large sprites.
            const LARGE_SPRITE_SIZE: usize = 16;
            /// Bytes per line for large sprites.
            const BYTES_PER_LINE: usize = 2;

            let actual_height = min(
                LARGE_SPRITE_SIZE,
                DISPLAY_HEIGHT.saturating_sub(x_y_coordinates.1 as usize),
            );
            let actual_width = min(
                LARGE_SPRITE_SIZE,
                DISPLAY_WIDTH.saturating_sub(x_y_coordinates.0 as usize),
            );

            for line in 0..actual_height {
                for part in 0..BYTES_PER_LINE {
                    let sprite_address =
                        (self.index as usize + BYTES_PER_LINE * line + part) & RAM_MASK as usize;
                    let sprite = self.ram[sprite_address];
                    let row = (x_y_coordinates.1 as usize % DISPLAY_HEIGHT) + line;
                    for bit in 0..(min(BITS, actual_width - part * BITS)) {
                        let col = x_y_coordinates.0 as usize % DISPLAY_WIDTH
                            + part * BYTES_PER_LINE
                            + bit;

                        // Should the pixel be flipped or not.
                        let pixel_xor = ((sprite >> (BITS - 1 - bit)) & 1) != 0;

                        let pixel = &mut self.display_buffer[row][col];
                        let pixel_before = *pixel;
                        *pixel ^= pixel_xor;
                        // Set flag when turned off
                        if pixel_before && !*pixel {
                            self.set_flag();
                        }
                    }
                }
            }
        } else {
            // Handle instruction DXYN : display 8xN sprite

            // Maximum input coordinates
            let (max_width, max_height, coordinates_scaler) = if high_resolution {
                (DISPLAY_WIDTH, DISPLAY_HEIGHT, 1)
            } else {
                (DISPLAY_WIDTH / 2, DISPLAY_HEIGHT / 2, 2)
            };

            let x_y_coordinates = (
                x_y_coordinates.0 % max_width as u8,
                x_y_coordinates.1 % max_height as u8,
            );

            let actual_height =
                min(height, (max_height as u8).saturating_sub(x_y_coordinates.1)) as usize;
            let actual_width = min(
                BITS as u8,
                (max_width as u8).saturating_sub(x_y_coordinates.0),
            ) as usize;

            for line in 0..actual_height {
                let sprite_address = ((self.index + line as u16) & RAM_MASK) as usize;
                let sprite = self.ram[sprite_address];
                let row = ((x_y_coordinates.1 as usize) + line) * coordinates_scaler;
                for bit in 0..actual_width {
                    let col = (x_y_coordinates.0 as usize + bit) * coordinates_scaler;

                    // Should the pixel be flipped or not.
                    let pixel_xor = ((sprite >> (BITS - 1 - bit)) & 1) != 0;

                    let pixel_before = self.display_buffer[row][col];
                    let mut pixel = pixel_before;
                    pixel ^= pixel_xor;
                    self.display_buffer[row][col] = pixel;
                    if !high_resolution {
                        // Draw 2x2 "pixels" when on low resolution
                        self.display_buffer[row][col + 1] = pixel;
                        self.display_buffer[row + 1][col] = pixel;
                        self.display_buffer[row + 1][col + 1] = pixel;
                    }
                    // Set flag when turned off
                    if pixel_before && !pixel {
                        self.set_flag();
                    }
                }
            }
        }
    }

    /// Indicates if the display changed since the last time this method was called.
    pub fn display_changed(&mut self) -> bool {
        let result = self.display_changed;
        self.display_changed = false;
        result
    }

    /// Indicates whether the sound buzzer is currently on.
    pub fn is_buzzer_on(&self) -> bool {
        self.sound_timer > 0
    }

    /// Load a ROM into memory.
    /// The ROM may be smaller than array, in that case pad with any value.
    pub fn load_rom(&mut self, rom: &[u8; PROGRAM_SIZE]) {
        self.ram[PROGRAM_START..(PROGRAM_START + PROGRAM_SIZE)].copy_from_slice(rom);
    }

    /// Returns a reference to the internal display buffer.
    /// Notice that when running on Cosmac mode, each "pixel" is displayed as a 2 by 2 square,
    /// in order to match the resolution of the Super-Chip mode.
    pub fn get_display_buffer(&self) -> &[[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT] {
        &self.display_buffer
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn opcode_set_vx_nn() {
        let mut emulator = Chirp8::default();
        emulator.ram[PROGRAM_START..PROGRAM_START + 2].copy_from_slice(&[0x63, 0xAB]);
        emulator.step();

        assert_eq!(emulator.registers[3], 0xAB);
    }

    #[test]
    fn opcode_skip_if_key_pressed() {
        let mut emulator = Chirp8::default();
        emulator.ram[PROGRAM_START..PROGRAM_START + 2].copy_from_slice(&[0xE2, 0x9E]);
        emulator.registers[2] = 11;

        emulator.key_release(11);
        let pc_before = emulator.pc;
        emulator.step();
        assert_eq!(emulator.pc, pc_before + 2);

        emulator.pc = PROGRAM_START as u16;

        emulator.key_press(11);
        let pc_before = emulator.pc;
        emulator.step();
        assert_eq!(emulator.pc, pc_before + 4);
    }

    #[test]
    fn opcode_skip_if_key_not_pressed() {
        let mut emulator = Chirp8::default();
        emulator.ram[PROGRAM_START..PROGRAM_START + 2].copy_from_slice(&[0xE2, 0xA1]);
        emulator.registers[2] = 11;

        emulator.key_release(11);
        let pc_before = emulator.pc;
        emulator.step();
        assert_eq!(emulator.pc, pc_before + 4);

        emulator.pc = PROGRAM_START as u16;

        emulator.key_press(11);
        let pc_before = emulator.pc;
        emulator.step();
        assert_eq!(emulator.pc, pc_before + 2);
    }
    // TODO : test other opcodes
}
