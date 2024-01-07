#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod chirp8;
mod stack;

pub use chirp8::*;
