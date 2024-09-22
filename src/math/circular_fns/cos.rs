// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{
    approx_cos::approx_cos, approx_sin::approx_sin, reduce::reduce, BigFloat,
    FP509,
};
use crate::{f256, HI_ABS_MASK};

impl f256 {
    /// Computes the cosine of a number (in radians).
    #[inline(always)]
    pub fn cos(&self) -> Self {
        if self.is_special() {
            // x is NAN or infinite => cosine x is NAN
            if (self.bits.hi.0 & HI_ABS_MASK) > Self::MAX.bits.hi.0 {
                return Self::NAN;
            }
            // x = 0 => cosine x = 1
            return Self::ONE;
        }
        // Calculate ⌈|x|/½π⌋ % 4 and |x| % ½π.
        let (quadrant, fx) = reduce(&self.abs());
        // Map result according to quadrant
        match quadrant {
            0 => Self::from(&approx_cos(&fx)),
            1 => -Self::from(&approx_sin(&fx)),
            2 => -Self::from(&approx_cos(&fx)),
            3 => Self::from(&approx_sin(&fx)),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod cos_tests {
    use core::str::FromStr;

    use super::*;
    use crate::consts::FRAC_PI_3;

    #[test]
    fn test_neg_values() {
        // cos(-x) = cos(x)
        for f in [f256::MIN, -FRAC_PI_3, f256::NEG_ONE] {
            assert_eq!(f.cos(), f.abs().cos());
        }
    }

    #[test]
    fn test_very_small_value() {
        // 2.39802116520473056435462995347760591504487406138046915932763406344460743e-36
        let f = f256::from_sign_exp_signif(
            0,
            -355,
            (
                0x0000198008d7e326fca4eaaddac8f3a6,
                0x4033b94a21af2db28e7aa2336d79615f,
            ),
        );
        // 9.99999999999999999999999999999999999999999999999999999999999999999999995e-1
        let cos_f = f256::from_sign_exp_signif(
            0,
            -237,
            (
                0x00001fffffffffffffffffffffffffff,
                0xffffffffffffffffffffffffffffffff,
            ),
        );
        assert_eq!(f.cos(), cos_f);
    }

    #[test]
    fn test_some_lt_2pi() {
        // 3.9622190502335594146985862764820552764739776569793665175285348244614259
        let f = f256::from_sign_exp_signif(
            0,
            -235,
            (
                0x00001fb29fe6c2bb05604696f175f2d5,
                0xf484b7bfe311af1286402ac83b589d66,
            ),
        );
        // -0.681763086181216231966881974885607400311207886837597710945442436355880494
        let cos_f = f256::from_sign_exp_signif(
            1,
            -237,
            (
                0x000015d100d1d896598bcfdb27f38ee5,
                0x5495ad9278b46941fc542a78cf6d980d,
            ),
        );
        assert_eq!(f.cos(), cos_f);
    }
}
