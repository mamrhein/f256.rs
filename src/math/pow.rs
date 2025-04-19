// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::num::FpCategory;

use super::bkm::{bkm_e, bkm_l};
use super::exp::approx_exp;
use super::log::approx_ln;
use super::{BigUInt, Float512, HiLo, Parity};
use crate::{
    abs_bits, exp, f256, norm_signif_exp, EMAX, EMIN, FRACTION_BITS,
};

enum Lim {
    Ok,
    Overflow,
    Underflow,
}

impl Lim {
    // Check the range of exponents n where xⁿ is guarantied to be infinite.
    #[inline(always)]
    #[allow(clippy::cast_possible_wrap)]
    fn powi(x: &f256, n: i32) -> Self {
        let (m, mut e) = norm_signif_exp(&abs_bits(x));
        // Now we have |x| = m⋅2ᵉ with 1 <= m < 2 and Eₘᵢₙ-P+1 <= e <= Eₘₐₓ
        // |x|ⁿ = (m⋅2ᵉ)ⁿ = mⁿ⋅(2ᵉ)ⁿ = mⁿ⋅2ᵉⁿ
        // n > 1 => 1 <= mⁿ < 2ⁿ => 2ᵉⁿ <= mⁿ⋅2ᵉⁿ < 2ⁿ⁺ᵉⁿ
        // n < 1 => 2ⁿ < mⁿ <= 1 => 2ⁿ⁺ᵉⁿ < mⁿ⋅2ᵉⁿ <= 2ᵉⁿ
        const LOWER_LIM: i32 = -EMAX - 1 - FRACTION_BITS as i32;
        const UPPER_LIM: i32 = EMAX + 1;
        let e_times_n = e.saturating_mul(n);
        match e_times_n {
            ..=LOWER_LIM => Self::Underflow,
            UPPER_LIM.. => Self::Overflow,
            _ => Self::Ok,
        }
    }

    // Check the range of exponents n where xʸ is guarantied to be infinite.
    #[inline(always)]
    fn powf(x: &f256, y: &f256) -> Self {
        match i32::try_from(&y.trunc()) {
            Ok(n) => Self::powi(x, n),
            Err(_) => Self::powi(x, [i32::MAX, i32::MIN][y.sign() as usize]),
        }
    }
}

// Calculate xⁿ
#[inline(always)]
fn powi(x: &f256, mut n: i32) -> f256 {
    debug_assert!(x.is_finite() && !x.eq_zero());
    debug_assert!(n.abs() > 1);
    // x < 0 => x = -1⋅|x| => xⁿ = (-1)ⁿ⋅|x|ⁿ
    let s = x.sign() * ((n.abs() % 2) == 1) as u32;
    match Lim::powi(x, n) {
        Lim::Overflow => [f256::INFINITY, f256::NEG_INFINITY][s as usize],
        Lim::Underflow => [f256::ZERO, f256::NEG_ZERO][s as usize],
        _ => {
            // Result is most likely finite.
            f256::from(&Float512::from(x).powi(n))
        }
    }
}

pub(crate) fn approx_powf(mut x: Float512, mut y: Float512) -> Float512 {
    debug_assert!(y.abs() < Float512::from(i32::MAX));
    if y.signum() == -1 {
        x = x.recip();
        y.flip_sign();
    }
    // 0 < y < 2³¹
    // y = a + b where a ∈ ℤ and 0 <= b < 1
    // xʸ = xᵅ⁺ᵇ = xᵅ⋅xᵇ = xᵅ⋅eʷ where w = b⋅logₑ x
    // println!("x = {:e}", f256::from(&x));
    // println!("y = {:e}", f256::from(&y));
    let a = y.trunc();
    // println!("a = {:e}", f256::from(&a));
    let b = y - a;
    // println!("b = {:e}", f256::from(&b));
    let mut a = i32::try_from(&a).unwrap();
    // println!("a = {a}");
    let lnx = approx_ln(&x);
    // println!("l = {lnx:?}");
    let w = b * lnx;
    // println!("w = {w:?}");
    let ew = approx_exp(&w);
    // println!("ew = {:?}", ew);
    x.powi(a) * ew
}

