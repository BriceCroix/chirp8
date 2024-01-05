use chirp8;

fn _print_display(buffer: &[[bool; chirp8::DISPLAY_WIDTH]; chirp8::DISPLAY_HEIGHT]) {
    for row in buffer {
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

#[test]
fn ibm_logo() {
    // Statically load test rom.
    let rom_file = include_bytes!("../submodules/chip8-test-suite/bin/2-ibm-logo.ch8");
    let mut rom = [0; chirp8::PROGRAM_SIZE];
    rom[..rom_file.len()].copy_from_slice(rom_file);

    // Statically load expected display at end of test rom.
    let image = bmp::open("tests/ibm_logo.bmp").unwrap();

    // Create and run emulator
    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    chirp8.load_rom(&rom);
    for _ in 0..20 {
        chirp8.step();
    }

    let display = chirp8.get_display_buffer();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], image.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn chip8_logo() {
    // Statically load test rom.
    let rom_file = include_bytes!("../submodules/chip8-test-suite/bin/1-chip8-logo.ch8");
    let mut rom = [0; chirp8::PROGRAM_SIZE];
    rom[..rom_file.len()].copy_from_slice(rom_file);

    // Statically load expected display at end of test rom.
    let image = bmp::open("tests/chip8_logo.bmp").unwrap();

    // Create and run emulator
    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    chirp8.load_rom(&rom);
    for _ in 0..39 {
        chirp8.step();
    }

    let display = chirp8.get_display_buffer();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], image.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn corax() {
    // Statically load test rom.
    let rom_file = include_bytes!("../submodules/chip8-test-suite/bin/3-corax+.ch8");
    let mut rom = [0; chirp8::PROGRAM_SIZE];
    rom[..rom_file.len()].copy_from_slice(rom_file);

    // Statically load expected display at end of test rom.
    let image = bmp::open("tests/corax+.bmp").unwrap();

    // Create and run emulator
    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    chirp8.load_rom(&rom);
    // Although undocumented, this test has to run for 284 steps to render entirely
    for _ in 0..284 {
        chirp8.step();
    }

    let display = chirp8.get_display_buffer();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], image.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn flags() {
    // Statically load test rom.
    let rom_file = include_bytes!("../submodules/chip8-test-suite/bin/4-flags.ch8");
    let mut rom = [0; chirp8::PROGRAM_SIZE];
    rom[..rom_file.len()].copy_from_slice(rom_file);

    // Statically load expected display at end of test rom.
    let image = bmp::open("tests/flags.bmp").unwrap();

    // Create and run emulator
    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    chirp8.load_rom(&rom);
    // Although undocumented, this test has to run for 952 steps to render entirely
    for _ in 0..952 {
        chirp8.step();
    }

    let display = chirp8.get_display_buffer();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], image.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn quirks_chip_8() {
    // Statically load test rom.
    let rom_file = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");
    let mut rom = [0; chirp8::PROGRAM_SIZE];
    rom[..rom_file.len()].copy_from_slice(rom_file);

    // Statically load expected display at end of test rom.
    let image = bmp::open("tests/quirks_chip8.bmp").unwrap();

    // Create and run emulator
    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    chirp8.load_rom(&rom);
    // Force Chip-8 test mode (key 1)
    let key = 1;
    chirp8.key_press(key);
    for _ in 0..1000 {
        chirp8.step();
    }
    chirp8.key_release(key);
    for _ in 0..3000 {
        chirp8.step();
    }

    let display = chirp8.get_display_buffer();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], image.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn quirks_super_chip() {
    // Statically load test rom.
    let rom_file = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");
    let mut rom = [0; chirp8::PROGRAM_SIZE];
    rom[..rom_file.len()].copy_from_slice(rom_file);

    // Statically load expected display at end of test rom.
    let image = bmp::open("tests/quirks_super_chip_modern.bmp").unwrap();

    // Create and run emulator
    let mut chirp8 = chirp8::Chirp8::new(chirp8::Chirp8Mode::SuperChip);

    chirp8.load_rom(&rom);
    // Force super chip test mode
    let key = 2;
    chirp8.key_press(key);
    for _ in 0..1000 {
        chirp8.step();
    }
    chirp8.key_release(key);
    for _ in 0..1000 {
        chirp8.step();
    }
    let key = 1; // Modern mode
    chirp8.key_press(1);
    for _ in 0..1000 {
        chirp8.step();
    }
    chirp8.key_release(key);
    for _ in 0..(3000 + 8000) {
        chirp8.step();
    }

    let display = chirp8.get_display_buffer();
    //_print_display(display);
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], image.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}
