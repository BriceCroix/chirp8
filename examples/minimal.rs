fn main() {
    // Create emulator.
    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    // Load the ROM, which is just a byte array. Statically loaded here for simplicity.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/1-chip8-logo.ch8");
    chirp8.load_rom(rom);

    // Keys can be pressed as follows.
    chirp8.key_press(1);

    // Run 100 frames (Normally at 60 frames per seconds).
    for _ in 0..100 {
        chirp8.run_frame();
    }

    // Release key as follows.
    chirp8.key_release(1);

    // The display buffer, the pixels array, can be accessed as follows.
    // Also try `display_changed()` to know if the screen needs to be redrawn.
    let screen = chirp8.get_display_buffer();
    for pixel_row in screen {
        for pixel in pixel_row {
            // If a pixel is ON, printing a black square.
            if *pixel == chirp8::PIXEL_ON {
                print!("\u{25A0}");
            } else {
                print!("-");
            }
        }
        println!();
    }

    // The original Chip interpreter only has a sound buzzer, get its state with :
    let sound_is_on = chirp8.is_sounding();
    if sound_is_on {
        println!("Buzzer on !");
    } else {
        println!("Buzzer off !");
    }
}
