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
const KEYS_COUNT: u8 = 16;
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
/// The high-resolution font sprites, from '0' to 'F' (only 0-9 are present in original interpreter).
const FONT_SPRITES_HIGH: [u8; FONT_SPRITES_HIGH_STEP * FONT_SPRITES_COUNT] = [
    0x3C, 0x7E, 0xE7, 0xC3, 0xC3, 0xC3, 0xC3, 0xE7, 0x7E, 0x3C, // 0
    0x18, 0x38, 0x58, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x3C, // 1
    0x3E, 0x7F, 0xC3, 0x06, 0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, // 2
    0x3C, 0x7E, 0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C, // 3
    0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF, 0x06, 0x06, // 4
    0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE, 0x03, 0xC3, 0x7E, 0x3C, // 5
    0x3E, 0x7C, 0xC0, 0xC0, 0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, // 6
    0xFF, 0xFF, 0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60, // 7
    0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3, 0x7E, 0x3C, // 8
    0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F, 0x03, 0x03, 0x3E, 0x7C, // 9
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // A (absent)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // B (absent)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // C (absent)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // D (absent)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // E (absent)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // F (absent)
];
/// Refresh rate, number of frames per second.
/// Also dictates the decrease rate of the emulator's timers.
pub const REFRESH_RATE_HZ: usize = 60;
/// Number of CPU steps executed between two consecutive frames.
/// Also dictates the number of steps between two timer decreases.
const STEPS_PER_FRAME: usize = 10; // TODO : make this a parameter.
/// Number of CPU steps executed each second.
pub const STEPS_PER_SECOND: usize = STEPS_PER_FRAME * REFRESH_RATE_HZ;
/// Number of RPL flags registers on the HP48.
const RPL_REGISTERS_COUNT: usize = 8;
/// Number of memory bytes read by CPU at each cycle.
const PROGRAM_COUNTER_STEP: u16 = 2;

/// The mode in which the emulator runs, affects the display size and the
/// way some instruction are handled.
#[derive(PartialEq)]
pub enum Chirp8Mode {
    /// Original Cosmac VIP chip-8 mode from 1977, uses 64x32 display.
    CosmacChip8,
    // HP48 Super-Chip 1.1 extension from 1991, uses 128x64 display.
    SuperChip1_1,
    /// Modernized Super-Chip 1.1 extension from 1991, uses 128x64 display.
    /// Like "modern" interpreters, does not feature the "display wait" quirk,
    /// where the Super-Chip 1.1 would feature the display wait but only in low-resolution.
    /// Does not feature the "half scroll" quirk as well, where the original interpreter would
    /// allow to scroll half pixels when in low-resolution.
    SuperChipModern,
    // TODO : XOChip,
    // TODO SuperChip1_0
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
    /// Persistent RPL flags registers.
    rpl_registers: [u8; RPL_REGISTERS_COUNT],

    /// Each key is set to true whe pressed and false when released.
    keys: [bool; KEYS_COUNT as usize],
    /// On Super Chip 8, true when high-resolution is enabled.
    high_resolution: bool,

