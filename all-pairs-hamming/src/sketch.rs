//! Traits of binary short sketches of primitive integer types.
use std::ops::Range;
use std::usize;

use num_traits::int::PrimInt;
use num_traits::{FromPrimitive, ToPrimitive};

/// Trait of a binary short sketch from a primitive integer type.
pub trait Sketch: Default + PrimInt + FromPrimitive + ToPrimitive {
    /// Gets the number of dimensions.
    fn dim() -> usize;
    /// Gets the Hamming distance to the other sketch.
    fn hamdist(self, rhs: Self) -> usize;
    /// Produces a sketch for masking a given bit-position range.
    fn mask(rng: Range<usize>) -> Self;
}

impl Sketch for u8 {
    #[inline(always)]
    fn dim() -> usize {
        8
    }
    #[inline(always)]
    fn hamdist(self, rhs: Self) -> usize {
        (self ^ rhs).count_ones() as usize
    }
    #[inline(always)]
    fn mask(rng: Range<usize>) -> Self {
        debug_assert!(rng.end <= Self::dim());
        if rng.len() == Self::dim() {
            Self::MAX
        } else {
            ((1 << rng.len()) - 1) << rng.start
        }
    }
}

impl Sketch for u16 {
    #[inline(always)]
    fn dim() -> usize {
        16
    }
    #[inline(always)]
    fn hamdist(self, rhs: Self) -> usize {
        (self ^ rhs).count_ones() as usize
    }
    #[inline(always)]
    fn mask(rng: Range<usize>) -> Self {
        debug_assert!(rng.end <= Self::dim());
        if rng.len() == Self::dim() {
            Self::MAX
        } else {
            ((1 << rng.len()) - 1) << rng.start
        }
    }
}

impl Sketch for u32 {
    #[inline(always)]
    fn dim() -> usize {
        32
    }
    #[inline(always)]
    fn hamdist(self, rhs: Self) -> usize {
        (self ^ rhs).count_ones() as usize
    }
    #[inline(always)]
    fn mask(rng: Range<usize>) -> Self {
        debug_assert!(rng.end <= Self::dim());
        if rng.len() == Self::dim() {
            Self::MAX
        } else {
            ((1 << rng.len()) - 1) << rng.start
        }
    }
}

impl Sketch for u64 {
    #[inline(always)]
    fn dim() -> usize {
        64
    }
    #[inline(always)]
    fn hamdist(self, rhs: Self) -> usize {
        (self ^ rhs).count_ones() as usize
    }
    #[inline(always)]
    fn mask(rng: Range<usize>) -> Self {
        debug_assert!(rng.end <= Self::dim());
        if rng.len() == Self::dim() {
            Self::MAX
        } else {
            ((1 << rng.len()) - 1) << rng.start
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_u8() {
        assert_eq!(u8::mask(0..4), 0b00001111);
        assert_eq!(u8::mask(3..6), 0b00111000);
        assert_eq!(u8::mask(4..8), 0b11110000);
        assert_eq!(u8::mask(0..8), 0b11111111);
    }
}
