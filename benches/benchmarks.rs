use chirp8::{Chirp8, Chirp8Mode};
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {

    let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
    // Draw 15-high sprite
    emulator.load_rom(&[0xD0, 0x1F]); 
    c.bench_function("Draw h15 low-res 1 plane", move |b| {
        b.iter(|| {
            emulator.step();
            emulator.reset();
        })
    });

    let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
    // Enable hi-res, Draw 15-high sprite
    emulator.load_rom(&[0x00, 0xFF, 0xD0, 0x1F]);
    c.bench_function("Draw h15 high-res 1 plane", move |b| {
        b.iter(|| {
            emulator.step();
            emulator.step();
            emulator.reset();
        })
    });

    let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
    // Enable hi-res, Enable both planes, draw 15-high sprite
    emulator.load_rom(&[0x00, 0xFF, 0xF3, 0x01, 0xD0, 0x1F]);
    c.bench_function("Draw h15 high-res 2 planes", move |b| {
        b.iter(|| {
            emulator.step();
            emulator.step();
            emulator.step();
            emulator.reset();
        })
    });

    let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
    // Enable hi-res, Enable both planes, draw 16x16 large sprite
    emulator.load_rom(&[0x00, 0xFF, 0xF3, 0x01, 0xD0, 0x10]);
    c.bench_function("Draw large sprite 2 planes", move |b| {
        b.iter(|| {
            emulator.step();
            emulator.step();
            emulator.step();
            emulator.reset();
        })
    });

    let mut emulator = Chirp8::new(Chirp8Mode::XOChip);
    // Enable hi-res, scroll right
    emulator.load_rom(&[0x00, 0xFF, 0x00, 0xFB]);
    c.bench_function("Scroll 1 plane", move |b| {
        b.iter(|| {
            emulator.step();
            emulator.step();
            emulator.reset();
        })
    });

    let mut emulator = Chirp8::new(Chirp8Mode::SuperChipModern);
    // Enable hi-res, scroll right
    emulator.load_rom(&[0x00, 0xFF, 0x00, 0xFB]);
    c.bench_function("Scroll all planes", move |b| {
        b.iter(|| {
            emulator.step();
            emulator.step();
            emulator.reset();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
