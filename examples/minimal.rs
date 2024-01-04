fn main() {
    let rom_file = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");

    let mut rom = [0; chirp8::PROGRAM_SIZE];
    rom[..rom_file.len()].copy_from_slice(rom_file);

    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    chirp8.load_rom(&rom);

    chirp8.key_press(1);
    for step in 0..1000 {
        chirp8.step();
        if chirp8.display_changed() {
            println!("Display changed at step {} !", step);
        }
    }
    chirp8.key_release(1);
    for _ in 0..3000 {
        chirp8.step();
    }
    let display = chirp8.get_display_buffer();

    for row in display {
        for pixel in row {
            if *pixel {
                print!("\u{25A0}");
            } else {
                print!("-");
            }
        }

        println!();
    }
}
