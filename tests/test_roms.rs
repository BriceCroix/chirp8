use chirp8::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

fn print_display(buffer: &chirp8::DisplayBuffer) {
    for row in buffer {
        for pixel in row {
            if *pixel != 0 {
                print!("\u{25A0}");
            } else {
                print!("-");
            }
        }
        println!();
    }
    println!();
}

/// Asserts that every pixel in the given `buffer` is the same as given `expected` image.
/// Compares the pixel value if `compare_value` is true, otherwise only checks if pixels are turned on or not.
fn assert_screen_eq(buffer: &chirp8::DisplayBuffer, expected: &bmp::Image, compare_value: bool) {
    for i in 0..DISPLAY_HEIGHT {
        for j in 0..DISPLAY_WIDTH {
            if compare_value {
                assert_eq!(buffer[i][j], expected.get_pixel(j as u32, i as u32).r);
            } else {
                assert_eq!(
                    buffer[i][j] == 0,
                    expected.get_pixel(j as u32, i as u32).r == 0
                );
            }
        }
    }
}

fn acknowledge_keypress(emulator: &mut chirp8::Chirp8, key: u8) {
    const ACKNOWLEDGE_FRAMES: usize = 100;
    emulator.key_press(key);
    for _ in 0..ACKNOWLEDGE_FRAMES {
        emulator.run_frame();
    }
    emulator.key_release(key);
    for _ in 0..ACKNOWLEDGE_FRAMES {
        emulator.run_frame();
    }
}

#[test]
fn ibm_logo() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/2-ibm-logo.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);
    emulator.take_steps(20);
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/ibm_logo.bmp").unwrap();
    assert_screen_eq(display, &expected, true);
}

#[test]
fn chip8_logo() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/1-chip8-logo.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);
    emulator.take_steps(39);
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/chip8_logo.bmp").unwrap();
    assert_screen_eq(display, &expected, true);
}

#[test]
fn corax() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/3-corax+.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);
    // Although undocumented, this test has to run for 284 steps to render entirely
    emulator.take_steps(284);
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/corax+.bmp").unwrap();
    assert_screen_eq(display, &expected, true);
}

#[test]
fn flags() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/4-flags.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::CosmacChip8);

    emulator.load_rom(rom);
    // Although undocumented, this test has to run for 952 steps to render entirely
    emulator.take_steps(952);
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/flags.bmp").unwrap();
    assert_screen_eq(display, &expected, true);
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

    for _ in 0..300 {
        emulator.run_frame();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/quirks_chip8.bmp").unwrap();
    assert_screen_eq(display, &expected, true);
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

    for _ in 0..500 {
        emulator.run_frame();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/quirks_super_chip_legacy.bmp").unwrap();

    assert_screen_eq(display, &expected, true);
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

    for _ in 0..500 {
        emulator.run_frame();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/quirks_super_chip_modern.bmp").unwrap();

    assert_screen_eq(display, &expected, true);
}

#[test]
fn quirks_xo_chip() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/5-quirks.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::XOChip);

    emulator.load_rom(rom);
    // Force XO chip test mode
    let key = 3;
    acknowledge_keypress(&mut emulator, key);

    for _ in 0..500 {
        emulator.run_frame();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/quirks_xo_chip.bmp").unwrap();

    assert_screen_eq(display, &expected, false);
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

    assert_screen_eq(display, &expected, true);
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

    for _ in 0..500 {
        emulator.run_frame();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_hires.bmp").unwrap();
    assert_screen_eq(display, &expected, true);
}

#[test]
fn scrolling_lores_super_chip_1_1() {
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

    for _ in 0..500 {
        emulator.run_frame();
    }

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_lores.bmp").unwrap();
    print_display(display);

    assert_screen_eq(display, &expected, true);
}

#[test]
fn scrolling_hires_super_chip_modern() {
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

    for _ in 0..500 {
        emulator.run_frame();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_hires.bmp").unwrap();
    assert_screen_eq(display, &expected, true);
}

#[test]
fn scrolling_lores_super_chip_modern() {
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

    for _ in 0..500 {
        emulator.run_frame();
    }

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_lores.bmp").unwrap();
    print_display(display);

    assert_screen_eq(display, &expected, true);
}

#[test]
fn scrolling_hires_xo_chip() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/8-scrolling.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::XOChip);
    emulator.load_rom(rom);

    // XO chip test mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);
    // hires mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);

    for _ in 0..500 {
        emulator.run_frame();
    }
    print_display(emulator.get_display_buffer());

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_xo_chip_hires.bmp").unwrap();
    assert_screen_eq(display, &expected, false);
}

#[test]
fn scrolling_lores_xo_chip() {
    // Statically load test rom.
    let rom = include_bytes!("../submodules/chip8-test-suite/bin/8-scrolling.ch8");

    // Create and run emulator
    let mut emulator = chirp8::Chirp8::new(chirp8::Chirp8Mode::XOChip);
    emulator.load_rom(rom);

    // XO chip test mode
    let key = 2;
    acknowledge_keypress(&mut emulator, key);
    // lores mode
    let key = 1;
    acknowledge_keypress(&mut emulator, key);

    for _ in 0..500 {
        emulator.run_frame();
    }

    let display = emulator.get_display_buffer();
    let expected = bmp::open("tests/scrolling_xo_chip_lores.bmp").unwrap();
    print_display(display);

    assert_screen_eq(display, &expected, false);
}
