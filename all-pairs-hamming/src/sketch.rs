use std::ops::Range;
use std::usize;

use num_traits::int::PrimInt;
use num_traits::{FromPrimitive, ToPrimitive};

pub trait Sketch: Default + PrimInt + FromPrimitive + ToPrimitive {
    fn dim() -> usize;
    fn hamdist(self, rhs: Self) -> usize;
    fn mask(rng: Range<usize>) -> Self;
}

impl Sketch for u8 {
    fn dim() -> usize {
        8
    }
    fn hamdist(self, rhs: Self) -> usize {
        (self ^ rhs).count_ones() as usize
    }
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
    fn dim() -> usize {
        16
    }
    fn hamdist(self, rhs: Self) -> usize {
        (self ^ rhs).count_ones() as usize
    }
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
    fn dim() -> usize {
        32
    }
    fn hamdist(self, rhs: Self) -> usize {
        (self ^ rhs).count_ones() as usize
    }
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
    fn dim() -> usize {
        64
    }
    fn hamdist(self, rhs: Self) -> usize {
        (self ^ rhs).count_ones() as usize
    }
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
