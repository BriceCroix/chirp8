use core::cmp::min;
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use crate::QuirkFlags;

use super::stack::Stack;

/// Number of elements storable in the emulator's stack (originally 12, 16 from super chip and above).
const STACK_SIZE: usize = 16;
/// The whole emulator's memory is RAM : 12-bits addresses.
#[cfg(not(feature = "mem_extend"))]
const RAM_SIZE: usize = 0x1000;
/// The whole emulator's memory is RAM : 16-bits addresses.
#[cfg(feature = "mem_extend")]
const RAM_SIZE: usize = 0x10000;
/// A mask to use on addresses.
const RAM_MASK: u16 = (RAM_SIZE - 1) as u16;
/// Every Program should start at this address.
const PROGRAM_START: usize = 0x200;
/// The maximum size a program can use.
pub const PROGRAM_SIZE: usize = RAM_SIZE - PROGRAM_START;
/// Number of registers used by the emulator.
const REGISTERS_COUNT: usize = 16;
/// The index of the flag used as a flag register.
const FLAG_REGISTER_INDEX: usize = 0xF;
/// Numbers of keys used by the system.
const KEYS_COUNT: u8 = 16;
/// The location in memory of the font sprite '0'.
const FONT_SPRITES_ADDRESS: usize = 0;
/// The address step between two consecutive font sprites.
const FONT_SPRITES_STEP: usize = 5;
/// The number of font sprites (1 for each character from '0' to 'F').
const FONT_SPRITES_COUNT: usize = 16;
/// The font sprites, from '0' to 'F'. Same as CosmacVIP font.
#[rustfmt::skip]
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
/// The 0 to 9 font is taken from super chip, and A to F come from
/// [C-Octo](https://github.com/JohnEarnest/c-octo/blob/main/src/octo_emulator.h).
/// These font characters are 8x10 (two classic sprites stacked vertically).
#[rustfmt::skip]
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
    0x7E, 0xFF, 0xC3, 0xC3, 0xC3, 0xFF, 0xFF, 0xC3, 0xC3, 0xC3, // A 
    0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, 0xC3, 0xC3, 0xFC, 0xFC, // B
    0x3C, 0xFF, 0xC3, 0xC0, 0xC0, 0xC0, 0xC0, 0xC3, 0xFF, 0x3C, // C
    0xFC, 0xFE, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xC3, 0xFE, 0xFC, // D
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, // E
    0xFF, 0xFF, 0xC0, 0xC0, 0xFF, 0xFF, 0xC0, 0xC0, 0xC0, 0xC0  // F
];
/// Refresh rate, number of frames per second.
/// Also dictates the decrease rate of the emulator's timers.
pub const REFRESH_RATE_HZ: usize = 60;
/// Number of RPL flags registers. 8 on the HP48, 16 on XO-Chip.
const RPL_REGISTERS_COUNT: usize = 16;
/// Number of memory bytes read by CPU at each cycle.
const PROGRAM_COUNTER_STEP: u16 = 2;

/// Maximum display width, used by Super-chip and XO-chip.
pub const DISPLAY_WIDTH: usize = 128;
/// Maximum display height, used by Super-chip and XO-chip.
pub const DISPLAY_HEIGHT: usize = 64;
/// The value of pixel not set, when not in XO-Chip.
pub const PIXEL_OFF: u8 = 0x00;
/// The value of pixel set, when not in XO-Chip.
pub const PIXEL_ON: u8 = 0xFF;
/// The value to add to a pixel to get the next value, on XO-Chip.
/// For 2 display planes, this yields 85 : 0, 85 170, 255.
pub const PIXEL_STEP: u8 = repeat_bits(1, DISPLAY_PLANES);
/// Number of display planes used by XO-Chip.
const DISPLAY_PLANES: usize = 2;
/// Mask of relevant bits in plane selection
const PLANES_MASK: u8 = (1 << DISPLAY_PLANES as u8) - 1;
/// Number of bytes for the audio pattern buffer on XO-Chip.
const AUDIO_BUFFER_SIZE: usize = 16;

// Create type aliases depending on if the heap is available or not.
// cfg_if is not used here in order to provide type hints in IDEs.

#[cfg(feature = "alloc")]
pub type DisplayBuffer = alloc::vec::Vec<alloc::vec::Vec<u8>>;
#[cfg(not(feature = "alloc"))]
pub type DisplayBuffer = [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

#[cfg(feature = "alloc")]
type Ram = alloc::vec::Vec<u8>;
#[cfg(not(feature = "alloc"))]
type Ram = [u8; RAM_SIZE];

/// Repeats the `count` least-significant bits of `value` on following bits.
/// See [test::test_repeat_bits].
#[inline]
const fn repeat_bits(value: u8, count: usize) -> u8 {
    let step = u8::MAX as u8 / ((1 << count) - 1);
    let mask = (1 << count) - 1;
    (value & mask).wrapping_mul(step)
}

/// The mode in which the emulator runs, affects the display size and the
/// way some instruction are handled.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
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
    /// Octo XO-Chip extension from 2014. Uses 4-color 128x64 display.
    XOChip,
    // Should be implemented :
    // Chip48
    // SuperChip1_0
}

/// Chip-8 Emulator.
pub struct Chirp8 {
    /// Memory of interpreter.
    ram: Ram,
    /// Display buffer, true when pixel is on, false otherwise.
    display_buffer: DisplayBuffer,
    /// V0 to VF.
    registers: [u8; REGISTERS_COUNT],
    /// Program counter.
    pc: u16,
    /// Index register, "I".
    index: u16,
    /// Stack used for calling subroutines.
    stack: Stack<u16, STACK_SIZE>,
    /// Sound timer, sound is on when non zero.
    sound_timer: u8,
    /// Delay timer used by programs to keep count of time.
    delay_timer: u8,
    /// Persistent RPL flags registers.
    rpl_registers: [u8; RPL_REGISTERS_COUNT],
    /// The audio buffer of XO-Chip.
    audio_buffer: [u8; AUDIO_BUFFER_SIZE],
    /// The pitch buffer, each bit of the audio buffer is played at a rate of 4000*2^((pitch-64)/48).
    pitch: u8,

    /// Each key is set to true whe pressed and false when released.
    keys: [bool; KEYS_COUNT as usize],
    /// The keys state at the last cpu state. Used to know when a key is just pressed or released.
    keys_previous: [bool; KEYS_COUNT as usize],
    /// On Super Chip 8 and above, true when high-resolution is enabled.
    high_resolution: bool,
    /// On XO-Chip, bit-mask of the selected planes.
    /// All bits are used ! For P planes (2 on XO-Chip), these bits are repeated every P bits :
    /// - P=1 : [0b00000000, 0b11111111]
    /// - P=2 : [0b00_00_00_00, 0b01_01_01_01, 0b10_10_10_10, 0b11_11_11_11]
    /// - P=4 : [0b0000_0000, 0b0001_0001, 0b0010_0010, 0b0011_0011, 0b0100_0100, ..., 0b1110_1110, 0b1111_1111]
    /// This allows for 2^P values equally distant and filling all 0..255 range.
    plane_selection: u8,

