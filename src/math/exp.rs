// ---------------------------------------------------------------------------
// Copyright:   (c) 2025 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::num::FpCategory;

use super::{
    bkm::{bkm_e, bkm_l},
    pow::approx_powf,
    Float256, Float512,
};
use crate::math::log::approx_ln;
use crate::{
    abs_bits,
    big_uint::{BigUInt, U256},
    consts, f256, norm_signif_exp, SIGNIFICAND_BITS,
};

// f256::MAX.ln()
// 181704.374500706303191870897247532238261583907221734753336211540408636177
const LN_MAX: f256 = f256 {
    bits: U256::new(
        85075909386780507726367091749182177406,
        126058098762831240896900834817458532725,
    ),
};

pub(crate) fn approx_exp(x: &Float512) -> Float512 {
    // 2⁻²³⁶ <= |x| <= LN_MAX
    let mut m = x.abs();
    let mut e = 0_i32;
    // exp(|x|) = exp(m⋅2ᵉ)
    // |x| <= LN_MAX => |e| <= 18
    // Assure that m < 1.5 as pre-condition for using fn bkm_e.
    if m > Float512::THREE_HALF {
        let ms = m.signif();
        if ms < Float512::THREE_HALF.signif() {
            e = m.exp();
            m = Float512::from(&ms);
        } else {
            e = m.exp() + 1;
            m = Float512::from(&ms).mul_pow2(-1);
        }
    }
    let mut res = match e {
        0 => bkm_e(&m),
        1.. => {
            // e >= 1 => exp(|x|) = exp(m)ⁿ with n =⋅2ᵉ
            let n = 1_i32 << e as u32;
            bkm_e(&m).powi(n)
        }
        -1 => {
            // e = -1 => exp(|x|) = exp(m)ʸ with y = ½ = √(exp(m))
            bkm_e(&m).sqrt()
        }
        _ => {
            // e < -1
            let mut a = Float512::TWO.mul_pow2(e);
            let mut t = bkm_e(&m);
            let w = a * approx_ln(&t);
            t * approx_exp(&w)
        }
    };
    // x < 0 => eˣ = 1/e⁻ˣ
    if x.signum() == -1 {
        res = res.recip();
    }
    res
}

impl f256 {
    /// Returns e^(self), (the exponential function).
    pub fn exp(&self) -> Self {
        // x = 0  or x is subnornal => eˣ = 1
        // x = ∞ => eˣ = ∞
        // x = -∞ => eˣ = 0
        // x is nan => eˣ is nan
        match self.classify() {
            FpCategory::Zero | FpCategory::Subnormal => Self::ONE,
            FpCategory::Infinite => {
                [Self::INFINITY, Self::ZERO][self.sign() as usize]
            }
            FpCategory::Nan => Self::NAN,
            _ => {
                // self is finite and != 0
                if self == &Self::ONE {
                    // x = 1 => eˣ = e
                    return consts::E;
                }
                let self_abs = self.abs();
                if &self_abs < &Self::EPSILON {
                    return Self::ONE;
                }
                if &self_abs > &LN_MAX {
                    return [Self::INFINITY, Self::ZERO]
                        [self.sign() as usize];
                }
                Self::from(&approx_exp(&Float512::from(self)))
            }
        }
    }

    /// Returns e^(self) - 1 in a way that is accurate even if the number is
    /// close to zero.
    pub fn exp_m1(self) -> Self {
        unimplemented!()
    }

    /// Returns 2^(self).
    pub fn exp2(self) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod exp_tests {
    use super::*;
    use crate::big_uint::HiLo;
    use crate::consts::E;
    use core::ops::Neg;

    #[test]
    fn calc_ln_max() {
        let ln_max = f256::MAX.ln();
        assert_eq!(ln_max, LN_MAX);
    }

    #[test]
    fn test_specials() {
        assert!(f256::NAN.exp().is_nan());
        assert_eq!(f256::INFINITY.exp(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.exp(), f256::ZERO);
        assert_eq!(f256::ZERO.exp(), f256::ONE);
        assert_eq!(f256::NEG_ZERO.exp(), f256::ONE);
    }

    #[test]
    fn test_subnormal() {
        assert_eq!(f256::MIN_GT_ZERO.exp(), f256::ONE);
        let mut f = f256::MIN_POSITIVE;
        f = f - f.ulp();
        assert!(f.is_subnormal());
        assert_eq!(f.exp(), f256::ONE);
    }

    #[test]
    fn test_near_zero() {
        assert_eq!(f256::MIN_POSITIVE.exp(), f256::ONE);
        let mut f = f256::EPSILON;
        assert_eq!(f.exp(), f256::ONE + f256::EPSILON);
        f -= f256::EPSILON.ulp();
        assert_eq!(f.exp(), f256::ONE);
        let f = f256::EPSILON / f256::TWO;
        assert_eq!(f.exp(), f256::ONE);
        let mut f = -f256::EPSILON;
        assert_eq!(f.exp(), f256::ONE);
        f += f.ulp();
        assert_eq!(f.exp(), f256::ONE);
    }

    #[test]
    fn test_near_one() {
        assert_eq!(f256::ONE.exp(), E);
        let mut f = f256::ONE + f256::EPSILON;
        assert_eq!(f.exp(), E + E.ulp());
        let mut f = f256::ONE - f256::EPSILON / f256::TWO;
        assert_eq!(f.exp(), E - E.ulp());
    }

    #[test]
    fn test_max() {
        let f = LN_MAX;
        let e = f.exp();
        assert!(e.diff_within_n_bits(&f256::MAX, 17));
    }

    #[test]
    fn test_overflow() {
        let f = LN_MAX + LN_MAX.ulp();
        assert_eq!(f.exp(), f256::INFINITY);
        assert_eq!(f.neg().exp(), f256::ZERO);
    }
}
