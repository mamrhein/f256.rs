// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::num::FpCategory;

use crate::{
    abs_bits, exp, f256, math::big_float::Float512, norm_signif_exp, EMAX,
    EMIN, FRACTION_BITS,
};

pub(crate) fn approx_powi(mut base: Float512, mut n: i32) -> Float512 {
    let mut result = Float512::ONE;
    if n < 0 {
        n = -n;
        base = base.recip();
    }
    while n > 0 {
        if n % 2 != 0 {
            result *= base;
        }
        base *= base;
        n /= 2;
    }
    result
}

#[inline(always)]
fn powi(x: &f256, mut n: i32) -> f256 {
    debug_assert!(x.is_finite() && !x.eq_zero());
    debug_assert!(n.abs() > 1);
    // x < 0 => x = -1⋅|x| => xⁿ = (-1)ⁿ⋅|x|ⁿ
    let s = x.sign() * ((n.abs() % 2) == 1) as u32;
    let (m, e) = norm_signif_exp(&abs_bits(x));
    // Now we have |x| = m⋅2ᵉ with 1 <= m < 2 and Eₘᵢₙ-P+1 <= e <= Eₘₐₓ
    // |x|ⁿ = (m⋅2ᵉ)ⁿ = mⁿ⋅(2ᵉ)ⁿ = mⁿ⋅2ᵉⁿ
    // n > 1 => 1 <= mⁿ < 2ⁿ => 2ᵉⁿ <= mⁿ⋅2ᵉⁿ < 2ⁿ⁺ᵉⁿ
    // n < 1 => 2ⁿ < mⁿ <= 1 => 2ⁿ⁺ᵉⁿ < mⁿ⋅2ᵉⁿ <= 2ᵉⁿ
    let mut lim = e.saturating_mul(n);
    if lim > EMAX {
        return [f256::INFINITY, f256::NEG_INFINITY][s as usize];
    }
    if lim < EMIN - FRACTION_BITS as i32 {
        return [f256::ZERO, f256::NEG_ZERO][s as usize];
    }
    // Result is most likely finite.
    let base = Float512::from(x);
    let result = approx_powi(base, n);
    f256::from(&result)
}

impl f256 {
    /// Raises a number to an integer power.
    pub fn powi(&self, n: i32) -> Self {
        // x⁰ = 1 for any x, incl. NaN
        // 1ⁿ = 1 for any n
        if n == 0 || *self == Self::ONE {
            return Self::ONE;
        }
        // x¹ = x for any x, incl. NaN
        if n == 1 {
            return *self;
        }
        // x⁻¹ = 1/x for any x, incl. NaN (note: 1/NaN = NaN)
        if n == -1 {
            return self.recip();
        }
        // This test for special values is redundant, but it reduces the
        // number of tests for normal cases.
        if self.is_special() {
            // NaNⁿ = NaN for n != 0
            if self.is_nan() {
                return Self::NAN;
            }
            // 0ⁿ = 0 for n > 0
            // 0ⁿ = ∞ for n < 0
            if self.eq_zero() {
                return [Self::ZERO, Self::INFINITY][(n < 0) as usize];
            }
            // ∞ⁿ = ∞ for n > 0
            // ∞ⁿ = 0 for n < 0
            // (-∞)ⁿ = ∞ for n > 0 and n is even
            // (-∞)ⁿ = -∞ for n > 0 and n is odd
            // (-∞)ⁿ = 0 for n < 0 and n is even
            // (-∞)ⁿ = -0 for n < 0 and n is odd
            if self.is_infinite() {
                match (self.sign(), n.signum()) {
                    (0, 1) => return Self::INFINITY,
                    (0, -1) => return Self::ZERO,
                    (1, 1) => {
                        return [Self::INFINITY, Self::NEG_INFINITY]
                            [(n & 1 == 1) as usize]
                    }
                    (1, -1) => {
                        return [Self::ZERO, Self::NEG_ZERO]
                            [(n & 1 == 1) as usize]
                    }
                    _ => unreachable!(),
                }
            }
        }
        // self is finite and != 0, n ∉ [-1…1]
        powi(self, n)
    }
}

#[cfg(test)]
mod powi_tests {
    use super::*;
    use crate::{EMAX, FRACTION_BITS};