    /// The current running mode of the emulator.
    mode: Chirp8Mode,
    /// The enabled quirks of the emulator.
    quirks: QuirkFlags,
    /// Number of cpu steps taken since last timer step.
    steps_since_frame: usize,
    /// Meta flag to indicate that the display changed.
    display_changed: bool,
    /// Random numbers generator.
    randomizer: SmallRng,
    /// Number of taken steps. This is not incremented if the interpreter is idle.
    steps: usize,
    /// Number of CPU steps executed between two consecutive frames.
    /// Also dictates the number of steps between two timer decreases.
    steps_per_frame: usize,
}

impl Default for Chirp8 {
    fn default() -> Self {
        Self::new(Chirp8Mode::CosmacChip8)
    }
}

impl Chirp8 {
    /// Creates a new emulator, which will behave according to given `mode`.
    pub fn new(mode: Chirp8Mode) -> Self {
        Chirp8::with_custom_quirks(mode, QuirkFlags::from_mode(mode))
    }

    /// Creates a new emulator, which will behave according to given `mode` and with custom quirks
    /// behavior.
    pub fn with_custom_quirks(mode: Chirp8Mode, quirks: QuirkFlags) -> Self {
        // Create RAM and display buffer
        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")]{
                let mut ram = alloc::vec![0u8; RAM_SIZE];
                let display_buffer = alloc::vec![alloc::vec![PIXEL_OFF; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
            }else{
                let mut ram = [0u8; RAM_SIZE];
                let display_buffer = [[PIXEL_OFF; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
            }
        }

        // Load font to RAM
        const FONT_SPRITES_SIZE: usize = FONT_SPRITES_COUNT * FONT_SPRITES_STEP;
        ram[FONT_SPRITES_ADDRESS..FONT_SPRITES_ADDRESS + FONT_SPRITES_SIZE]
            .copy_from_slice(&FONT_SPRITES);
        const FONT_SPRITES_HIGH_SIZE: usize = FONT_SPRITES_COUNT * FONT_SPRITES_HIGH_STEP;
        ram[FONT_SPRITES_HIGH_ADDRESS..FONT_SPRITES_HIGH_ADDRESS + FONT_SPRITES_HIGH_SIZE]
            .copy_from_slice(&FONT_SPRITES_HIGH);

        let steps_per_frame = match mode {
            Chirp8Mode::CosmacChip8 => 10,
            Chirp8Mode::SuperChip1_1 => 30,
            Chirp8Mode::SuperChipModern => 30,
            Chirp8Mode::XOChip => 30,
        };

        let plane_selection = if mode == Chirp8Mode::XOChip {
            // First plane selected, repeated 4 times.
            repeat_bits(0b01, DISPLAY_PLANES)
        } else {
            // Draw on all planes.
            repeat_bits(1, 1)
        };

        // Fill audio buffer with 128-samples long square wave. (8x16)
        // Played at a rate of 4000 Hz, this yields a frequency of 31.25 Hz
        let mut audio_buffer = [0; AUDIO_BUFFER_SIZE];
        audio_buffer
            .split_at_mut(AUDIO_BUFFER_SIZE / 2)
            .1
            .fill(0xFF);

        let mut randomizer = SmallRng::seed_from_u64(0xDEADCAFEDEADCAFE);

        if quirks.contains(QuirkFlags::RAM_RANDOM) {
            ram[PROGRAM_START..(PROGRAM_START + PROGRAM_SIZE)]
                .fill_with(|| randomizer.next_u32() as u8);
        }

        // Create emulator
        Self {
            ram: ram,
            display_buffer: display_buffer,
            registers: [0; REGISTERS_COUNT],
            pc: PROGRAM_START as u16,
            index: 0,
            stack: Stack::new(),
            sound_timer: 0,
            delay_timer: 0,
            rpl_registers: [0; RPL_REGISTERS_COUNT],
            audio_buffer: audio_buffer,
            pitch: 0,
            keys: [false; KEYS_COUNT as usize],
            keys_previous: [false; KEYS_COUNT as usize],
            high_resolution: false,
            plane_selection: plane_selection,
            mode: mode,
            quirks: quirks,
            steps_since_frame: 0,
            display_changed: true,
            randomizer: randomizer,
            steps: 0,
            steps_per_frame: steps_per_frame,
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
        // Do-while
        loop {
            self.step();
            if self.steps_since_frame == 0 {
                break;
            }
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
            row.fill(PIXEL_OFF);
        }
    }

    /// Forces the interpreter to take given number of `steps`.
    /// `step()` may be called more times than `steps` parameter, due to interpreter being idle in certain conditions.
    /// In most cases, do not use this method, prefer `run_frame` or just `step`.
    pub fn take_steps(&mut self, steps: usize) {
        let target_steps = self.steps.wrapping_add(steps);
        while self.steps != target_steps {
            self.step();
        }
    }

    /// Execute one machine instruction, decrement timers if necessary.
    /// If the interpreter is in idle, if waiting for an interrupt for instance, the step is not taken,
    /// which is to say the `steps` counter is not incremented.
    pub fn step(&mut self) {
        // Big endian instruction
        let instruction = self.next_instruction();
        self.pc = self.pc.wrapping_add(PROGRAM_COUNTER_STEP) & RAM_MASK;
        self.steps = self.steps.wrapping_add(1);

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

        // TODO : make so that modes that do not know instructions treat them as NOP (example : chip 8 ignores scroll).

        match opcode {
            0x0 => match nn {
                // Clear screen
                0xE0 => {
                    if self.mode != Chirp8Mode::XOChip {
                        self.clear_display();
                    } else {
                        self.clear_planes();
                    }
                }
                // Return from subroutine
                0xEE => self.pc = self.stack.pop().ok().unwrap(),
                // Exit from interpreter (Super-Chip)
                0xFD => {
                    if self.mode >= Chirp8Mode::SuperChip1_1 {
                        self.reset()
                    } else {
                        self.print_unknown_instruction(instruction)
                    }
                }
                // Disable High-res (Super-Chip and above)
                0xFE => {
                    if self.mode >= Chirp8Mode::SuperChip1_1 {
                        self.high_resolution = false;
                        if self.quirks.contains(QuirkFlags::CLEAR_ON_RES) {
                            self.clear_display();
                        }
                    } else {
                        self.print_unknown_instruction(instruction)
                    }
                }
                // Enable High-res (Super-chip and above)
                0xFF => {
                    if self.mode >= Chirp8Mode::SuperChip1_1 {
                        self.high_resolution = true;
                        if self.quirks.contains(QuirkFlags::CLEAR_ON_RES) {
                            self.clear_display();
                        }
                    } else {
                        self.print_unknown_instruction(instruction)
                    }
                }
                // Scroll up N pixels (XO-Chip)
                0xD0..=0xDF => {
                    if self.mode == Chirp8Mode::XOChip {
                        self.scroll_up(n)
                    } else {
                        self.print_unknown_instruction(instruction)
                    }
                }
                // Scroll up N pixels (Unofficial Super Chip)
                0xB0..=0xBF => {
                    if self.mode == Chirp8Mode::SuperChipModern {
                        self.scroll_up(n)
                    } else {
                        self.print_unknown_instruction(instruction)
                    }
                }
                // Scroll down N pixels (Super Chip and above)
                0xC0..=0xCF => {
                    if self.mode >= Chirp8Mode::SuperChip1_1 {
                        self.scroll_down(n)
                    } else {
                        self.print_unknown_instruction(instruction)
                    }
                }
                // Scroll right 4 pixels (Super Chip and above)
                0xFB => {
                    if self.mode >= Chirp8Mode::SuperChip1_1 {
                        self.scroll_right(4)
                    } else {
                        self.print_unknown_instruction(instruction)
                    }
                }
                // Scroll left 4 pixels (Super Chip and above)
                0xFC => {
                    if self.mode >= Chirp8Mode::SuperChip1_1 {
                        self.scroll_left(4)
                    } else {
                        self.print_unknown_instruction(instruction)
                    }
                }
                _ => self.print_unknown_instruction(instruction),
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
                    self.skip_next_instruction();
                }
            }
            // Skip
            0x4 => {
                if self.registers[x] != nn {
                    self.skip_next_instruction();
                }
            }
            0x5 => {
                match n {
                    // 0x5XY0 : Skip if vx == vy
                    0 => {
                        if self.registers[x] == self.registers[y] {
                            self.skip_next_instruction();
                        }
                    }
                    // 0x5XY2 : Save vx - vy (XO-chip)
                    2 => {
                        if self.mode == Chirp8Mode::XOChip {
                            if x < y {
                                let end = self.index as usize + y - x;
                                self.ram[self.index as usize..=end]
                                    .copy_from_slice(&self.registers[x..=y]);
                            } else {
                                let end = self.index as usize + x - y;
                                self.ram[self.index as usize..=end]
                                    .copy_from_slice(&self.registers[y..=x]);
                                self.ram[self.index as usize..=end].reverse();
                            }
                        } else {
                            self.print_unknown_instruction(instruction)
                        }
                    }
                    // 0x5XY3 : Load vx - vy (XO-chip)
                    3 => {
                        if self.mode == Chirp8Mode::XOChip {
                            if x < y {
                                let end = self.index as usize + y - x;
                                self.registers[x..=y]
                                    .copy_from_slice(&self.ram[self.index as usize..=end]);
                            } else {
                                let end = self.index as usize + x - y;
                                self.registers[y..=x]
                                    .copy_from_slice(&self.ram[self.index as usize..=end]);
                                self.registers[y..=x].reverse();
                            }
                        } else {
                            self.print_unknown_instruction(instruction)
                        }
                    }
                    _ => self.print_unknown_instruction(instruction),
                }
            }
            // Skip
            0x9 => {
                // n should be equal to 0 (0x9XY0), not checked for performance.
                if self.registers[x] != self.registers[y] {
                    self.skip_next_instruction();
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
                    if self.quirks.contains(QuirkFlags::FLAG_RESET) {
                        self.reset_flag();
                    }
                }
                // AND
                0x2 => {
                    self.registers[x] &= self.registers[y];
                    if self.quirks.contains(QuirkFlags::FLAG_RESET) {
                        self.reset_flag();
                    }
                }
                // XOR
                0x3 => {
                    self.registers[x] ^= self.registers[y];
                    if self.quirks.contains(QuirkFlags::FLAG_RESET) {
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
                    self.registers[FLAG_REGISTER_INDEX] = flag;
                }
                // SUB VX - VY
                0x5 => {
                    let flag = if self.registers[x] >= self.registers[y] {
                        1
                    } else {
                        0
                    };
                    self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
                    self.registers[FLAG_REGISTER_INDEX] = flag;
                }
                // Shift VX right
                0x6 => {
                    if !self.quirks.contains(QuirkFlags::SHIFT_X_ONLY) {
                        self.registers[x] = self.registers[y];
                    }
                    let flag = self.registers[x] & 0x1;
                    self.registers[x] >>= 1;
                    self.registers[FLAG_REGISTER_INDEX] = flag;
                }
                // SUB VY - VX
                0x7 => {
                    let flag = if self.registers[y] >= self.registers[x] {
                        1
                    } else {
                        0
                    };
                    self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
                    self.registers[FLAG_REGISTER_INDEX] = flag;
                }
                // Shift VX left
                0xE => {
                    if !self.quirks.contains(QuirkFlags::SHIFT_X_ONLY) {
                        self.registers[x] = self.registers[y];
                    }
                    let flag = (self.registers[x] >> 7) & 0x1;
                    self.registers[x] <<= 1;
                    self.registers[FLAG_REGISTER_INDEX] = flag;
                }
                _ => self.print_unknown_instruction(instruction),
            },
            // Set index
            0xA => self.index = nnn,
            // Jump with offset
            0xB => {
                self.pc = (nnn
                    + self.registers[if self.quirks.contains(QuirkFlags::JUMP_XNN) {
                        x
                    } else {
                        0
                    }] as u16)
                    & RAM_MASK;
            }
            // Random
            0xC => self.registers[x] = (self.randomizer.next_u32() as u8) & nn,
            // Display
            0xD => {
                // Handle the "display wait" quirk. If enabled, the CPU waits for the next v-blank interrupt,
                // so the step is not taken and the program counter is not incremented.
                // This quirk is only enabled on original Chip 8 and low-resolution (low-speed) super-chip.
                // See : https://github.com/Timendus/chip8-test-suite/blob/main/legacy-superchip.md
                let wait_enabled = if self.high_resolution {
                    self.quirks.contains(QuirkFlags::DISPLAY_WAIT_HIRES)
                } else {
                    self.quirks.contains(QuirkFlags::DISPLAY_WAIT_LORES)
                };

                if wait_enabled {
                    if self.steps_since_frame != 0 {
                        self.pc = self.pc.wrapping_sub(PROGRAM_COUNTER_STEP) & RAM_MASK;
                        self.steps = self.steps.wrapping_sub(1);
                    } else {
                        self.handle_display_instruction((self.registers[x], self.registers[y]), n);
                    }
                } else {
                    self.handle_display_instruction((self.registers[x], self.registers[y]), n);
                }
            }
            // Skip if key
            0xE => match nn {
                // Skip if VX pressed
                0x9E => {
                    let key = (0xF & self.registers[x]) as usize;
                    if self.keys[key] {
                        self.skip_next_instruction();
                    }
                }
                // Skip if VX not pressed
                0xA1 => {
                    let key = (0xF & self.registers[x]) as usize;
                    if !self.keys[key] {
                        self.skip_next_instruction();
                    }
                }
                _ => self.print_unknown_instruction(instruction),
            },
            0xF => {
                match nn {
                    // F000 : Load 16-bits address in index (XO-Chip)
                    0x00 => {
                        if self.mode == Chirp8Mode::XOChip {
                            if self.mode == Chirp8Mode::XOChip && x == 0 {
                                // The next "instruction" is actually a 16-bits address
                                self.index = self.next_instruction();
                                self.pc = self.pc.wrapping_add(PROGRAM_COUNTER_STEP);
                            } else {
                                self.print_unknown_instruction(instruction);
                            }
                        } else {
                            self.print_unknown_instruction(instruction)
                        }
                    }
                    // FX01 Plane, select plane(s) X (XO-Chip)
                    0x01 => {
                        if self.mode == Chirp8Mode::XOChip {
                            self.plane_selection = repeat_bits(x as u8, DISPLAY_PLANES)
                        } else {
                            self.print_unknown_instruction(instruction)
                        }
                    }

                    // Timers set VX
                    0x07 => self.registers[x] = self.delay_timer,
                    0x15 => self.delay_timer = self.registers[x],
                    0x18 => self.sound_timer = self.registers[x],
                    // Add to index
                    0x1E => {
                        if cfg!(feature = "mem_extend") {
                            // Check 16-bits overflow
                            if let Some(result) = self.index.checked_add(self.registers[x] as u16) {
                                self.index = result;
                            } else {
                                self.index = self.index.wrapping_add(self.registers[x] as u16);
                                self.set_flag();
                            }
                        } else {
                            self.index = self.index + self.registers[x] as u16;
                            // Check 12-bits overflow
                            if self.index & !RAM_MASK != 0 {
                                self.set_flag();
                                self.index &= RAM_MASK;
                            }
                        }
                    }
                    // Get Key
                    0x0A => {
                        if let Option::Some(key) = self.get_first_key_released() {
                            self.registers[x] = key;
                        } else {
                            self.pc = self.pc.wrapping_sub(PROGRAM_COUNTER_STEP);
                        }
                    }
                    // FX29: Font character
                    0x29 => {
                        // Not implemented : SuperChip1.0 : Point I to 5-byte font sprite as in CHIP-8,
                        // but if the high nibble in VX is 1 (ie. for values between 10 and 19 in hex) it will
                        // point I to a 10-byte font sprite for the digit in the lower nibble of VX (only digits 0-9).
                        // The following is the SuperChip1.1 behavior.
                        self.index = FONT_SPRITES_ADDRESS as u16
                            + FONT_SPRITES_STEP as u16 * self.registers[x] as u16;
                    }
                    // FX30: Large font character (Super-Chip 1.1 and above)
                    0x30 => {
                        if self.mode >= Chirp8Mode::SuperChip1_1 {
                            self.index = FONT_SPRITES_HIGH_ADDRESS as u16
                                + FONT_SPRITES_HIGH_STEP as u16 * self.registers[x] as u16;
                        } else {
                            self.print_unknown_instruction(instruction)
                        }
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
                        let end_index = (x + 1) as u16;
                        for i in 0..end_index {
                            self.ram[((self.index.wrapping_add(i)) & RAM_MASK) as usize] =
                                self.registers[i as usize];
                        }
                        // if mode == SuperChip1.0 self.index = (self.index + (end_index as u16) - 1) & RAM_MASK;
                        if self.quirks.contains(QuirkFlags::INC_INDEX) {
                            self.index = (self.index.wrapping_add(end_index as u16)) & RAM_MASK;
                        }
                    }
                    // FX65: Load
                    0x65 => {
                        let end_index = (x + 1) as u16;
                        for i in 0..end_index {
                            self.registers[i as usize] =
                                self.ram[((self.index.wrapping_add(i)) & RAM_MASK) as usize];
                        }
                        // if mode == SuperChip1.0 self.index = (self.index + (end_index as u16) - 1) & RAM_MASK;
                        if self.quirks.contains(QuirkFlags::INC_INDEX) {
                            self.index = (self.index.wrapping_add(end_index as u16)) & RAM_MASK;
                        }
                    }
                    // FX75 : Save to flags registers (Super-Chip 1.0 and above)
                    0x75 => {
                        if self.mode >= Chirp8Mode::SuperChip1_1 {
                            let count = if self.mode == Chirp8Mode::XOChip {
                                x
                            } else {
                                x & 0x7
                            };
                            self.rpl_registers[0..count].copy_from_slice(&self.registers[0..count]);
                        } else {
                            self.print_unknown_instruction(instruction)
                        }
                    }
                    // FX85 : Load from flags registers (Super-Chip 1.0 and above)
                    0x85 => {
                        if self.mode >= Chirp8Mode::SuperChip1_1 {
                            let count = if self.mode == Chirp8Mode::XOChip {
                                x
                            } else {
                                x & 0x7
                            };
                            self.registers[0..count].copy_from_slice(&self.rpl_registers[0..count]);
                        } else {
                            self.print_unknown_instruction(instruction)
                        }
                    }
                    _ => self.print_unknown_instruction(instruction),
                }
            }

            _ => self.print_unknown_instruction(instruction),
        }
        // Handle timers
        self.step_timers();
        // Handle keys
        self.keys_previous.copy_from_slice(&self.keys);
    }

    /// Tick timers by one machine cycle, and update them accordingly.
    fn step_timers(&mut self) {
        self.steps_since_frame += 1;
        if self.steps_since_frame >= self.steps_per_frame {
            self.steps_since_frame = 0;
            self.delay_timer = self.delay_timer.saturating_sub(1);
            self.sound_timer = self.sound_timer.saturating_sub(1);
        }
    }

    /// Increments program counter so that the next instruction is skipped.
    fn skip_next_instruction(&mut self) {
        const LOAD_LARGE_INDEX_OPCODE: u16 = 0xF000;
        let offset = if self.mode == Chirp8Mode::XOChip
            && self.next_instruction() == LOAD_LARGE_INDEX_OPCODE
        {
            // Jump over 4 bytes instructions
            PROGRAM_COUNTER_STEP * 2
        } else {
            PROGRAM_COUNTER_STEP
        };
        self.pc = self.pc.wrapping_add(offset) & RAM_MASK;
    }

    /// Modifies the number of CPU steps executed between each frame.
    pub fn set_steps_per_frame(&mut self, steps: usize) {
        while self.steps_since_frame != 0 {
            self.step()
        }
        self.steps_per_frame = steps;
    }

    #[allow(unused_variables)]
    fn print_unknown_instruction(&self, instruction: u16) {
        #[cfg(feature = "std")]
        {
            let message = alloc::format!(
                "Unknown instruction 0x{:04X} in mode '{}', at program counter 0x{:04X} {}.",
                instruction,
                match self.mode {
                    Chirp8Mode::CosmacChip8 => "Chip-8",
                    Chirp8Mode::SuperChip1_1 => "Super Chip 1.1",
                    Chirp8Mode::SuperChipModern => "Super Chip Modern",
                    Chirp8Mode::XOChip => "XO-Chip",
                },
                self.pc.wrapping_sub(PROGRAM_COUNTER_STEP),
                if let Option::Some(address) = self
                    .pc
                    .wrapping_sub(PROGRAM_COUNTER_STEP)
                    .checked_sub(PROGRAM_START as u16)
                {
                    alloc::format!("(At program address 0x{:04X})", address)
                } else {
                    alloc::string::String::from("(Lost in reserved memory < 0x0200)")
                }
            );
            std::println!("{}", message);
        }
    }

    #[inline]
    fn set_flag(&mut self) {
        self.registers[FLAG_REGISTER_INDEX] = 1;
    }

    #[inline]
    fn reset_flag(&mut self) {
        self.registers[FLAG_REGISTER_INDEX] = 0;
    }

    /// Returns the first key just released, between 0 and 15 included, or `Option::None` when nothing has changed.
    fn get_first_key_released(&self) -> Option<u8> {
        for (index, (key, key_previous)) in
            self.keys.iter().zip(self.keys_previous.iter()).enumerate()
        {
            if *key_previous && !*key {
                return Option::Some(index as u8);
            }
        }
        Option::None
    }

    /// Clears the screen.
    fn clear_display(&mut self) {
        for row in &mut self.display_buffer {
            row.fill(PIXEL_OFF);
        }
    }

    /// Clears the selected screen planes.
    fn clear_planes(&mut self) {
        if self.plane_selection & PLANES_MASK == PLANES_MASK {
            self.clear_display();
        } else {
            for plane in 0..DISPLAY_PLANES {
                let plane_mask = repeat_bits(1 << plane, DISPLAY_PLANES);
                if plane_mask & self.plane_selection != 0 {
                    for row in &mut self.display_buffer {
                        for pixel in row {
                            *pixel &= !plane_mask;
                        }
                    }
                }
            }
        }
    }

    /// Displays sprite on screen.
    fn display_sprite(
        &mut self,
        x_y_coordinates: (u8, u8),
        height: u8,
        colliding_rows_quirk: bool,
    ) {
        // Maximum input coordinates
        let (max_width, max_height, coordinates_scaler) = if self.high_resolution {
            (DISPLAY_WIDTH, DISPLAY_HEIGHT, 1)
        } else {
            (DISPLAY_WIDTH / 2, DISPLAY_HEIGHT / 2, 2)
        };

        let x_y_coordinates = (
            x_y_coordinates.0 % max_width as u8,
            x_y_coordinates.1 % max_height as u8,
        );

        // Number of color planes
        let planes_count = if self.quirks.contains(QuirkFlags::USE_SEVERAL_PLANES) {
            DISPLAY_PLANES
        } else {
            1
        };
        // Do sprites wrap around screen edges.
        let wrapping = !self.quirks.contains(if self.high_resolution {
            QuirkFlags::CLIP_SPRITES_HIRES
        } else {
            QuirkFlags::CLIP_SPRITES_LORES
        });

        // The number of planes drawn so far.
        let mut drawn_planes = 0;
        for plane in 0..planes_count {
            // if 2 planes, masks of plane 0 and 1 are 0b01_01_01_01 and 0b10_10_10_10
            // if 1 planes, mask of only plane is 0xFF
            let pixel_bits_mask = repeat_bits(1 << plane, planes_count);
            if self.plane_selection & pixel_bits_mask == 0 {
                continue;
            }
            for line in 0..(height as usize) {
                let sprite_address = (self
                    .index
                    .wrapping_add((height as u16) * (drawn_planes as u16))//Plane offset
                    .wrapping_add(line as u16) // Line offset
                    & RAM_MASK) as usize;
                let sprite = self.ram[sprite_address];
                let row = ((x_y_coordinates.1 as usize) + line) * coordinates_scaler;

                // Handle line clipping / wrapping
                if row >= DISPLAY_HEIGHT && !wrapping {
                    if colliding_rows_quirk {
                        self.registers[FLAG_REGISTER_INDEX] += 1;
                        continue;
                    }
                    break;
                }
                let row = row % DISPLAY_HEIGHT;

                let mut colliding_line = false;
                for bit in 0..(u8::BITS as usize) {
                    let col = (x_y_coordinates.0 as usize + bit) * coordinates_scaler;

                    // Handle width clipping / wrapping
                    if col >= DISPLAY_WIDTH && !wrapping {
                        break;
                    }
                    let col = col % DISPLAY_WIDTH;

                    // Should the pixel be flipped or not.
                    let pixel_bits_xor = if ((sprite >> ((u8::BITS as usize) - 1 - bit)) & 1) == 0 {
                        0x00
                    } else {
                        pixel_bits_mask
                    };

                    let pixel_before = self.display_buffer[row][col];
                    let mut pixel = pixel_before;
                    pixel ^= pixel_bits_xor;
                    self.display_buffer[row][col] = pixel;
                    if !self.high_resolution {
                        // Draw 2x2 "pixels" when on low resolution
                        self.display_buffer[row][col + 1] = pixel;
                        self.display_buffer[row + 1][col] = pixel;
                        self.display_buffer[row + 1][col + 1] = pixel;
                    }
                    // Set flag when turned off
                    colliding_line |=
                        pixel_before & pixel_bits_mask != 0 && pixel & pixel_bits_mask == 0;
                }
                if colliding_line {
                    self.registers[FLAG_REGISTER_INDEX] += 1;
                }
            }
            drawn_planes += 1;
        }
    }

    /// Display large 16 by 16 sprite.
    fn display_large_sprite(&mut self, x_y_coordinates: (u8, u8), colliding_rows_quirk: bool) {
        /// Width and Height of large sprites.
        const LARGE_SPRITE_SIZE: usize = 16;
        /// Bytes per line for large sprites.
        const BYTES_PER_LINE: u16 = 2;

        // Number of color planes
        let planes_count = if self.quirks.contains(QuirkFlags::USE_SEVERAL_PLANES) {
            DISPLAY_PLANES
        } else {
            1
        };
        // Do sprites wrap around screen edges.
        let wrapping = !self.quirks.contains(if self.high_resolution {
            QuirkFlags::CLIP_SPRITES_HIRES
        } else {
            QuirkFlags::CLIP_SPRITES_LORES
        });

        // In SChip mode, VF is set to the number of colliding rows, not just 0 or 1.
        // Although disabled on XO-chip, this quirk is handled as VF being the number of colliding rows on all planes.

        // The number of planes drawn so far.
        let mut drawn_planes = 0;
        for plane in 0..planes_count {
            // if 2 planes, masks of plane 0 and 1 are 0b01_01_01_01 and 0b10_10_10_10
            // if 1 planes, mask of only plane is 0xFF
            let pixel_bits_mask = repeat_bits(1 << plane, planes_count);
            if self.plane_selection & pixel_bits_mask == 0 {
                continue;
            }
            for line in 0..LARGE_SPRITE_SIZE {
                let row = (x_y_coordinates.1 as usize % DISPLAY_HEIGHT) + line;

                // Handle line clipping / wrapping
                if row >= DISPLAY_HEIGHT && !wrapping {
                    if colliding_rows_quirk {
                        self.registers[FLAG_REGISTER_INDEX] += 1;
                        continue;
                    }
                    break;
                }
                let row = row % DISPLAY_HEIGHT;

                let mut colliding_line = false;
                for half in 0..BYTES_PER_LINE {
                    let sprite_address = (self
                        .index
                        .wrapping_add(BYTES_PER_LINE * (LARGE_SPRITE_SIZE as u16) * drawn_planes)
                        .wrapping_add(BYTES_PER_LINE * (line as u16))
                        .wrapping_add(half)
                        & RAM_MASK) as usize;
                    let sprite = self.ram[sprite_address];

                    for bit in 0..(min(
                        u8::BITS as usize,
                        LARGE_SPRITE_SIZE - (half as usize) * (u8::BITS as usize),
                    )) {
                        let col = x_y_coordinates.0 as usize % DISPLAY_WIDTH
                            + (half as usize) * (u8::BITS as usize)
                            + bit;

                        // Handle width clipping / wrapping
                        if col >= DISPLAY_WIDTH && !wrapping {
                            break;
                        }
                        let col = col % DISPLAY_WIDTH;

                        // Should the pixel be flipped or not
                        let pixel_bits_xor =
                            if ((sprite >> ((u8::BITS as usize) - 1 - bit)) & 1) == 0 {
                                0x00
                            } else {
                                pixel_bits_mask
                            };

                        let pixel = &mut self.display_buffer[row][col];
                        let pixel_before = *pixel;
                        *pixel ^= pixel_bits_xor;
                        // Set flag when turned off
                        colliding_line |= (pixel_before & pixel_bits_mask) != 0
                            && (*pixel & pixel_bits_mask) == 0;
                    }
                }
                if colliding_line {
                    self.registers[FLAG_REGISTER_INDEX] += 1;
                }
            }
            drawn_planes += 1;
        }
    }

    /// Display `height`-pixel tall sprite pointed by index register at given `x_y_coordinates`.
    /// If `height` is 0 then a large 16x16 sprite is used.
    /// On XO-Chip, can dray on different planes.
    fn handle_display_instruction(&mut self, x_y_coordinates: (u8, u8), height: u8) {
        self.display_changed = true;
        self.reset_flag();

        // High resolution does not exist on original chip 8.
        let high_resolution = self.mode != Chirp8Mode::CosmacChip8 && self.high_resolution;

        // On Super-chip, height of 0 indicates a large sprite in hires only.
        let large_sprite = if self.mode != Chirp8Mode::XOChip {
            high_resolution && height == 0
        } else {
            height == 0
        };

        // VF counts the number of colliding rows instead of just being set to 0 or 1.
        let colliding_rows_quirk = self.quirks.contains(if self.high_resolution {
            QuirkFlags::COLLISION_COUNT_HIRES
        } else {
            QuirkFlags::COLLISION_COUNT_HIRES
        });

        if large_sprite {
            // Handle instruction DXY0 : display 16x16 sprite (height is 16, not 0)
            self.display_large_sprite(x_y_coordinates, colliding_rows_quirk);
        } else {
            // Handle instruction DXYN : display 8xN sprite
            self.display_sprite(x_y_coordinates, height, colliding_rows_quirk);
        }

        // Saturate flag to 1 if no colliding flag quirk
        if !colliding_rows_quirk && self.registers[FLAG_REGISTER_INDEX] != 0 {
            self.registers[FLAG_REGISTER_INDEX] = 1;
        }
    }

    /// Indicates if the display changed since the last time this method was called.
    pub fn display_changed(&mut self) -> bool {
        let result = self.display_changed;
        self.display_changed = false;
        result
    }

    /// Scrolls up display by `scroll` pixels.
    fn scroll_up(&mut self, scroll: u8) {
        // mode == Cosmac Chip 8 is not checked, should not happen.
        let scroll =
            if !self.quirks.contains(QuirkFlags::SCROLL_HALF_PIXEL) && !self.high_resolution {
                scroll * 2
            } else {
                scroll
            } as usize;
        self.display_buffer.rotate_left(scroll);
        // Bottom of screen is black.
        for black_row in &mut self.display_buffer[(DISPLAY_HEIGHT - scroll)..DISPLAY_HEIGHT] {
            black_row.fill(PIXEL_OFF);
        }
    }

    /// Scrolls down display by `scroll` pixels.
    fn scroll_down(&mut self, scroll: u8) {
        // mode == Cosmac Chip 8 is not checked, should not happen.
        let scroll =
            if !self.quirks.contains(QuirkFlags::SCROLL_HALF_PIXEL) && !self.high_resolution {
                scroll * 2
            } else {
                scroll
            } as usize;
        self.display_buffer.rotate_right(scroll);
        // Top of screen is black.
        for black_row in &mut self.display_buffer[0..scroll] {
            black_row.fill(PIXEL_OFF);
        }
    }

    /// Scrolls left display by `scroll` pixels.
    fn scroll_left(&mut self, scroll: u8) {
        let scroll =
            if !self.quirks.contains(QuirkFlags::SCROLL_HALF_PIXEL) && !self.high_resolution {
                scroll * 2
            } else {
                scroll
            } as usize;
        for row in &mut self.display_buffer {
            row.rotate_left(scroll);
            row[(DISPLAY_WIDTH - scroll)..DISPLAY_WIDTH].fill(PIXEL_OFF);
        }
    }

    /// Scrolls right display by `scroll` pixels.
    fn scroll_right(&mut self, scroll: u8) {
        let scroll =
            if !self.quirks.contains(QuirkFlags::SCROLL_HALF_PIXEL) && !self.high_resolution {
                scroll * 2
            } else {
                scroll
            } as usize;
        for row in &mut self.display_buffer {
            row.rotate_right(scroll);
            row[0..scroll].fill(PIXEL_OFF);
        }
    }

    /// Indicates whether the sound buzzer is currently on or not.
    /// On XO-Chip, indicates that the audio buffer is being played.
    pub fn is_sounding(&self) -> bool {
        self.sound_timer > 0
    }

    /// Indicates whether the emulator has complex sound waves, given by [Chirp8::get_audio_buffer] (true),
    /// or only simple buzzing sounds (false).
    pub fn has_sound_wave(&self) -> bool {
        self.mode == Chirp8Mode::XOChip
    }

    /// Load a ROM into memory. The ROM must be smaller than `PROGRAM_SIZE`.
    /// Returns true if the ROM has been loaded to RAM, false otherwise.
    pub fn load_rom(&mut self, rom: &[u8]) -> bool {
        if rom.len() < PROGRAM_SIZE {
            self.ram[PROGRAM_START..(PROGRAM_START + rom.len())].copy_from_slice(rom);
            true
        } else {
            false
        }
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
    /// in order to match the resolution of the Super-Chip / XO-Chip.
    pub fn get_display_buffer(&self) -> &DisplayBuffer {
        &self.display_buffer
    }

    /// Access the 128 1-bit samples in the audio buffer.
    pub fn get_audio_buffer(&self) -> &[u8; AUDIO_BUFFER_SIZE] {
        &self.audio_buffer
    }

    /// Returns the log2 of the audio bit rate in Hertz.
    /// Each bit in the audio buffer (128 bits) must be played at this rate.
    /// This method does not do the exponentiation in order not to use the standard library.
    /// Usage :
    /// ```
    /// let chirp = chirp8::Chirp8::new(chirp8::Chirp8Mode::XOChip);
    /// // ...
    /// let frequency = 2f32.powf(chirp.get_audio_bit_rate_log2_hz());
    /// ```
    pub fn get_audio_bit_rate_log2_hz(&self) -> f32 {
        // 4000 * 2^((pitch-64)/48)
        // = 2 ^ ( log2(4000) + ((pitch-64) / 48) )
        const LOG2_4000: f32 = 11.96578428466208704361095828846817052759449417907374183616426;

        LOG2_4000 + ((self.pitch as f32 - 64f32) / 48f32)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_repeat_bits() {
        assert_eq!(repeat_bits(1, 1), 0xFF);
        assert_eq!(repeat_bits(0b01, 2), 0b01_01_01_01);
        assert_eq!(repeat_bits(0b10, 2), 0b10_10_10_10);
        assert_eq!(repeat_bits(0b11, 2), 0xFF);
        assert_eq!(repeat_bits(0b1011, 4), 0b1011_1011);

        assert_eq!(repeat_bits(0b0101_1001, 4), 0b1001_1001);
        assert_eq!(repeat_bits(0b11_10_11_01, 2), 0b01_01_01_01);
    }

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

        assert_eq!(emulator.get_display_buffer()[45][67], PIXEL_ON);
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 0);

        emulator.pc -= 2;
        emulator.step();

        assert_eq!(emulator.get_display_buffer()[45][67], PIXEL_OFF);
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 1);
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
        emulator.display_buffer[37][67] = PIXEL_ON;
        emulator.index = PROGRAM_START as u16 + 4;
        emulator.high_resolution = true;

        emulator.step();

        assert_eq!(emulator.display_buffer[37][67], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[32][67], PIXEL_ON);

        emulator.step();

        assert_eq!(emulator.display_buffer[32][67], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[39][67], PIXEL_ON);

        emulator.pc = PROGRAM_START as u16;
        emulator.high_resolution = false;

        emulator.step();

        assert_eq!(emulator.display_buffer[39][67], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[29][67], PIXEL_ON);

        emulator.step();

        assert_eq!(emulator.display_buffer[29][67], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[43][67], PIXEL_ON);
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
        emulator.display_buffer[37][67] = PIXEL_ON;
        emulator.index = PROGRAM_START as u16 + 4;
        emulator.high_resolution = true;

        emulator.step();

        assert_eq!(emulator.display_buffer[37][67], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[37][71], PIXEL_ON);

        emulator.step();

        assert_eq!(emulator.display_buffer[37][71], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[37][67], PIXEL_ON);

        emulator.pc = PROGRAM_START as u16;
        emulator.high_resolution = false;

        emulator.step();

        assert_eq!(emulator.display_buffer[37][67], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[37][75], PIXEL_ON);

        emulator.step();

        assert_eq!(emulator.display_buffer[37][75], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[37][67], PIXEL_ON);
    }

    #[test]
    fn opcode_display_colliding_rows() {
        #[rustfmt::skip]
        let rom = [
            0xD0, 0x15, // Display v0 v1 5
            0b1000_0000, // Sprite with one pixel to the left, 5 bytes tall
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
        ];

        let mut emulator = Chirp8::new(Chirp8Mode::SuperChipModern);
        emulator.ram[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(&rom);

        emulator.index = PROGRAM_START as u16 + 2;
        emulator.high_resolution = true;

        // 2 out of bounds rows
        emulator.registers[0] = 17;
        emulator.registers[1] = 61;
        emulator.step();
        assert_eq!(emulator.display_buffer[61][17], PIXEL_ON);
        assert_eq!(emulator.display_buffer[62][17], PIXEL_ON);
        assert_eq!(emulator.display_buffer[63][17], PIXEL_ON);
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 2);

        // 3 colliding rows (61 to 63 included)
        emulator.pc = PROGRAM_START as u16;
        emulator.registers[0] = 17;
        emulator.registers[1] = 59;
        emulator.step();

        assert_eq!(emulator.display_buffer[59][17], PIXEL_ON);
        assert_eq!(emulator.display_buffer[60][17], PIXEL_ON);
        assert_eq!(emulator.display_buffer[61][17], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[62][17], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[63][17], PIXEL_OFF);
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 3);
    }

    #[test]
    fn opcode_display_colliding_rows_16_16() {
        #[rustfmt::skip]
        let rom = [
            0xD0, 0x10, // Display v0 v1 0
            0b1000_0000, 0b0000_0000, // 16x16 sprite with one pixel to the left
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
        ];

        let mut emulator = Chirp8::new(Chirp8Mode::SuperChipModern);
        emulator.ram[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(&rom);

        emulator.index = PROGRAM_START as u16 + 2;
        emulator.high_resolution = true;

        // 13 out of bounds rows
        emulator.registers[0] = 17;
        emulator.registers[1] = 61;
        emulator.step();
        assert_eq!(emulator.display_buffer[61][17], PIXEL_ON);
        assert_eq!(emulator.display_buffer[62][17], PIXEL_ON);
        assert_eq!(emulator.display_buffer[63][17], PIXEL_ON);
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 13);

        // 3 colliding rows (61 to 63 included)
        emulator.pc = PROGRAM_START as u16;
        emulator.registers[0] = 17;
        emulator.registers[1] = 48;
        emulator.step();

        assert_eq!(emulator.display_buffer[61][17], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[62][17], PIXEL_OFF);
        assert_eq!(emulator.display_buffer[63][17], PIXEL_OFF);
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 3);
    }