// Compute xʸ
#[inline(always)]
fn powf(x: &f256, y: &f256) -> f256 {
    debug_assert!(x.is_finite() && !x.eq_zero());
    debug_assert!(y.is_finite() && !y.eq_zero());
    let x_sign = x.sign();
    if let Ok(n) = i32::try_from(y) {
        let s: usize = match (x_sign, (n % 2) == 1) {
            (0, _) | (1, false) => 0,
            (1, true) => 1,
            _ => unreachable!(),
        };
        return match Lim::powi(x, n) {
            Lim::Overflow => [f256::INFINITY, f256::NEG_INFINITY][s],
            Lim::Underflow => [f256::ZERO, f256::NEG_ZERO][s],
            _ => powi(x, n),
        };
    };
    if let Some(p) = y.parity() {
        // y ∈ ℤ and |y| >= 2³¹
        let s = match (x_sign, p) {
            (0, _) | (1, Parity::Even) => 0,
            (1, Parity::Odd) => 1,
            _ => unreachable!(),
        };
        return match Lim::powi(x, [i32::MAX, i32::MIN][y.sign() as usize]) {
            Lim::Overflow => [f256::INFINITY, f256::NEG_INFINITY][s],
            Lim::Underflow => [f256::ZERO, f256::NEG_ZERO][s],
            _ => unreachable!(),
        };
    }
    // y ∉ ℤ
    if x_sign == 1 {
        // xʸ = NaN for x < 0 and non-integer y
        return f256::NAN;
    };
    match Lim::powf(x, y) {
        Lim::Overflow => {
            [f256::INFINITY, f256::NEG_INFINITY][x_sign as usize]
        }
        Lim::Underflow => [f256::ZERO, f256::NEG_ZERO][x_sign as usize],
        _ => {
            // Result is most likely finite.
            f256::from(&approx_powf(Float512::from(x), Float512::from(y)))
        }
    }
}

impl f256 {
    /// Raises a number to an integer power.
    #[must_use]
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

    /// Raises a number to a floating point power.
    #[must_use]
    pub fn powf(&self, exp: &Self) -> Self {
        // a⁰ = 1 for any a, incl. NaN
        // 1ᵇ = 1 for any b, incl. NaN
        if exp.eq_zero() || *self == Self::ONE {
            return Self::ONE;
        }
        // a¹ = a for any a, incl. NaN
        if *exp == Self::ONE {
            return *self;
        }
        // a⁻¹ = 1/a for any a, incl. NaN (note: 1/NaN = NaN)
        if *exp == Self::NEG_ONE {
            return self.recip();
        }
        // This test for special values is redundant, but it reduces the
        // number of tests for normal cases.
        if self.is_special() || exp.is_special() {
            // aᴺᵃᴺ = NaN for a != 1
            // NaNᵇ = NaN for |b| != 0
            if self.is_nan() || exp.is_nan() {
                return Self::NAN;
            }
            // 0ᵇ = 0 for b > 0
            // 0ᵇ = ∞ for b < 0
            if self.eq_zero() {
                return [Self::ZERO, Self::INFINITY][exp.sign() as usize];
            }
            // ∞ᵇ = ∞ for b > 0
            // ∞ᵇ = 0 for b < 0
            // (-∞)ᵇ = ∞ for b > 0 and (b ∉ ℕ or b ∈ ℕ and b is even)
            // (-∞)ᵇ = -∞ for b > 0 and b ∈ ℕ and b is odd
            // (-∞)ᵇ = 0 for b < 0 and (|b| ∉ ℕ or |b| ∈ ℕ and |b| is even)
            // (-∞)ᵇ = -0 for b < 0 and |b| ∈ ℕ and |b| is odd
            if self.is_infinite() {
                return match (self.sign(), exp.sign()) {
                    (0, 0) => Self::INFINITY,
                    (0, 1) => Self::ZERO,
                    (1, 0) => match exp.parity() {
                        Some(Parity::Odd) => Self::NEG_INFINITY,
                        _ => Self::INFINITY,
                    },
                    (1, 1) => match exp.parity() {
                        Some(Parity::Odd) => Self::NEG_ZERO,
                        _ => Self::ZERO,
                    },
                    _ => unreachable!(),
                };
            }
        }
        // self is finite and != 0, exp is finite and ∉ [-1, 0, 1]
        powf(self, exp)
    }
}

