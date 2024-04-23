// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{
    approx_cos::approx_cos, approx_sin::approx_sin, reduce::rem_frac_pi_2,
    BigFloat, FP509,
};
use crate::{f256, HI_ABS_MASK};

impl f256 {
    /// Computes the sine of a number (in radians).
    #[inline(always)]
    pub fn sin(&self) -> Self {
        if self.is_special() {
            // x is NAN or infinite => sine x is NAN
            if (self.bits.hi & HI_ABS_MASK) > f256::MAX.bits.hi {
                return f256::NAN;
            }
            // x = 0 => sine x = 0
            return f256::ZERO;
        }
        // Calculate ⌈x/½π⌋ % 4 and x % ½π.
        let (quadrant, x1, x2) = rem_frac_pi_2(&BigFloat::from(self));
        debug_assert!(x1.abs() < BigFloat::FRAC_PI_4);
        // Convert (x1 + x2) into a fixed-point number with 509-bit-fraction
        // |x1| < ½π => x1.exp <= 0
        let mut fx = FP509::from(&x1);
        fx += &FP509::from(&x2);
        // Map result according to quadrant and sign
        match (quadrant, self.sign()) {
            (0, 0) => Self::from(&approx_sin(&fx)),
            (0, 1) => -Self::from(&approx_sin(&fx)),
            (1, 0) => Self::from(&approx_cos(&fx)),
            (1, 1) => -Self::from(&approx_cos(&fx)),
            (2, 0) => -Self::from(&approx_sin(&fx)),
            (2, 1) => Self::from(&approx_sin(&fx)),
            (3, 0) => -Self::from(&approx_cos(&fx)),
            (3, 1) => Self::from(&approx_cos(&fx)),
            _ => unreachable!(),
        }
    }
}