    #[test]
    fn opcode_save_range() {
        // 0x5XY2
        let rom = [
            0x56, 0x92, // Save v6 v9
            0x59, 0x62, // Save v9 v6
        ];

        let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
        emulator.ram[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(&rom);

        emulator.registers[6..=9].copy_from_slice(&[3, 7, 13, 59]);
        emulator.index = 0x0ABC;

        emulator.step();
        assert_eq!(emulator.ram[0xABC], 3);
        assert_eq!(emulator.ram[0xABC + 1], 7);
        assert_eq!(emulator.ram[0xABC + 2], 13);
        assert_eq!(emulator.ram[0xABC + 3], 59);
        assert_eq!(emulator.index, 0xABC);

        emulator.step();
        assert_eq!(emulator.ram[0xABC], 59);
        assert_eq!(emulator.ram[0xABC + 1], 13);
        assert_eq!(emulator.ram[0xABC + 2], 7);
        assert_eq!(emulator.ram[0xABC + 3], 3);
        assert_eq!(emulator.index, 0xABC);
    }

    #[test]
    fn opcode_load_range() {
        // 0x5XY3
        let rom = [
            0x56, 0x93, // Load v6 v9
            0x59, 0x63, // Load v9 v6
            0x07, 0x54, 0x23, 0xDA, // 4 bytes of data
        ];

        let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
        emulator.ram[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(&rom);

        emulator.index = PROGRAM_START as u16 + 4;

        emulator.step();
        assert_eq!(emulator.registers[6], 0x07);
        assert_eq!(emulator.registers[7], 0x54);
        assert_eq!(emulator.registers[8], 0x23);
        assert_eq!(emulator.registers[9], 0xDA);
        assert_eq!(emulator.index, PROGRAM_START as u16 + 4);

        emulator.step();
        assert_eq!(emulator.registers[6], 0xDA);
        assert_eq!(emulator.registers[7], 0x23);
        assert_eq!(emulator.registers[8], 0x54);
        assert_eq!(emulator.registers[9], 0x07);
        assert_eq!(emulator.index, PROGRAM_START as u16 + 4);
    }

    #[test]
    fn opcode_display_plane_xo_chip() {
        #[rustfmt::skip]
        let rom = [
            0xF2, 0x01, // Select plane 1
            0xD0, 0x13, // Display v0 v1 3
            0xF3, 0x01, // Select both planes
            0xD0, 0x13, // Display v0 v1 3

            0b10000000, // 3-pixels long vertical sprite
            0b10000000,
            0b10000000,

            0b11100000, // 3-pixels long horizontal sprite
            0b00000000,
            0b00000000,
        ];

        let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
        emulator.ram[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(&rom);

        emulator.index = PROGRAM_START as u16 + 8;
        emulator.high_resolution = true;
        emulator.plane_selection = 0;

        emulator.step();
        assert_eq!(emulator.plane_selection, repeat_bits(0b10, 2));

        emulator.registers[0] = 17;
        emulator.registers[1] = 23;

        emulator.step();
        // plane 0 untouched
        // plane 1 is
        // 100
        // 100
        // 100
        assert_eq!(emulator.display_buffer[23][17], repeat_bits(0b10, 2));
        assert_eq!(emulator.display_buffer[24][17], repeat_bits(0b10, 2));
        assert_eq!(emulator.display_buffer[25][17], repeat_bits(0b10, 2));
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 0);

        emulator.step();
        assert_eq!(emulator.plane_selection, repeat_bits(0b11, 2));

        emulator.step();
        // plane 0 is
        // 100
        // 100
        // 100
        // plane 1 is
        // 011 -> first pixel is XOR'ed with previous step.
        // 100
        // 100
        assert_eq!(emulator.display_buffer[23][17], repeat_bits(0b01, 2));
        assert_eq!(emulator.display_buffer[23][18], repeat_bits(0b10, 2));
        assert_eq!(emulator.display_buffer[23][19], repeat_bits(0b10, 2));
        assert_eq!(emulator.display_buffer[24][17], repeat_bits(0b11, 2));
        assert_eq!(emulator.display_buffer[25][17], repeat_bits(0b11, 2));
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 1);
    }

    #[test]
    fn opcode_display_plane_16x16_xo_chip() {
        #[rustfmt::skip]
        let rom = [
            0xF2, 0x01, // Select plane 1
            0xD0, 0x10, // Display v0 v1 0
            0xF3, 0x01, // Select both planes
            0xD0, 0x10, // Display v0 v1 0

            0b1000_0000, 0b0000_0000, // 16x16 sprite with one pixel to the left on first 3px
            0b1000_0000, 0b0000_0000,
            0b1000_0000, 0b0000_0000,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            
            0b11100000, 0b0000_0000,// // 16x16 sprite with one pixel to the top on first 3px
            0b0000_0000, 0b0000_0000,
            0b0000_0000, 0b0000_0000,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
        emulator.ram[PROGRAM_START..PROGRAM_START + rom.len()].copy_from_slice(&rom);

        emulator.index = PROGRAM_START as u16 + 8;
        emulator.high_resolution = true;
        emulator.plane_selection = 0;

        emulator.step();
        assert_eq!(emulator.plane_selection, repeat_bits(0b10, 2));

        emulator.registers[0] = 17;
        emulator.registers[1] = 23;

        emulator.step();
        // plane 0 untouched
        // plane 1 is
        // 100
        // 100
        // 100
        assert_eq!(emulator.display_buffer[23][17], repeat_bits(0b10, 2));
        assert_eq!(emulator.display_buffer[24][17], repeat_bits(0b10, 2));
        assert_eq!(emulator.display_buffer[25][17], repeat_bits(0b10, 2));
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 0);

        emulator.step();
        assert_eq!(emulator.plane_selection, repeat_bits(0b11, 2));

        emulator.step();
        // plane 0 is
        // 100
        // 100
        // 100
        // plane 1 is
        // 011 -> first pixel is XOR'ed with previous step.
        // 100
        // 100
        assert_eq!(emulator.display_buffer[23][17], repeat_bits(0b01, 2));
        assert_eq!(emulator.display_buffer[23][18], repeat_bits(0b10, 2));
        assert_eq!(emulator.display_buffer[23][19], repeat_bits(0b10, 2));
        assert_eq!(emulator.display_buffer[24][17], repeat_bits(0b11, 2));
        assert_eq!(emulator.display_buffer[25][17], repeat_bits(0b11, 2));
        assert_eq!(emulator.registers[FLAG_REGISTER_INDEX], 1);
    }

    #[test]
    fn test_pitch() {
        let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
        // Values given in https://johnearnest.github.io/Octo/docs/XO-ChipSpecification.html

        emulator.pitch = 247;
        let rate_log2 = emulator.get_audio_bit_rate_log2_hz();
        const LOG2_56200_06: f32 = 15.778284050238645;

        assert_eq!(rate_log2, LOG2_56200_06);
    }
}