#[cfg(test)]
mod powi_tests {
    use super::*;
    use crate::{EMAX, FRACTION_BITS, SIGNIFICAND_BITS};

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
        let mut f = f256::MIN_GT_ZERO.sqrt().div2();
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

    #[test]
    fn test_base_near_minus_1() {
        let f = f256::NEG_ONE + f256::EPSILON;
        let f2 = f.square();
        assert_eq!(f.powi(2), f2);
        assert_eq!(f.powi(5), f2.square() * f);
    }

    #[test]
    fn test_max() {
        let f = f256::TWO;
        let p =
            (f.powi(EMAX) - f.powi(EMAX - SIGNIFICAND_BITS as i32)).mul2();
        assert_eq!(p, f256::MAX);
    }

    #[test]
    fn test_near_max() {
        let mut f = f256::TWO;
        f -= f.ulp();
        let p = f.powi(f256::MAX_EXP);
        assert!(p.diff_within_n_bits(&f256::MAX, 19));
    }
}

#[cfg(test)]
mod powf_tests {
    use super::*;
    use crate::{EMAX, FIVE, FRACTION_BITS, ONE_HALF};

    #[test]
    fn test_specials() {
        let g = f256::from(123.45_f64);
        let h = f256::TWO.powi(236) - f256::ONE;
        for b in [f256::MIN_GT_ZERO, f256::ONE, g, f256::MAX] {
            // ∞ᵇ = ∞ for b > 0
            assert_eq!(f256::INFINITY.powf(&b), f256::INFINITY);
            // 0ᵇ = 0 for b > 0
            assert_eq!(f256::ZERO.powf(&b), f256::ZERO);
            // NaNᵇ = NaN for b != 0
            assert!(f256::NAN.powf(&b).is_nan());
        }
        for b in [f256::MIN, -g, f256::NEG_ONE, -f256::MIN_POSITIVE] {
            // ∞ᵇ = 0 for b < 0
            assert_eq!(f256::INFINITY.powf(&b), f256::ZERO);
            // 0ᵇ = ∞ for b < 0
            assert_eq!(f256::ZERO.powf(&b), f256::INFINITY);
            // NaNᵇ = NaN for b != 0
            assert!(f256::NAN.powf(&b).is_nan());
        }
        for b in [f256::ONE, FIVE, h] {
            // (-∞)ᵇ = -∞ for b > 0 and b ∈ ℕ and b is odd
            assert_eq!(f256::NEG_INFINITY.powf(&b), f256::NEG_INFINITY);
        }
        for b in [f256::TWO, g, f256::MAX] {
            // (-∞)ᵇ = ∞ for b > 0 and (b ∉ ℕ or b ∈ ℕ and b is even)
            assert_eq!(f256::NEG_INFINITY.powf(&b), f256::INFINITY);
        }
        for b in [f256::MIN, -g, -f256::TEN] {
            // (-∞)ᵇ = 0 for b < 0 and (|b| ∉ ℕ or |b| ∈ ℕ and |b| is even)
            let z = f256::NEG_INFINITY.powf(&b);
            assert!(z.eq_zero());
            assert!(z.is_sign_positive());
        }
        for b in [-h, -FIVE] {
            // (-∞)ᵇ = -0 for b < 0 and |b| ∈ ℕ and |b| is odd
            let z = f256::NEG_INFINITY.powf(&b);
            assert!(z.eq_zero());
            assert!(z.is_sign_negative());
        }
        for a in [
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
            f256::TWO,
            f256::MAX,
            f256::INFINITY,
        ] {
            // aᴺᵃᴺ = NaN for a != 1
            assert!(a.powf(&f256::NAN).is_nan());
        }
        for b in [
            f256::NAN,
            f256::NEG_INFINITY,
            f256::MIN,
            -f256::TEN,
            f256::NEG_ONE,
            f256::MIN_GT_ZERO,
            f256::MIN_POSITIVE,
            f256::EPSILON,
            f256::ONE,
            f256::TWO,
            f256::MAX,
            f256::INFINITY,
        ] {
            // NaNᵇ = NaN for |b| != 0
            assert!(f256::NAN.powf(&b).is_nan());
        }
        for x in [f256::MIN, -g, f256::NEG_ONE, -f256::MIN_POSITIVE] {
            // xʸ = NaN for x < 0 and non-integer y
            assert!(x.powf(&g).is_nan());
        }
    }