    /// The current running mode of the emulator.
    mode: Chirp8Mode,
    /// Number of cpu steps taken since last timer step.
    steps_since_frame: usize,
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
        const FONT_SPRITES_HIGH_SIZE: usize = FONT_SPRITES_COUNT * FONT_SPRITES_HIGH_STEP;
        ram[FONT_SPRITES_HIGH_ADDRESS..FONT_SPRITES_HIGH_ADDRESS + FONT_SPRITES_HIGH_SIZE]
            .copy_from_slice(&FONT_SPRITES_HIGH);

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
            rpl_registers: [0; RPL_REGISTERS_COUNT],
            keys: [false; KEYS_COUNT as usize],
            high_resolution: false,
            mode: mode,
            steps_since_frame: 0,
            display_changed: true,
            randomizer: SmallRng::seed_from_u64(0xDEADCAFEDEADCAFE),
        }
    }

    /// Press the given `key` on the key-pad, between 0 and 15 included.
    pub fn key_press(&mut self, key: u8) {
        if key < KEYS_COUNT {
            self.keys[key as usize] = true;
        }
    }

    /// Release the given `key` on the key-pad, between 0 and 15 included.
    pub fn key_release(&mut self, key: u8) {
        if key < KEYS_COUNT {
            self.keys[key as usize] = false;
        }
    }

    /// Set the given `key` on the key-pad to given `value`, between 0 and 15 included.
    /// `pressed` is true when pressed, false when released.
    pub fn key_set(&mut self, key: u8, pressed: bool) {
        if key < KEYS_COUNT {
            self.keys[key as usize] = pressed;
        }
    }

    /// Run as many instruction as necessary to generate a frame.
    pub fn run_frame(&mut self) {
        for _ in 0..STEPS_PER_FRAME {
            self.step();
        }
    }

    /// Get the next instruction to execute from memory.
    fn next_instruction(&self) -> u16 {
        const BITS_IN_BYTE: u16 = 8;
        ((self.ram[self.pc as usize] as u16) << BITS_IN_BYTE)
            + (self.ram[self.pc as usize + 1] as u16)
    }

    /// Resets interpreter to beginning of program.
    pub fn reset(&mut self) {
        self.pc = PROGRAM_START as u16;
        self.registers.fill(0);
        self.display_changed = true;
        for row in &mut self.display_buffer {
            row.fill(false);
        }
    }

    /// Execute one machine instruction.
    pub fn step(&mut self) {
        // Big endian instruction
        let instruction = self.next_instruction();
        self.pc = (self.pc + PROGRAM_COUNTER_STEP) & RAM_MASK;

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
            0x0 => match nn {
                // NOP
                0x00 => {}
                // // Exit from interpreter (chip8run)
                // 0x10..=0x1F => self.reset(),
                // Clear screen
                0xE0 => {
                    for row in &mut self.display_buffer {
                        row.fill(false)
                    }
                }
                // Return from subroutine
                0xEE => self.pc = self.stack.pop().ok().unwrap(),
                // Exit from interpreter (S-Chip)
                0xFD => self.reset(),
                // Disable High-res (S-Chip)
                0xFE => self.high_resolution = false,
                // Enable High-res (S-chip)
                0xFF => self.high_resolution = true,
                // Scroll up N pixels (Unofficial Super Chip)
                0xB0..=0xBF => self.scroll_up(n),
                // Scroll down N pixels (Super Chip)
                0xC0..=0xCF => self.scroll_down(n),
                // Scroll right 4 pixels (Super Chip)
                0xFB => self.scroll_right(4),
                // Scroll left 4 pixels (Super Chip)
                0xFC => self.scroll_left(4),
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
                    self.pc = (self.pc + PROGRAM_COUNTER_STEP) & RAM_MASK;
                }
            }
            // Skip
            0x4 => {
                if self.registers[x] != nn {
                    self.pc = (self.pc + PROGRAM_COUNTER_STEP) & RAM_MASK;
                }
            }
            // Skip
            0x5 => {
                // n should be equal to 0 (0x5XY0), not checked for performance.
                if self.registers[x] == self.registers[y] {
                    self.pc = (self.pc + PROGRAM_COUNTER_STEP) & RAM_MASK;
                }
            }
            // Skip
            0x9 => {
                // n should be equal to 0 (0x9XY0), not checked for performance.
                if self.registers[x] != self.registers[y] {
                    self.pc = (self.pc + PROGRAM_COUNTER_STEP) & RAM_MASK
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
                0x1 => {
                    self.registers[x] |= self.registers[y];
                    if self.mode == Chirp8Mode::CosmacChip8 {
                        self.reset_flag();
                    }
                }
                // AND
                0x2 => {
                    self.registers[x] &= self.registers[y];
                    if self.mode == Chirp8Mode::CosmacChip8 {
                        self.reset_flag();
                    }
                }
                // XOR
                0x3 => {
                    self.registers[x] ^= self.registers[y];
                    if self.mode == Chirp8Mode::CosmacChip8 {
                        self.reset_flag();
                    }
                }
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
            0xD => {
                self.handle_display_wait();
                self.display((self.registers[x], self.registers[y]), n)
            }
            // Skip if key
            0xE => match nn {
                // Skip if VX pressed
                0x9E => {
                    let key = (0xF & self.registers[x]) as usize;
                    if self.keys[key] {
                        self.pc = (self.pc + PROGRAM_COUNTER_STEP) & RAM_MASK;
                    }
                }
                // Skip if VX not pressed
                0xA1 => {
                    let key = (0xF & self.registers[x]) as usize;
                    if !self.keys[key] {
                        self.pc = (self.pc + PROGRAM_COUNTER_STEP) & RAM_MASK;
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
                        // TODO : should wait for a key release.
                        if let Option::Some(key) = self.get_first_key() {
                            self.registers[x] = key;
                        } else {
                            self.pc -= 2;
                        }
                    }
                    // FX29: Font character
                    0x29 => {
                        // Not implemented : SuperChip1.0 : Point I to 5-byte font sprite as in CHIP-8,
                        // but if the high nibble in VX is 1 (ie. for values between 10 and 19 in hex) it will
                        // point I to a 10-byte font sprite for the digit in the lower nibble of VX (only digits 0-9).
                        // The following is the SuperChip1.1 behavior.
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
                    // FX75 : Save to flags registers (Super-Chip 1.0 and above)
                    0x75 => {
                        let count = x & 0x7;
                        self.rpl_registers[0..count].copy_from_slice(&self.registers[0..count]);
                    }
                    // FX85 : Load from flags registers (Super-Chip 1.0 and above)
                    0x85 => {
                        let count = x & 0x7;
                        self.registers[0..count].copy_from_slice(&self.rpl_registers[0..count]);
                    }
                    _ => panic!("Unrecognized E instruction {:x}", instruction),
                }
            }

            _ => panic!("Unrecognized instruction {:x}", instruction),
        }
        // Handle timers
        self.step_timers();
    }

    fn step_timers(&mut self) {
        self.steps_since_frame += 1;
        if self.steps_since_frame == STEPS_PER_FRAME {
            self.steps_since_frame = 0;
            self.delay_timer = self.delay_timer.saturating_sub(1);
            self.sound_timer = self.sound_timer.saturating_sub(1);
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

    /// Handle the *display wait quirk* before displaying anything.
    /// - The Cosmac chip-8 waits the display interrupt before displaying.
    /// - The original Super chip (legacy) waits but only in low-resolution.
    /// - The *modern* Super chip never waits.
    /// This method implements the original cosmac and the "modern" super-chip behaviors.
    fn handle_display_wait(&mut self) {
        // See : https://github.com/Timendus/chip8-test-suite/blob/main/legacy-superchip.md

        let wait_enabled = self.mode == Chirp8Mode::CosmacChip8
            || (self.mode == Chirp8Mode::SuperChip1_1 && !self.high_resolution);

        if wait_enabled {
            // Wait for next frame, but instead of waiting, shortcut time.
            // The other option is to decrement pc if steps_since_frame is not 0,
            // But that messes with the number of steps required to do something,
            // as 10 steps would be necessary to only execute a draw.
            while self.steps_since_frame != 0 {
                self.step_timers();
            }
        }
    }

    /// Display `height`-pixel tall sprite pointed by index register at given `x_y_coordinates`.
    /// If `height` is 0 then a large 16x16 sprite is used.
    fn display(&mut self, x_y_coordinates: (u8, u8), height: u8) {
        self.display_changed = true;
        self.reset_flag();
        /// Bits in a byte.
        const BITS: usize = 8;

        let high_resolution = self.mode != Chirp8Mode::CosmacChip8 && self.high_resolution;

        // TODO : in high resolution mode, VF is set to the number of colliding rows, not just 0 or 1

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
                        let col = x_y_coordinates.0 as usize % DISPLAY_WIDTH + part * BITS + bit;

                        // Should the pixel be flipped or not.
                        let pixel_xor = ((sprite >> (BITS - 1 - bit)) & 1) != 0;

                        let pixel = &mut self.display_buffer[row][col];
                        let pixel_before = *pixel;
                        *pixel ^= pixel_xor;
                        // Set flag when turned off
                        if pixel_before && !(*pixel) {
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

    /// Scrolls up display by `scroll` pixels.
    /// This is the "modern" behavior where in low-res, the screens scrolls by `scroll` low-res pixels,
    /// It does not scrolls by `scroll` half-pixels as it would on the original Super-CHip 1.1.
    fn scroll_up(&mut self, scroll: u8) {
        // mode == Cosmac Chip 8 is not checked, should not happen.
        let actual_scroll = if self.mode == Chirp8Mode::SuperChipModern {
            if self.high_resolution {
                scroll
            } else {
                scroll * 2
            }
        } else {
            scroll
        } as usize;
        self.display_buffer.rotate_left(actual_scroll);
        // Bottom of screen is black.
        for black_row in &mut self.display_buffer[(DISPLAY_HEIGHT - actual_scroll)..DISPLAY_HEIGHT]
        {
            black_row.fill(false);
        }
    }

    /// Scrolls down display by `scroll` pixels.
    /// This is the "modern" behavior where in low-res, the screens scrolls by `scroll` low-res pixels,
    /// It does not scrolls by `scroll` half-pixels as it would on the original Super-CHip 1.1.
    fn scroll_down(&mut self, scroll: u8) {
        // mode == Cosmac Chip 8 is not checked, should not happen.
        let actual_scroll = if self.mode == Chirp8Mode::SuperChipModern {
            if self.high_resolution {
                scroll
            } else {
                scroll * 2
            }
        } else {
            scroll
        } as usize;
        self.display_buffer.rotate_right(actual_scroll);
        // Top of screen is black.
        for black_row in &mut self.display_buffer[0..actual_scroll] {
            black_row.fill(false);
        }
    }

    /// Scrolls left display by `scroll` pixels.
    /// This is the "modern" behavior where in low-res, the screens scrolls by `scroll` low-res pixels,
    /// It does not scrolls by `scroll` half-pixels as it would on the original Super-CHip 1.1.
    fn scroll_left(&mut self, scroll: u8) {
        let actual_scroll = if self.mode == Chirp8Mode::SuperChipModern {
            if self.high_resolution {
                scroll
            } else {
                scroll * 2
            }
        } else {
            scroll
        } as usize;
        for row in &mut self.display_buffer {
            row.rotate_left(actual_scroll);
            row[(DISPLAY_WIDTH - actual_scroll)..DISPLAY_WIDTH].fill(false);
        }
    }

    /// Scrolls right display by `scroll` pixels.
    /// This is the "modern" behavior where in low-res, the screens scrolls by `scroll` low-res pixels,
    /// It does not scrolls by `scroll` half-pixels as it would on the original Super-CHip 1.1.
    fn scroll_right(&mut self, scroll: u8) {
        let actual_scroll = if self.mode == Chirp8Mode::SuperChipModern {
            if self.high_resolution {
                scroll
            } else {
                scroll * 2
            }
        } else {
            scroll
        } as usize;
        for row in &mut self.display_buffer {
            row.rotate_right(actual_scroll);
            row[0..actual_scroll].fill(false);
        }
    }

    /// Indicates whether the sound buzzer is currently on.
    pub fn is_buzzer_on(&self) -> bool {
        self.sound_timer > 0
    }

    /// Load a ROM into memory. The ROM must be smaller than `PROGRAM_SIZE`.
    pub fn load_rom(&mut self, rom: &[u8]) {
        self.ram[PROGRAM_START..(PROGRAM_START + rom.len())].copy_from_slice(rom);
    }

    /// Load given data into persistent RPL registers.
    pub fn load_rpl_registers(&mut self, registers: &[u8; RPL_REGISTERS_COUNT]) {
        self.rpl_registers.copy_from_slice(registers);
    }

    /// Get persistent RPL registers.
    pub fn get_rpl_registers(&self) -> &[u8; RPL_REGISTERS_COUNT] {
        &self.rpl_registers
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

    #[test]
    fn opcode_draw_high_res() {
        let mut emulator = Chirp8::new(Chirp8Mode::SuperChipModern);
        emulator.ram[PROGRAM_START..PROGRAM_START + 5].copy_from_slice(&[
            0x00, 0xFF, // Enable High-res
            0xD0, 0x11, // Draw v0 v1 1
            0x80, // Sprite with one pixel to the left
        ]);
        emulator.registers[0] = 67;
        emulator.registers[1] = 45;
        emulator.index = PROGRAM_START as u16 + 4;

        emulator.step();
        emulator.step();

        assert_eq!(emulator.get_display_buffer()[45][67], true);
        assert_eq!(emulator.registers[0xF], 0);

        emulator.pc -= 2;
        emulator.step();

        assert_eq!(emulator.get_display_buffer()[45][67], false);
        assert_eq!(emulator.registers[0xF], 1);
    }

    #[test]
    fn opcode_scroll_vertical() {
        let rom = [
            0x00, 0xB5, // Scroll up by 5
            0x00, 0xC7, // Scroll down by 7
            0x80, // Sprite with one pixel to the left
        ];

        let mut emulator = Chirp8::new(Chirp8Mode::SuperChipModern);
        emulator.ram[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(&rom);
        emulator.display_buffer[37][67] = true;
        emulator.index = PROGRAM_START as u16 + 4;
        emulator.high_resolution = true;

        emulator.step();

        assert_eq!(emulator.display_buffer[37][67], false);
        assert_eq!(emulator.display_buffer[32][67], true);

        emulator.step();

        assert_eq!(emulator.display_buffer[32][67], false);
        assert_eq!(emulator.display_buffer[39][67], true);

        emulator.pc = PROGRAM_START as u16;
        emulator.high_resolution = false;

        emulator.step();

        assert_eq!(emulator.display_buffer[39][67], false);
        assert_eq!(emulator.display_buffer[29][67], true);

        emulator.step();

        assert_eq!(emulator.display_buffer[29][67], false);
        assert_eq!(emulator.display_buffer[43][67], true);
    }

    #[test]
    fn opcode_scroll_horizontal() {
        let rom = [
            0x00, 0xFB, // Scroll right
            0x00, 0xFC, // Scroll left
            0x80, // Sprite with one pixel to the left
        ];

        let mut emulator = Chirp8::new(Chirp8Mode::SuperChipModern);
        emulator.ram[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(&rom);
        emulator.display_buffer[37][67] = true;
        emulator.index = PROGRAM_START as u16 + 4;
        emulator.high_resolution = true;

        emulator.step();

        assert_eq!(emulator.display_buffer[37][67], false);
        assert_eq!(emulator.display_buffer[37][71], true);

        emulator.step();

        assert_eq!(emulator.display_buffer[37][71], false);
        assert_eq!(emulator.display_buffer[37][67], true);

        emulator.pc = PROGRAM_START as u16;
        emulator.high_resolution = false;

        emulator.step();

        assert_eq!(emulator.display_buffer[37][67], false);
        assert_eq!(emulator.display_buffer[37][75], true);

        emulator.step();

        assert_eq!(emulator.display_buffer[37][75], false);
        assert_eq!(emulator.display_buffer[37][67], true);
    }
    // TODO : test other opcodes
}
