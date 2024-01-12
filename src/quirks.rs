use bitflags::bitflags;

use crate::Chirp8Mode;

bitflags! {
    /// Represent the deviations from the original Chip-8 language.
    pub struct QuirkFlags: u16 {
        /// The AND, OR and XOR opcodes (8xy1, 8xy2 and 8xy3) reset the flags register to zero.
        const FLAG_RESET = 1 << 0;
        /// The save and load opcodes (Fx55 and Fx65) increment the index register.
        const INC_INDEX = 1 << 1;
        /// Drawing sprites to the display waits for the vertical blank interrupt in low-resolution.
        const DISPLAY_WAIT_LORES = 1 << 2;
        /// Drawing sprites to the display waits for the vertical blank interrupt
        /// in high-resolution.
        const DISPLAY_WAIT_HIRES = 1 << 3;

        /// Sprites drawn in low-resolution at the bottom edge of the screen get clipped instead of
        /// wrapping around
        /// to the top of the screen.
        const CLIP_SPRITES_LORES = 1 << 4;
        /// Sprites drawn in high-resolution at the bottom edge of the screen get clipped instead
        /// of wrapping around
        /// to the top of the screen.
        const CLIP_SPRITES_HIRES = 1 << 5;
        /// The shift opcodes (8xy6 and 8xyE) only operate on vX instead of storing the shifted
        /// version of vY in vX.
        const SHIFT_X_ONLY = 1 << 6;
        /// The Jump with offset opcode (0xBNNN) jumps to NNN + vX instead of NNN + v0.
        const JUMP_XNN = 1 << 7;

        /// The memory is initialized at random at the beginning.
        const RAM_RANDOM = 1 << 8;
        /// The screen is cleared when swapping between low and high ressolution
        const CLEAR_ON_RES = 1 << 9;
        /// When drawing a sprite in low-resolution, counts the number of rows that either collide
        /// with something or are below the bottom of the screen instead of just being set to 1
        /// when a collision occurs.
        const COLLISION_COUNT_LORES = 1 << 10;
        /// When drawing a sprite in high-resolution, counts the number of rows that either collide
        /// with something or are below the bottom of the screen instead of just being set to 1
        /// when a collision occurs.
        const COLLISION_COUNT_HIRES = 1 << 11;

        /// If the display uses more than one plane (2 on XO-Chip).
        const USE_SEVERAL_PLANES = 1 << 12;
        /// The scroll instructions scroll by half pixels when in low-resolution.
        const SCROLL_HALF_PIXEL = 1 << 13;
    }
}

impl QuirkFlags {
    /// Creates the quirks configuration corresponding to given preset.
    pub fn from_mode(mode: Chirp8Mode) -> QuirkFlags {
        match mode {
            Chirp8Mode::CosmacChip8 => {
                QuirkFlags::FLAG_RESET
                    | QuirkFlags::INC_INDEX
                    | QuirkFlags::DISPLAY_WAIT_LORES
                    | QuirkFlags::CLIP_SPRITES_LORES
            }
            Chirp8Mode::SuperChip1_1 => {
                QuirkFlags::DISPLAY_WAIT_LORES
                    | QuirkFlags::CLIP_SPRITES_LORES
                    | QuirkFlags::CLIP_SPRITES_HIRES
                    | QuirkFlags::SHIFT_X_ONLY
                    | QuirkFlags::JUMP_XNN
                    | QuirkFlags::RAM_RANDOM
                    | QuirkFlags::COLLISION_COUNT_LORES
                    | QuirkFlags::COLLISION_COUNT_HIRES
                    | QuirkFlags::SCROLL_HALF_PIXEL
            }
            Chirp8Mode::SuperChipModern => {
                QuirkFlags::CLIP_SPRITES_LORES
                    | QuirkFlags::CLIP_SPRITES_HIRES
                    | QuirkFlags::SHIFT_X_ONLY
                    | QuirkFlags::JUMP_XNN
                    | QuirkFlags::CLEAR_ON_RES
                    | QuirkFlags::COLLISION_COUNT_LORES
                    | QuirkFlags::COLLISION_COUNT_HIRES
            }
            Chirp8Mode::XOChip => QuirkFlags::INC_INDEX | QuirkFlags::USE_SEVERAL_PLANES,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quirks() {
        let quirks = QuirkFlags::from_mode(Chirp8Mode::CosmacChip8);
        assert!(quirks.contains(QuirkFlags::DISPLAY_WAIT_LORES));
        assert!(!quirks.contains(QuirkFlags::DISPLAY_WAIT_HIRES));
    }
}
