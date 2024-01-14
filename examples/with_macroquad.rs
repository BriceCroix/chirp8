use std::process::exit;

use chirp8::Chirp8;
use macroquad::prelude::*;

mod common;
use common::*;

/// Number of desktop pixels per chip-8 pixel.
const PIXELS_PER_CELL: usize = 10;

fn macroquad_configuration() -> Conf {
    Conf {
        window_title: "Chirp-8".to_owned(),
        window_height: (chirp8::DISPLAY_HEIGHT * PIXELS_PER_CELL) as i32,
        window_width: (chirp8::DISPLAY_WIDTH * PIXELS_PER_CELL) as i32,
        window_resizable: false,

        ..Default::default()
    }
}

#[macroquad::main(macroquad_configuration)]
async fn main() {
    // Get the command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let (file_path, chirp_mode, keyboard_layout, ticks_per_second) = parse_arguments(&args);

    // Read given command line rom.
    let rom = read_file_bytes(&file_path);
    if let Result::Err(err) = rom {
        eprintln!("Error reading file \"{}\" : {}", file_path, err);
        exit(1);
    }
    let rom = rom.ok().unwrap();

    // Create emulator and load given rom.
    let mut emulator = Chirp8::new(chirp_mode);
    emulator.load_rom(&rom);
    if let Option::Some(speed) = ticks_per_second {
        emulator.set_steps_per_frame(speed);
    }

    // Initialize app.
    let mut paused = false;
    let mut previous_chirp_frame_time = get_time();
    let chirp_frame_interval = 1f64 / chirp8::REFRESH_RATE_HZ as f64;

    loop {
        // Handle inputs
        match keyboard_layout {
            KeyboardLayout::Qwerty => {
                emulator.key_set(0x1, is_key_down(KeyCode::Key1));
                emulator.key_set(0x2, is_key_down(KeyCode::Key2));
                emulator.key_set(0x3, is_key_down(KeyCode::Key3));
                emulator.key_set(0xC, is_key_down(KeyCode::Key4));

                emulator.key_set(0x4, is_key_down(KeyCode::Q));
                emulator.key_set(0x5, is_key_down(KeyCode::W));
                emulator.key_set(0x6, is_key_down(KeyCode::E));
                emulator.key_set(0xD, is_key_down(KeyCode::R));

                emulator.key_set(0x7, is_key_down(KeyCode::A));
                emulator.key_set(0x8, is_key_down(KeyCode::S));
                emulator.key_set(0x9, is_key_down(KeyCode::D));
                emulator.key_set(0xE, is_key_down(KeyCode::F));

                emulator.key_set(0xA, is_key_down(KeyCode::Z));
                emulator.key_set(0x0, is_key_down(KeyCode::X));
                emulator.key_set(0xB, is_key_down(KeyCode::C));
                emulator.key_set(0xF, is_key_down(KeyCode::V));
            }
            // NB : there is currently a bug with macroquad that always recognize the keyboard as QWERTY,
            // It is then best to call this executable with the qwerty option even though the keyboard might be AZERTY or else.
            KeyboardLayout::Azerty => {
                emulator.key_set(0x1, is_key_down(KeyCode::Key1));
                emulator.key_set(0x2, is_key_down(KeyCode::Key2));
                emulator.key_set(0x3, is_key_down(KeyCode::Key3));
                emulator.key_set(0xC, is_key_down(KeyCode::Key4));

                emulator.key_set(0x4, is_key_down(KeyCode::A));
                emulator.key_set(0x5, is_key_down(KeyCode::Z));
                emulator.key_set(0x6, is_key_down(KeyCode::E));
                emulator.key_set(0xD, is_key_down(KeyCode::R));

                emulator.key_set(0x7, is_key_down(KeyCode::Q));
                emulator.key_set(0x8, is_key_down(KeyCode::S));
                emulator.key_set(0x9, is_key_down(KeyCode::D));
                emulator.key_set(0xE, is_key_down(KeyCode::F));

                emulator.key_set(0xA, is_key_down(KeyCode::W));
                emulator.key_set(0x0, is_key_down(KeyCode::X));
                emulator.key_set(0xB, is_key_down(KeyCode::C));
                emulator.key_set(0xF, is_key_down(KeyCode::V));
            }
        }
        paused ^= is_key_pressed(KeyCode::Space);

        let time = get_time();
        let elapsed_since_chirp_frame = time - previous_chirp_frame_time;

        if paused {
            previous_chirp_frame_time = time;
        } else if elapsed_since_chirp_frame > chirp_frame_interval {
            previous_chirp_frame_time = time;
            emulator.run_frame();
        }
        // Draw red background if sound.
        const SOUND_COLOR: Color = Color::new(1.0, 0.0, 0.0, 1.0);
        const COLOR_OFF: Color = Color::new(0.0, 0.0, 0.0, 1.0);
        let background = if emulator.is_sounding() {
            SOUND_COLOR
        } else {
            COLOR_OFF
        };
        clear_background(background);
        for (i, row) in emulator.get_display_buffer().iter().enumerate() {
            for (j, pixel) in row.iter().enumerate() {
                if *pixel != 0 {
                    let color = (*pixel as f32) / (u8::MAX as f32);
                    let color = Color::new(color, color, color, 1.0);
                    draw_rectangle(
                        (j * PIXELS_PER_CELL) as f32,
                        (i * PIXELS_PER_CELL) as f32,
                        PIXELS_PER_CELL as f32,
                        PIXELS_PER_CELL as f32,
                        color,
                    );
                }
            }
        }

        next_frame().await;
    }
}
