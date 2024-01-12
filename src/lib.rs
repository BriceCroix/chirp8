#![doc = include_str!("../README.md")]

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod chirp8;
mod stack;
mod quirks;

pub use chirp8::*;
pub use quirks::*;
