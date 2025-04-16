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

// f256::MAX.log2() - 2⁻²³⁶
// 262143.999999999999999999999999999999999999999999999999999999999999999999
const LOG2_MAX: f256 = f256 {
    bits: U256::new(
        0x40010fffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
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
                if &self_abs <= &Self::EPSILON {
                    // for very small x, eˣ ≅ 1+x+½x²
                    let x = Float512::from(self);
                    return Self::from(
                        &(Float512::ONE + x + x.square().mul_pow2(-1)),
                    );
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
    pub fn exp_m1(&self) -> Self {
        // x = 0  or x is subnornal => eˣ-1 = 0
        // x = ∞ => eˣ-1 = ∞
        // x = -∞ => eˣ-1 = -1
        // x is nan => eˣ-1 is nan
        match self.classify() {
            FpCategory::Zero | FpCategory::Subnormal => Self::ZERO,
            FpCategory::Infinite => {
                [Self::INFINITY, Self::NEG_ONE][self.sign() as usize]
            }
            FpCategory::Nan => Self::NAN,
            _ => {
                // self is finite and != 0
                if self == &Self::ONE {
                    // x = 1 => eˣ-1 = e-1
                    return consts::E - Self::ONE;
                }
                let self_abs = self.abs();
                if &self_abs <= &Self::EPSILON {
                    // for very small x, eˣ-1 ≅ x+½x²
                    let x = Float512::from(self);
                    return Self::from(&(x + x.square().mul_pow2(-1)));
                }
                if &self_abs > &LN_MAX {
                    return [Self::INFINITY, Self::NEG_ONE]
                        [self.sign() as usize];
                }
                Self::from(
                    &(approx_exp(&Float512::from(self)) - Float512::ONE),
                )
            }
        }
    }

    /// Returns 2^(self).
    pub fn exp2(&self) -> Self {
        const LOG2_MIN: f256 = f256::power_of_two(-510);
        // x = 0  or x is subnornal => 2ˣ = 1
        // x = ∞ => 2ˣ = ∞
        // x = -∞ => 2ˣ = 0
        // x is nan => 2ˣ is nan
        match self.classify() {
            FpCategory::Zero | FpCategory::Subnormal => Self::ONE,
            FpCategory::Infinite => {
                [Self::INFINITY, Self::ZERO][self.sign() as usize]
            }
            FpCategory::Nan => Self::NAN,
            _ => {
                // self is finite and != 0
                if let Ok(e) = i32::try_from(self) {
                    return Self::power_of_two(e);
                }
                let self_abs = self.abs();
                if self_abs < LOG2_MIN {
                    return f256::ONE;
                }
                if &self.abs() > &LOG2_MAX {
                    return [Self::INFINITY, Self::ZERO]
                        [self.sign() as usize];
                }
                // 2ˣ = eʷ with w = x⋅logₑ 2
                Self::from(&approx_exp(
                    &(Float512::from(self) * Float512::LN_2),
                ))
            }
        }
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
        assert!(ln_max.exp().diff_within_n_bits(&f256::MAX, 17));
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
        let mut f = f256::EPSILON.div_pow2(2);
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
    fn test_near_epsilon() {
        let f = f256::EPSILON;
        assert_eq!(f.exp(), f256::ONE + f);
        let g = f - f.ulp().div2();
        assert_eq!(g.exp(), f256::ONE + f);
        let h = f.div2() - f.ulp().div2();
        assert_eq!(h.exp(), f256::ONE);
    }

    #[test]
    fn test_overflow() {
        let f = LN_MAX + LN_MAX.ulp();
        assert_eq!(f.exp(), f256::INFINITY);
        assert_eq!(f.neg().exp(), f256::ZERO);
    }
}

#[cfg(test)]
mod exp_m1_tests {
    use super::*;
    // use crate::big_uint::HiLo;
    use crate::consts::E;
    use core::ops::Neg;

    #[test]
    fn test_specials() {
        assert!(f256::NAN.exp_m1().is_nan());
        assert_eq!(f256::INFINITY.exp_m1(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.exp_m1(), f256::NEG_ONE);
        assert_eq!(f256::ZERO.exp_m1(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.exp_m1(), f256::ZERO);
    }

    #[test]
    fn test_subnormal() {
        assert_eq!(f256::MIN_GT_ZERO.exp_m1(), f256::ZERO);
        let mut f = f256::MIN_POSITIVE;
        f = f - f.ulp();
        assert!(f.is_subnormal());
        assert_eq!(f.exp_m1(), f256::ZERO);
    }

    #[test]
    fn test_near_zero() {
        assert_eq!(f256::MIN_POSITIVE.exp_m1(), f256::MIN_POSITIVE);
        let mut f = f256::EPSILON.div_pow2(2);
        assert_eq!(f.exp_m1(), f);
        f += f.ulp();
        assert_eq!(f.exp_m1(), f);
    }

    #[test]
    fn test_near_one() {
        assert_eq!(f256::ONE.exp_m1(), E - f256::ONE);
        let mut f = f256::ONE + f256::EPSILON;
        assert_eq!(f.exp_m1(), E + E.ulp() - f256::ONE);
        let mut f = f256::ONE - f256::EPSILON.div2();
        assert_eq!(f.exp_m1(), E - E.ulp() - f256::ONE);
    }

    #[test]
    fn test_near_epsilon() {
        let f = f256::EPSILON;
        assert_eq!(f.exp_m1(), f);
        let g = f - f.ulp().div2();
        assert_eq!(g.exp_m1(), f);
        let h = f.div2() - f.ulp().div2();
        assert_eq!(h.exp_m1(), h);
    }

    #[test]
    fn test_overflow() {
        let f = LN_MAX + LN_MAX.ulp();
        assert_eq!(f.exp_m1(), f256::INFINITY);
        assert_eq!(f.neg().exp_m1(), f256::NEG_ONE);
    }
}

#[cfg(test)]
mod exp2_tests {
    use super::*;
    use crate::big_uint::HiLo;
    use crate::consts::E;
    use core::ops::Neg;

    #[test]
    fn calc_log2_max() {
        let mut log2_max = f256::MAX.log2();
        log2_max -= log2_max.ulp().div2();
        assert_eq!(log2_max, LOG2_MAX);
        assert!(log2_max.exp2().diff_within_n_bits(&f256::MAX, 18));
    }

    #[test]
    fn test_specials() {
        assert!(f256::NAN.exp2().is_nan());
        assert_eq!(f256::INFINITY.exp2(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.exp2(), f256::ZERO);
        assert_eq!(f256::ZERO.exp2(), f256::ONE);
        assert_eq!(f256::NEG_ZERO.exp2(), f256::ONE);
    }

    #[test]
    fn test_subnormal() {
        assert_eq!(f256::MIN_GT_ZERO.exp2(), f256::ONE);
        let mut f = f256::MIN_POSITIVE;
        f = f - f.ulp();
        assert!(f.is_subnormal());
        assert_eq!(f.exp2(), f256::ONE);
    }

    #[test]
    fn test_near_zero() {
        assert_eq!(f256::MIN_POSITIVE.exp2(), f256::ONE);
        let mut f = f256::EPSILON.div_pow2(2);
        assert_eq!(f.exp2(), f256::ONE);
        f += f.ulp();
        assert_eq!(f.exp2(), f256::ONE);
    }

    #[test]
    fn test_near_one() {
        assert_eq!(f256::ONE.exp2(), f256::TWO);
        let mut f = f256::ONE + f256::EPSILON;
        assert_eq!(f.exp2(), f256::TWO + f256::TWO.ulp());
        let mut f = f256::ONE - f256::EPSILON.div2();
        assert_eq!(f.exp2(), f256::TWO - f256::TWO.ulp().div2());
    }

    #[test]
    fn test_near_epsilon() {
        let f = f256::EPSILON;
        assert_eq!(f.exp2(), f256::ONE + f);
        let g = f - f.ulp().div2();
        assert_eq!(g.exp2(), f256::ONE + f);
        let h = f.div2() - f.ulp().div2();
        assert_eq!(h.exp2(), f256::ONE);
    }

    #[test]
    fn test_min() {
        let f = -(LOG2_MAX + LOG2_MAX.ulp());
        assert_eq!(f.exp2(), f256::MIN_POSITIVE.div_pow2(2));
    }

    #[test]
    fn test_overflow() {
        let f = LOG2_MAX + LOG2_MAX.ulp();
        assert_eq!(f.exp2(), f256::INFINITY);
    }
}
