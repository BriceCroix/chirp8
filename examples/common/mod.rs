use std::io::Read;

use chirp8::Chirp8Mode;

#[derive(Debug, PartialEq)]
pub enum KeyboardLayout {
    Qwerty,
    Azerty,
}

/// Read given `file_path` as an array of bytes.
pub fn read_file_bytes(file_path: &str) -> Result<Vec<u8>, std::io::Error> {
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

/// Parses program options.
/// Returns :
/// - The rom file path as first unnamed argument.
/// - Chosen chip-8 mode, if option -c -s -x is supplied.
/// - Keyboard layout if option --azerty is supplied.
/// - Optional emulator steps per frame is option --speed is supplied
pub fn parse_arguments(
    args: &std::vec::Vec<String>,
) -> (String, chirp8::Chirp8Mode, KeyboardLayout, Option<usize>) {
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

    opts.optopt("", "speed", "Number of emulator steps per frame", "COUNT");

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

    let speed = if let Option::Some(speed) = matches.opt_str("speed") {
        if let Option::Some(speed) = usize::from_str_radix(&speed, 10).ok() {
            Option::Some(speed)
        } else {
            eprintln!("Invalid speed option {}", speed);
            Option::None
        }
    } else {
        Option::None
    };

    (file_path, mode, layout, speed)
}