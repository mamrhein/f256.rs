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
        let (quadrant, x1, x2) = rem_frac_pi_2(&BigFloat::from(&self.abs()));
        debug_assert!(x1.abs() < BigFloat::FRAC_PI_4);
        // Convert (x1 + x2) into a fixed-point number with 510-bit-fraction
        // |x1| < ½π => x1.exp <= 0
        let mut fx = FP509::from(&x1);
        fx += &FP509::from(&x2);
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
