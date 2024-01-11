use std::io::Read;

use chirp8::{Chirp8, Chirp8Mode, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use graphics::types::Color;
use opengl_graphics::OpenGL;
use piston::input::{UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{Button, Event, EventLoop, Key, PressEvent, ReleaseEvent};
use piston_window::PistonWindow as Window;

/// Number of desktop pixels per chip-8 pixel.
const PIXELS_PER_CELL: usize = 10;

pub struct App {
    emulator: Chirp8,
    window: Window,
    paused: bool,
    keyboard_layout: KeyboardLayout,
}

impl App {
    fn new(rom: &[u8], mode: chirp8::Chirp8Mode, keyboard_layout: KeyboardLayout) -> App {
        const WIDTH: u32 = (chirp8::DISPLAY_WIDTH * PIXELS_PER_CELL) as u32;
        const HEIGHT: u32 = (chirp8::DISPLAY_HEIGHT * PIXELS_PER_CELL) as u32;

        /// DIRTY FIX : for some reason on linux Piston creates a window too large by these values,
        /// This does not depend on the WIDTH and HEIGHT, this is a constant error.
        const WINDOW_WIDTH_ERROR: u32 = 20;
        const WINDOW_HEIGHT_ERROR: u32 = 55;

        let size = if cfg!(target_os = "linux") {
            [WIDTH - WINDOW_WIDTH_ERROR, HEIGHT - WINDOW_HEIGHT_ERROR]
        } else {
            [WIDTH, HEIGHT]
        };

        let window = WindowSettings::new("Chirp 8", size)
            .graphics_api(OpenGL::V3_2)
            .exit_on_esc(true)
            .resizable(false)
            .build()
            .unwrap();

        let mut app = Self {
            emulator: Chirp8::new(mode),
            window: window,
            paused: false,
            keyboard_layout: keyboard_layout,
        };
        app.emulator.load_rom(rom);
        app
    }

    pub fn get_cell_pixel_coordinates(row: usize, col: usize) -> (usize, usize) {
        return (row * PIXELS_PER_CELL, col * PIXELS_PER_CELL);
    }

    pub fn render(&mut self, event: &Event) {
        use graphics::*;

        const COLOR_OFF: Color = [0.0, 0.0, 0.0, 1.0];

        self.window.draw_2d(event, |c, g, _device| {
            // Draw red background if sound.
            const SOUND_COLOR: Color = [1.0, 0.0, 0.0, 0.5];
            let background = if self.emulator.is_sounding() {
                SOUND_COLOR
            } else {
                COLOR_OFF
            };
            // Clear the screen.
            clear(background, g);

            let emulator_screen = self.emulator.get_display_buffer();

            // Draw a square for "on" pixel
            for i in 0..DISPLAY_HEIGHT {
                for j in 0..DISPLAY_WIDTH {
                    if emulator_screen[i][j] != 0{
                        let color = emulator_screen[i][j] as f32 / 255f32;
                        let (i_px, j_px) = Self::get_cell_pixel_coordinates(i, j);
                        rectangle(
                            [color, color, color, 1.0],
                            rectangle::square(0.0, 0.0, PIXELS_PER_CELL as f64),
                            c.transform.trans(j_px as f64, i_px as f64),
                            g,
                        );
                    }
                }
            }
        });
    }

    pub fn update(&mut self, _args: &UpdateArgs) {
        if !self.paused {
            self.emulator.run_frame();
        }
    }

    /// `pressed` is true when the key is pressed and false when released.
    fn process_keyboard(&mut self, key: Key, pressed: bool) {
        match self.keyboard_layout {
            // QWERTY layout
            KeyboardLayout::Qwerty => match key {
                Key::D1 => self.emulator.key_set(0x1, pressed),
                Key::D2 => self.emulator.key_set(0x2, pressed),
                Key::D3 => self.emulator.key_set(0x3, pressed),
                Key::D4 => self.emulator.key_set(0xC, pressed),
                Key::Q => self.emulator.key_set(0x4, pressed),
                Key::W => self.emulator.key_set(0x5, pressed),
                Key::E => self.emulator.key_set(0x6, pressed),
                Key::R => self.emulator.key_set(0xD, pressed),
                Key::A => self.emulator.key_set(0x7, pressed),
                Key::S => self.emulator.key_set(0x8, pressed),
                Key::D => self.emulator.key_set(0x9, pressed),
                Key::F => self.emulator.key_set(0xE, pressed),
                Key::Z => self.emulator.key_set(0xA, pressed),
                Key::X => self.emulator.key_set(0x0, pressed),
                Key::C => self.emulator.key_set(0xB, pressed),
                Key::V => self.emulator.key_set(0xF, pressed),
                // Discard other keys
                _ => {}
            },

            // QWERTY layout
            KeyboardLayout::Azerty => match key {
                Key::D1 => self.emulator.key_set(0x1, pressed),
                Key::D2 => self.emulator.key_set(0x2, pressed),
                Key::D3 => self.emulator.key_set(0x3, pressed),
                Key::D4 => self.emulator.key_set(0xC, pressed),
                Key::A => self.emulator.key_set(0x4, pressed),
                Key::Z => self.emulator.key_set(0x5, pressed),
                Key::E => self.emulator.key_set(0x6, pressed),
                Key::R => self.emulator.key_set(0xD, pressed),
                Key::Q => self.emulator.key_set(0x7, pressed),
                Key::S => self.emulator.key_set(0x8, pressed),
                Key::D => self.emulator.key_set(0x9, pressed),
                Key::F => self.emulator.key_set(0xE, pressed),
                Key::W => self.emulator.key_set(0xA, pressed),
                Key::X => self.emulator.key_set(0x0, pressed),
                Key::C => self.emulator.key_set(0xB, pressed),
                Key::V => self.emulator.key_set(0xF, pressed),
                // Discard other keys
                _ => {}
            },
        }
        // Common to all layouts
        match key {
            // Numeric pad
            Key::NumPad7 => self.emulator.key_set(0x1, pressed),
            Key::NumPad8 => self.emulator.key_set(0x2, pressed),
            Key::NumPad9 => self.emulator.key_set(0x3, pressed),
            Key::NumPad4 => self.emulator.key_set(0x4, pressed),
            Key::NumPad5 => self.emulator.key_set(0x5, pressed),
            Key::NumPad6 => self.emulator.key_set(0x6, pressed),
            Key::NumPad1 => self.emulator.key_set(0x7, pressed),
            Key::NumPad2 => self.emulator.key_set(0x8, pressed),
            Key::NumPad3 => self.emulator.key_set(0x9, pressed),
            Key::NumPad0 => self.emulator.key_set(0x0, pressed),

            Key::Space => self.paused ^= pressed,
            // Discard other keys
            _ => {}
        }
    }

    pub fn run(&mut self) {
        let update_per_second = chirp8::REFRESH_RATE_HZ;
        self.window.set_max_fps(120);
        self.window.set_ups(update_per_second as u64);
        self.window.set_lazy(false);

        while let Some(e) = self.window.next() {
            // Then process inputs.
            if let Some(Button::Keyboard(key)) = e.press_args() {
                self.process_keyboard(key, true);
            };
            if let Some(Button::Keyboard(key)) = e.release_args() {
                self.process_keyboard(key, false);
            };
            // Update state accordingly.
            if let Some(args) = e.update_args() {
                self.update(&args);
            }
            // Finally render.
            self.render(&e);
        }
    }
}

#[derive(PartialEq)]
enum KeyboardLayout {
    Qwerty,
    Azerty,
}

/// Read given file path as bytes.
fn read_file_bytes(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
    // Attempt to open the file
    let mut file = std::fs::File::open(file_path)?;

    // Get the metadata to determine the file size
    let metadata = file.metadata()?;
    let file_size = metadata.len() as usize;

    // Read the file contents into a Vec<u8>
    let mut buffer = Vec::with_capacity(file_size);
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

/// Function to parse program options.
/// Returns the rom file path, chosen chip-8 mode, and keyboard layout.
fn parse_arguments(args: &std::vec::Vec<String>) -> (String, chirp8::Chirp8Mode, KeyboardLayout) {
    let mut opts = getopts::Options::new();

    opts.optflag("c", "chip", "Use original Chip-8");
    opts.optflag("s", "super-chip", "Use Super Chip 1.1");
    opts.optflag("m", "modern-super-chip", "Use Modernized Super Chip");
    opts.optflag("x", "xo-chip", "Use XO-Chip");
    opts.optflag(
        "a",
        "azerty",
        "Use Azerty keyboard layout instead of Qwerty",
    );

    // Parse options
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("Error parsing options: {}", f);
            std::process::exit(1);
        }
    };

    // Get the file path
    let file_path = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        // If no file path is provided, print usage and exit
        eprintln!("Usage: {} [-c | -s | -m] <file_path>", args[0]);
        std::process::exit(1);
    };

    let mode = if matches.opt_present("c") {
        Chirp8Mode::CosmacChip8
    } else if matches.opt_present("s") {
        Chirp8Mode::SuperChip1_1
    } else if matches.opt_present("m") {
        Chirp8Mode::SuperChipModern
    } else if matches.opt_present("x") {
        Chirp8Mode::XOChip
    } else {
        Chirp8Mode::CosmacChip8
    };

    let layout = if matches.opt_present("a") {
        KeyboardLayout::Azerty
    } else {
        KeyboardLayout::Qwerty
    };

    (file_path, mode, layout)
}

fn main() {
    // Get the command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // Check if at least one argument (the program name) is provided
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        return;
    }

    let (file_path, mode, layout) = parse_arguments(&args);

    match read_file_bytes(&file_path) {
        Ok(rom) => {
            // Create a new app and run it.
            let mut app = App::new(rom.as_slice(), mode, layout);
            app.run();
        }
        Err(err) => eprintln!("Error reading file: {}", err),
    }
}
