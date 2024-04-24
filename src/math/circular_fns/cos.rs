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
            if (self.bits.hi & HI_ABS_MASK) > f256::MAX.bits.hi {
                return f256::NAN;
            }
            // x = 0 => cosine x = 1
            return f256::ONE;
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
    use super::*;
    use crate::consts::FRAC_PI_3;

    #[test]
    fn test_neg_values() {
        // cos(-x) = cos(x)
        for f in [f256::MIN, -FRAC_PI_3, f256::NEG_ONE] {
            assert_eq!(f.cos(), f.abs().cos());
        }
    }
}
