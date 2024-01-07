fn print_display(buffer: &[[bool; chirp8::DISPLAY_WIDTH]; chirp8::DISPLAY_HEIGHT]) {
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
    println!();
}

fn acknowledge_keypress(emulator: &mut chirp8::Chirp8, key: u8) {
    const AKNOWLEDGE_STEPS: usize = 1000;
    emulator.key_press(key);
    for _ in 0..AKNOWLEDGE_STEPS {
        emulator.step();
    }
    emulator.key_release(key);
    for _ in 0..AKNOWLEDGE_STEPS {
        emulator.step();
    }
}

#[test]
fn ibm_logo() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/2-ibm-logo.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);
    for _ in 0..20 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/ibm_logo.bmp").unwrap();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn chip8_logo() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/1-chip8-logo.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);
    for _ in 0..39 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/chip8_logo.bmp").unwrap();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn corax() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/3-corax+.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);
    // Although undocumented, this test has to run for 284 steps to render entirely
    for _ in 0..284 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/corax+.bmp").unwrap();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn flags() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/4-flags.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);
    // Although undocumented, this test has to run for 952 steps to render entirely
    for _ in 0..952 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/flags.bmp").unwrap();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn quirks_chip_8() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);

    // Force Chip-8 test mode (key 1)
    let key = 1;
    acknowledge_keypress(&mut emulator, key);

    for _ in 0..3000 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/quirks_chip8.bmp").unwrap();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn quirks_super_chip_1_1() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::SuperChip1_1);

    emulator.load_rom(rom);
    // Force super chip test mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);
    // Legacy mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);

    for _ in 0..5000 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/quirks_super_chip_legacy.bmp").unwrap();

    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn quirks_super_chip_modern() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::SuperChipModern);

    emulator.load_rom(rom);
    // Force super chip test mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);
    // Modern mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);

    for _ in 0..5000 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/quirks_super_chip_modern.bmp").unwrap();

    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn keypad_fx0a() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/6-keypad.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::default();

    emulator.load_rom(rom);
    // Use test FX0A
    let key = 3;
    acknowledge_keypress(&mut emulator, key);
    // Press anything
    let key = 14;
    acknowledge_keypress(&mut emulator, key);

    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/keypad_FX0A.bmp").unwrap();

    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn scrolling_hires_1_1() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/8-scrolling.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::SuperChip1_1);
    emulator.load_rom(rom);

    // Super chip test mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);
    //print_display(emulator.get_display_buffer());
    // hires mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);
    //print_display(emulator.get_display_buffer());

    for _ in 0..5000 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_hires.bmp").unwrap();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn scrolling_lores_1_1() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/8-scrolling.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::SuperChip1_1);
    emulator.load_rom(rom);

    // Super chip test mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);
    // lores mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);
    // Legacy mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);

    for _ in 0..5000 {
        emulator.step();
    }

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_lores.bmp").unwrap();
    print_display(display);

    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn scrolling_hires_modern() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/8-scrolling.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::SuperChipModern);
    emulator.load_rom(rom);

    // Super chip test mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);
    //print_display(emulator.get_display_buffer());
    // hires mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);
    //print_display(emulator.get_display_buffer());

    for _ in 0..5000 {
        emulator.step();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_hires.bmp").unwrap();
    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}

#[test]
fn scrolling_lores_modern() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/8-scrolling.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::SuperChipModern);
    emulator.load_rom(rom);

    // Super chip test mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);
    // lores mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);
    // Modern mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);

    for _ in 0..5000 {
        emulator.step();
    }

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_lores.bmp").unwrap();
    print_display(display);

    for i in 0..64 {
        for j in 0..128 {
            assert_eq!(display[i][j], expected.get_pixel(j as u32, i as u32).r != 0);
        }
    }
}