    #[test]
    fn test_a_pow_0() {
        for a in [
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
            // a⁰ = 1 for any a, incl. NaN
            assert_eq!(a.powf(&f256::ZERO), f256::ONE);
            assert_eq!(a.powf(&f256::NEG_ZERO), f256::ONE);
        }
    }

    #[test]
    fn test_a_pow_1() {
        // a¹ = a for any a, incl. NaN
        for a in [
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
            assert_eq!(a.powf(&f256::ONE), a);
        }
        assert!(f256::NAN.powf(&f256::ONE).is_nan());
    }

    #[test]
    fn test_1_pow_b() {
        for b in [
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
            // 1ᵇ = 1 for any b, incl. NaN
            assert_eq!(f256::ONE.powf(&b), f256::ONE);
        }
    }

    #[test]
    fn test_overflow() {
        let mut x = f256::MAX.sqrt();
        x += x.ulp();
        assert_eq!(x.powf(&f256::TWO), f256::INFINITY);
        let n = -7;
        let mut y = f256::from(n);
        y -= y.ulp();
        let x = f256::from_sign_exp_signif(0, EMAX / n - 1, (0, 1));
        assert_eq!(x.powf(&y), f256::INFINITY);
        let y = f256::from(1439.907);
        let x = f256::from_sign_exp_signif(
            0,
            -54,
            (
                0x00001be93972f42ce76d0fe4549e8709,
                0x822fb626bcccea99631b77b8b3c24781,
            ),
        );
        assert_eq!(x.powf(&y), f256::INFINITY);
    }

    #[test]
    fn test_underflow() {
        let mut x =
            f256::MAX.sqrt() * f256::from(2_u128.pow(FRACTION_BITS / 2));
        x += x.ulp();
        assert_eq!(x.powf(&-f256::TWO), f256::ZERO);
        let n = 7;
        let mut y = f256::from(n);
        y += y.ulp();
        let x = f256::from_sign_exp_signif(
            0,
            (EMIN - FRACTION_BITS as i32) / n - 1,
            (0, 1),
        );
        assert_eq!(x.powf(&y), f256::ZERO);
        let x = f256::MIN_GT_ZERO.sqrt().div2();
        let mut y = f256::TWO;
        y -= y.ulp();
        assert_eq!(x.powf(&y), f256::ZERO);
    }

    #[test]
    fn test_subnormal_result() {
        let x = f256::MIN_GT_ZERO.sqrt();
        let mut y = f256::TWO;
        y -= y.ulp();
        assert_eq!(x.powf(&y), f256::MIN_GT_ZERO);
        let x = f256::EPSILON;
        let y = f256::from(1111.1);
        let z = x.powf(&y);
        assert_eq!(
            z,
            f256::from_sign_exp_signif(
                0,
                -262377,
                (
                    0x0000000000000000000000002a3968a7,
                    0x75598fceb5d84852b84918221a2a837f,
                )
            )
        );
    }

    #[test]
    fn test_base_near_1() {
        let x = f256::ONE + f256::EPSILON;
        let z = x.square();
        let mut y = f256::TWO;
        y -= z.ulp();
        assert_eq!(x.powf(&y), z);
        assert_eq!(x.powf(&f256::from(9)), z.square().square() * x);
    }

    #[test]
    fn test_base_near_minus_1() {
        let x = f256::NEG_ONE + f256::EPSILON;
        let z = x.square();
        let mut y = f256::TWO;
        y -= z.ulp();
        assert_eq!(x.powf(&y), z);
        assert_eq!(x.powf(&f256::from(5)), z.square() * x);
    }

    #[test]
    fn test_int_base_non_int_exp() {
        let x = FIVE;
        assert_eq!(x.powf(&f256::from(2.5_f64)), FIVE.square() * FIVE.sqrt());
    }

    #[test]
    fn test_non_int_base_non_int_exp() {
        let x = FIVE - f256::EPSILON.mul_pow2(7);
        assert_eq!(
            x.powf(&f256::from(3.25_f64)),
            x.powi(3) * x.sqrt().sqrt()
        );
    }
}
