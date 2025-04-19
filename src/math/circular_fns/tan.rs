// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{f256, HI_ABS_MASK};

impl f256 {
    /// Computes the tangent of a number (in radians).
    #[inline(always)]
    #[must_use]
    pub fn tan(&self) -> Self {
        if self.is_special() {
            // x is NAN or infinite => tangent x is NAN
            if (self.bits.hi.0 & HI_ABS_MASK) > Self::MAX.bits.hi.0 {
                return Self::NAN;
            }
            // x = 0 => tangent x = 0
            return Self::ZERO;
        }
        // Calculate tangent x = sine x / cosine x.
        let (sin, cos) = self.sin_cos();
        sin / cos
    }
}

#[cfg(test)]
mod tan_tests {
    use core::ops::Neg;

    use super::*;
    use crate::consts::{FRAC_3_PI_2, FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, PI};

    #[test]
    fn test_neg_values() {
        // tan(-x) = -tan(x)
        for f in [f256::MIN, -FRAC_PI_3, f256::NEG_ONE] {
            assert_eq!(f.neg().tan(), f.tan().neg());
        }
    }

    #[test]
    fn test_tan_eq_sin_cos_quot() {
        // tan(x) = sin(x) / cos(x)
        for f in [
            f256::MIN,
            f256::from(-6_f64),
            f256::from(-3.3_f64),
            f256::from(-2.1_f64),
            -FRAC_PI_3,
            f256::ZERO,
            f256::ONE,
            FRAC_PI_4,
            FRAC_PI_2,
            PI,
            FRAC_3_PI_2,
            f256::from(1234567890_u64),
        ] {
            let tan = f.tan();
            let sin = f.sin();
            let cos = f.cos();
            let sin_over_cos = sin / cos;
            assert_eq!(tan, sin_over_cos, "{tan} != {sin_over_cos}");
        }
    }
}
