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
}

impl App {
    fn new(rom: &[u8], mode: chirp8::Chirp8Mode) -> App {
        const WIDTH: u32 = (chirp8::DISPLAY_WIDTH * PIXELS_PER_CELL) as u32;
        const HEIGHT: u32 = (chirp8::DISPLAY_HEIGHT * PIXELS_PER_CELL) as u32;

        /// DIRTY FIX : for some reason Piston creates a window too large by these values,
        /// This does not depend on the WIDTH and HEIGHT, this is a constant error.
        const WINDOW_WIDTH_ERROR: u32 = 20;
        const WINDOW_HEIGHT_ERROR: u32 = 55;

        let window = WindowSettings::new(
            "Chirp 8",
            [WIDTH - WINDOW_WIDTH_ERROR, HEIGHT - WINDOW_HEIGHT_ERROR],
        )
        .graphics_api(OpenGL::V3_2)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

        let mut app = Self {
            emulator: Chirp8::new(mode),
            window: window,
            paused: false,
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
        const COLOR_ON: Color = [1.0, 1.0, 1.0, 1.0];

        self.window.draw_2d(event, |c, g, _device| {
            // Draw red background if sound.
            const SOUND_COLOR: Color = [1.0, 0.0, 0.0, 0.5];
            let background = if self.emulator.is_buzzer_on() {
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
                    if emulator_screen[i][j] {
                        let (i_px, j_px) = Self::get_cell_pixel_coordinates(i, j);
                        rectangle(
                            COLOR_ON,
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
        match key {
            // QWERTY layout
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

fn main() {
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");

    // Create a new game and run it.
    let mut app = App::new(rom, Chirp8Mode::SuperChipModern);

    app.run();
}
