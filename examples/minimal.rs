fn main() {
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");

    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    chirp8.load_rom(rom);

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
            if *pixel == chirp8::PIXEL_ON{
                print!("\u{25A0}");
            } else {
                print!("-");
            }
        }

        println!();
    }
}