    #[test]
    fn test_specials() {
        for n in [1, 786, i32::MAX] {
            assert_eq!(f256::INFINITY.powi(n), f256::INFINITY);
            assert_eq!(f256::ZERO.powi(n), f256::ZERO);
            assert!(f256::NAN.powi(n).is_nan());
        }
        for n in [i32::MIN, -328, -1] {
            assert_eq!(f256::INFINITY.powi(n), f256::ZERO);
            assert_eq!(f256::ZERO.powi(n), f256::INFINITY);
            assert!(f256::NAN.powi(n).is_nan());
        }
        for n in [1, 783, i32::MAX] {
            assert_eq!(f256::NEG_INFINITY.powi(n), f256::NEG_INFINITY);
        }
        for n in [4, 780, i32::MAX - 1] {
            assert_eq!(f256::NEG_INFINITY.powi(n), f256::INFINITY);
        }
        for n in [i32::MIN, -328, -8] {
            let z = f256::NEG_INFINITY.powi(n);
            assert!(z.eq_zero());
            assert!(z.is_sign_positive());
        }
        for n in [i32::MIN + 1, -321, -5] {
            let z = f256::NEG_INFINITY.powi(n);
            assert!(z.eq_zero());
            assert!(z.is_sign_negative());
        }
    }

    #[test]
    fn test_n_eq_0() {
        for f in [
            f256::NAN,
            f256::NEG_INFINITY,
            f256::MIN,
            -f256::TEN,
            f256::NEG_ONE,
            f256::NEG_ZERO,
            f256::ZERO,
            f256::MIN_GT_ZERO,
            f256::MIN_POSITIVE,
            f256::EPSILON,
            f256::ONE,
            f256::TWO,
            f256::MAX,
            f256::INFINITY,
        ] {
            assert_eq!(f.powi(0), f256::ONE);
        }
    }

    #[test]
    fn test_overflow() {
        let mut f = f256::MAX.sqrt();
        f += f.ulp();
        assert_eq!(f.powi(2), f256::INFINITY);
        let n = -7;
        let f = f256::from_sign_exp_signif(1, EMAX / n - 1, (0, 1));
        assert_eq!(f.powi(n), f256::NEG_INFINITY);
        let n = 1440;
        let f = f256::from_sign_exp_signif(
            1,
            -54,
            (
                0x00001be93972f42ce76d0fe4549e8709,
                0x822fb626bcccea99631b77b8b3c24781,
            ),
        );
        assert_eq!(f.powi(n), f256::INFINITY);
    }

    #[test]
    fn test_underflow() {
        let mut f =
            f256::MAX.sqrt() * f256::from(2_u128.pow(FRACTION_BITS / 2));
        f += f.ulp();
        assert_eq!(f.powi(-2), f256::ZERO);
        let n = 7;
        let f = f256::from_sign_exp_signif(
            1,
            (EMIN - FRACTION_BITS as i32) / n - 1,
            (0, 1),
        );
        assert_eq!(f.powi(n), f256::NEG_ZERO);
        let mut f = f256::MIN_GT_ZERO.sqrt();
        f -= f.ulp();
        assert_eq!(f.powi(2), f256::ZERO);
    }

    #[test]
    fn test_subnormal_result() {
        let f = f256::MIN_GT_ZERO.sqrt();
        assert_eq!(f.powi(2), f256::MIN_GT_ZERO);
    }

    #[test]
    fn test_int_base_with_n_gt_0() {
        let m = 73_u64;
        let f = f256::from(m);
        let n = 5;
        assert_eq!(f.powi(n), f256::from(m.pow(n as u32)));
        let m = 17_u128;
        let f = f256::from(m);
        let n = 30;
        assert_eq!(f.powi(n), f256::from(m.pow(n as u32)));
    }

    #[test]
    fn test_int_base_with_n_lt_0() {
        let m = 69_u64;
        let f = f256::from(m);
        let n = -4;
        assert_eq!(f.powi(n), f256::from(m.pow(n.unsigned_abs())).recip());
        let m = 7_u128;
        let f = f256::from(m);
        let n = -41;
        assert_eq!(f.powi(n), f256::from(m.pow(n.unsigned_abs())).recip());
    }

    #[test]
    fn test_base_near_1() {
        let f = f256::ONE + f256::EPSILON;
        let f2 = f.square();
        assert_eq!(f.powi(2), f2);
        assert_eq!(f.powi(9), f2.square().square() * f);
    }
}
