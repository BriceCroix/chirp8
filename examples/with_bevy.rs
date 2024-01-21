mod common;
use std::process::exit;

use chirp8::Chirp8;
use common::*;

use bevy::prelude::*;
use bevy_pixel_buffer::prelude::*;

/// The emulator, one per app.
#[derive(Resource)]
struct EmulatorResource {
    emulator: Chirp8,
}

/// Configuration of the app.
#[derive(Resource)]
struct Configuration {
    keyboard_layout: KeyboardLayout,
}

/// How often a new emulator frame should be rendered.
#[derive(Resource)]
struct NewFrameConfig {
    /// How often to render a new frame? (repeating timer)
    timer: Timer,
    /// Is the app paused ?
    paused: bool,
}

/// Indicates if the emulator is currently sounding.
#[derive(Resource)]
struct IsSounding(bool);

/// Setup function to initialize emulator and insert bevy's resources.
fn setup(mut commands: Commands) {
    // Get the command-line arguments
    let args: Vec<String> = std::env::args().collect();
    let (file_path, chirp_mode, keyboard_layout, ticks_per_frame) = parse_arguments(&args);

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
    if let Option::Some(speed) = ticks_per_frame {
        emulator.set_steps_per_frame(speed);
    }

    // Insert resources.
    commands.insert_resource(EmulatorResource { emulator: emulator });
    commands.insert_resource(Configuration {
        keyboard_layout: keyboard_layout,
    });
    commands.insert_resource(NewFrameConfig {
        timer: Timer::new(
            core::time::Duration::from_secs_f32(1.0 / chirp8::REFRESH_RATE_HZ as f32),
            TimerMode::Repeating,
        ),
        paused: false,
    });
}

fn setup_sound(mut commands: Commands) {
    commands.insert_resource(IsSounding(false));
}

/// System to handle user input and press keys in the emulator.
fn emulator_input_system(
    keys: Res<Input<KeyCode>>,
    mut emulator_resource: ResMut<EmulatorResource>,
    configuration: Res<Configuration>,
    mut frame_configuration: ResMut<NewFrameConfig>,
) {
    let emulator = &mut emulator_resource.emulator;

    // Handle inputs
    match configuration.keyboard_layout {
        KeyboardLayout::Qwerty => {
            emulator.key_set(0x1, keys.pressed(KeyCode::Key1));
            emulator.key_set(0x2, keys.pressed(KeyCode::Key2));
            emulator.key_set(0x3, keys.pressed(KeyCode::Key3));
            emulator.key_set(0xC, keys.pressed(KeyCode::Key4));

            emulator.key_set(0x4, keys.pressed(KeyCode::Q));
            emulator.key_set(0x5, keys.pressed(KeyCode::W));
            emulator.key_set(0x6, keys.pressed(KeyCode::E));
            emulator.key_set(0xD, keys.pressed(KeyCode::R));

            emulator.key_set(0x7, keys.pressed(KeyCode::A));
            emulator.key_set(0x8, keys.pressed(KeyCode::S));
            emulator.key_set(0x9, keys.pressed(KeyCode::D));
            emulator.key_set(0xE, keys.pressed(KeyCode::F));

            emulator.key_set(0xA, keys.pressed(KeyCode::Z));
            emulator.key_set(0x0, keys.pressed(KeyCode::X));
            emulator.key_set(0xB, keys.pressed(KeyCode::C));
            emulator.key_set(0xF, keys.pressed(KeyCode::V));
        }
        KeyboardLayout::Azerty => {
            emulator.key_set(0x1, keys.pressed(KeyCode::Key1));
            emulator.key_set(0x2, keys.pressed(KeyCode::Key2));
            emulator.key_set(0x3, keys.pressed(KeyCode::Key3));
            emulator.key_set(0xC, keys.pressed(KeyCode::Key4));

            emulator.key_set(0x4, keys.pressed(KeyCode::A));
            emulator.key_set(0x5, keys.pressed(KeyCode::Z));
            emulator.key_set(0x6, keys.pressed(KeyCode::E));
            emulator.key_set(0xD, keys.pressed(KeyCode::R));

            emulator.key_set(0x7, keys.pressed(KeyCode::Q));
            emulator.key_set(0x8, keys.pressed(KeyCode::S));
            emulator.key_set(0x9, keys.pressed(KeyCode::D));
            emulator.key_set(0xE, keys.pressed(KeyCode::F));

            emulator.key_set(0xA, keys.pressed(KeyCode::W));
            emulator.key_set(0x0, keys.pressed(KeyCode::X));
            emulator.key_set(0xB, keys.pressed(KeyCode::C));
            emulator.key_set(0xF, keys.pressed(KeyCode::V));
        }
    }
    frame_configuration.paused ^= keys.just_pressed(KeyCode::Space);
}

/// System to run a frame of the emulator and update Bevy's UI
fn emulator_frame_system(
    mut emulator_resource: ResMut<EmulatorResource>,
    mut pixel_buffer: QueryPixelBuffer,
    time: Res<Time>,
    mut config: ResMut<NewFrameConfig>,
) {
    // Do not render a new emulator frame if paused or between frame rate.
    config.timer.tick(time.delta());
    if config.paused || !config.timer.finished() {
        return;
    }

    let emulator = &mut emulator_resource.emulator;

    // Update emulator state
    emulator.run_frame();

    // Get display buffer from emulator
    let display_buffer = emulator.get_display_buffer();

    // Set each pixel to corresponding color
    pixel_buffer.frame().per_pixel(|pos, _| {
        let value = display_buffer[pos.y as usize][pos.x as usize] as f32 / u8::MAX as f32;
        Pixel::from([value; 3])
    });
}

/// Inserts and remove audio source depending on the emulator state.
fn emulator_audio_system(
    mut commands: Commands,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    emulator_resource: Res<EmulatorResource>,
    mut is_sounding_resource: ResMut<IsSounding>,
    mut query: Query<Entity, With<AudioSink>>,
) {
    if !is_sounding_resource.0 && emulator_resource.emulator.is_sounding() {
        commands.spawn(PitchBundle {
            source: pitch_assets.add(Pitch::new(440.0, core::time::Duration::from_millis(1000))),
            settings: PlaybackSettings::LOOP,
        });
        is_sounding_resource.0 = true;
    } else if is_sounding_resource.0 && !emulator_resource.emulator.is_sounding() {
        for entity in query.iter_mut() {
            commands.entity(entity).despawn();
        }
        is_sounding_resource.0 = false;
    }
}

fn main() {
    let size = PixelBufferSize {
        size: UVec2::new(chirp8::DISPLAY_WIDTH as u32, chirp8::DISPLAY_HEIGHT as u32), // amount of pixels
        pixel_size: UVec2::new(8, 8), // size of each pixel in the screen
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PixelBufferPlugin)
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_sound)
        .add_systems(Startup, pixel_buffer_setup(size))
        .add_systems(Update, emulator_input_system)
        .add_systems(Update, emulator_frame_system)
        .add_systems(Update, emulator_audio_system)
        .run();
}
