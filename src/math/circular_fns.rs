// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::max,
    ops::{Div, Neg, Rem, Shl, Shr},
};

use crate::{
    abs_bits, abs_bits_sticky,
    big_uint::u256,
    BinEncAnySpecial,
    consts::{FRAC_PI_2, FRAC_PI_4, PI},
    EXP_BIAS,
    f256, FRACTION_BITS, HI_ABS_MASK, HI_EXP_MASK, HI_FRACTION_BITS,
    math::{approx_cos::approx_cos, approx_sin::approx_sin, fp509::FP509}, sign_bits_hi,
};

use super::BigFloat;

// Number of bits to shift left for adjusting the radix point from f256 to
// FP255
const PREC_ADJ: u32 = BigFloat::FRACTION_BITS - FRACTION_BITS;

// Cut-off for small values
// ≈0.00000000000000000000000000000000000210094754024801845063812748106760843
const SMALL_CUT_OFF: u256 = u256::new(
    0x3ff8865752be2a167f0644b50757a602,
    0x81800000000000000000000000000000,
);

// Cut-off of exponent for large values
const LARGE_EXP_CUT_OFF: u32 = 240;
// Cut-off for large values (2²⁴⁰)
const LARGE_CUT_OFF: u256 = u256::new(
    ((EXP_BIAS + LARGE_EXP_CUT_OFF) as u128) << HI_FRACTION_BITS,
    0_u128,
);

// fn fastest_reduce(x: &BigFloat) -> (u32, BigFloat, BigFloat) {
//     let q = (x * FRAC_2_PI).round_tie_even();
//     let r = *x - q * FRAC_PI_2;
//     let m = signif(&q.bits);
//     let e = exp_bits(&q.bits) as i32 - EXP_BIAS as i32 - FRACTION_BITS as
// i32;     let q = (&m >> e.unsigned_abs()).lo as u32 & 0x3;
//     (q, r)
// }

// Accurate range reduction algorithm, adapted from
// S. Boldo, M. Daumas, R.-C. Li,
// Formally verified argument reduction with a fused multiply-add
// IEEE Trans. Comput. 58(8), 1139–1145 (2009)
// For the input value f, calculate ⌊f/½π⌋ and f%½π
fn fast_reduce(x: &BigFloat) -> (u32, BigFloat, BigFloat) {
    // R = ◯₂₅₅(1/½π) =
    // 0.6366197723675813430755350534900574481378385829618257949906693762355871905369
    const R: BigFloat = BigFloat::new(
        0x517cc1b727220a94fe13abe8fa9a6ee0,
        0x6db14acc9e21c820ff28b1d5ef5de2b1,
        -1,
    );
    // C = ◯₂₅₅(1/R) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    // C1 = ◯₂₅₃(C) =
    // 1.57079632679489661923132169163975144209858469968755291048747229615390820314306
    const C1: BigFloat = BigFloat::new(
        0x6487ed5110b4611a62633145c06e0e68,
        0x948127044533e63a0105df531d89cd90,
        0,
    );
    // C2 = ⌈(C - C1) / 8⋅ulp(ulp(C1))⌋ ⋅ 8⋅ulp(ulp(C1)) =
    // 4.0029261425274538885256060583180088389717792640288565295989842465635080655216e-77
    const C2: BigFloat = BigFloat::new(
        0x4a29410f31c6809bbdf2a33679a74863,
        0x6605614dbe4be286e9fc26adadaa3848,
        -254,
    );
    // D = 3⋅2²⁵³ =
    // 43422033463993573283839119378257965444976244249615211514796594002967423614976
    const D: BigFloat = BigFloat::new(
        0x60000000000000000000000000000000,
        0x00000000000000000000000000000000,
        254,
    );

    let z = x.mul_add(&R, &D) - &D;
    // let z = (*x * &R).trunc();
    // debug_assert_eq!(z.abs(), zz.abs());
    let u = *x - &(z * &C1);
    let v1 = u - &(z * &C2);
    let (p1, p2) = z.mul_exact(&C2);
    let (t1, t2) = u.sum_exact(&p1.neg());
    let v2 = ((t1 - &v1) + &t2) - &p2;
    // x <= M => z < 2ᴾ⁻²
    let e = z.exp - BigFloat::FRACTION_BITS as i32;
    let q = (&z.signif >> e.unsigned_abs()).lo as u32 & 0x3;
    (q, v1, v2)
}

// Max input value for fast_reduce
// M = ◯₂₅₅((2²⁵³-1)⋅C) =
// 22735723555735395267514683923608116415837086083024526872423259444871180904135
const M: BigFloat = BigFloat::new(
    0x6487ed5110b4611a62633145c06e0e68,
    0x948127044533e63a0105df531d89cd8e,
    253,
);

fn rem_frac_pi_2(x: &BigFloat) -> (u32, BigFloat, BigFloat) {
    let x_abs = x.abs();
    if &x_abs <= &BigFloat::FRAC_PI_4 {
        (0, *x, BigFloat::ZERO)
    } else if &x_abs < &BigFloat::FRAC_3_PI_4 {
        let mut y = BigFloat::FRAC_PI_2;
        y.copy_sign(x);
        y.flip_sign();
        let (hi, lo) = x.sum_exact(&y);
        (1, hi, lo)
    } else if &x_abs <= &BigFloat::FRAC_5_PI_4 {
        let mut y = BigFloat::PI;
        y.copy_sign(x);
        y.flip_sign();
        let (hi, lo) = x.sum_exact(&y);
        (2, hi, lo)
    } else if &x_abs < &BigFloat::FRAC_7_PI_4 {
        let mut y = BigFloat::FRAC_3_PI_2;
        y.copy_sign(x);
        y.flip_sign();
        let (hi, lo) = x.sum_exact(&y);
        (3, hi, lo)
    } else if &x_abs <= &BigFloat::FRAC_9_PI_4 {
        let mut y = BigFloat::TAU;
        y.copy_sign(x);
        y.flip_sign();
        let (hi, lo) = x.sum_exact(&y);
        (0, hi, lo)
    } else if &x_abs <= &M {
        fast_reduce(x)
    } else {
        // M < x <= f256::MAX
        const D: u256 = BigFloat::TAU.signif;
        // x >= TAU => exp(x) >= 2 => following expression can't be < 0
        let sh = x.exp as u32 - 2;
        let mut t = x.signif;
        t = t.lshift_rem(&D, sh);
        if t.is_zero() {
            return (0, BigFloat::ZERO, BigFloat::ZERO);
        }
        let shl = t.leading_zeros() - 1;
        t <<= shl;
        let u = BigFloat::new(
            t.hi as i128,
            t.lo,
            -((BigFloat::FRACTION_BITS + shl - 2) as i32),
        );
        debug_assert!(u.abs() < BigFloat::TAU);
        debug_assert!(u.abs() <= x.abs());
        rem_frac_pi_2(&u)
    }
}

impl f256 {
    /// Simultaneously computes the sine and cosine of the number x
    /// (in radians).
    ///
    /// Returns (sin(x), cos(x)).
    pub fn sin_cos(&self) -> (Self, Self) {
        (self.sin(), self.cos())
    }

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

    /// Computes the arctangent of a number (in radians).
    ///
    /// Return value is in radians in the range [-½π, ½π].
    pub fn atan(&self) -> Self {
        let abs_bits_self = abs_bits(self);
        // If self is NAN, atan self is NAN.
        if (abs_bits_self.hi | (abs_bits_self.lo != 0) as u128) > HI_EXP_MASK
        {
            return f256::NAN;
        }
        // If |self| >= 2²⁴⁰, atan self = ±½π.
        if abs_bits_self.hi >= LARGE_CUT_OFF.hi {
            let mut res = FRAC_PI_2;
            res.bits.hi ^= sign_bits_hi(self);
            return res;
        }
        // If |self| is very small, atan self = self.
        if abs_bits_self <= SMALL_CUT_OFF {
            return *self;
        }
        // Now we have ε < |self| < 2²⁴⁰.
        f256::from(&BigFloat::from(self).atan())
    }

    /// Computes the four quadrant arctangent of `self` (`y`) and `other`
    /// (`x`) in radians.
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-½π, ½π]`
    /// * `y >= 0`: `arctan(y/x) + π` -> `(½π, π]`
    /// * `y < 0`: `arctan(y/x) - π` -> `(-π, -½π)`
    pub fn atan2(&self, other: &Self) -> Self {
        let mut abs_bits_x = abs_bits(&other);
        let mut abs_bits_y = abs_bits(&self);
        // Check whether one or both operands are NaN, infinite or zero.
        // We mask off the sign bit and mark subnormals having a significand
        // less than 2¹²⁸ in least bit of the representations high
        // u128. This allows to use only that part for the handling of
        // special cases.
        let abs_bits_sticky_x = abs_bits_sticky(&abs_bits_x);
        let abs_bits_sticky_y = abs_bits_sticky(&abs_bits_y);
        if (abs_bits_sticky_x, abs_bits_sticky_y).any_special() {
            if max(abs_bits_sticky_x, abs_bits_sticky_y) > HI_EXP_MASK {
                // Atleast one operand is NAN.
                return f256::NAN;
            }
            if abs_bits_sticky_x == 0_u128 {
                return if abs_bits_sticky_y == 0 {
                    // Both operands are zero.
                    f256::ZERO
                } else {
                    // other = 0, self != 0 => ±½π
                    let mut res = FRAC_PI_2;
                    res.bits.hi |= sign_bits_hi(&self);
                    res
                };
            }
            if abs_bits_sticky_y == 0_u128 {
                // self = 0, other > 0 => 0
                // self = 0, other < 0 => π
                return [f256::ZERO, PI][other.sign() as usize];
            }
            // Both operands are infinite.
            return match (self.sign(), other.sign()) {
                (0, 0) => FRAC_PI_4,
                // TODO: replace by constant FRAC_3_PI_2
                (0, 1) => &PI - &FRAC_PI_4,
                (1, 0) => -FRAC_PI_4,
                _ => &FRAC_PI_4 - &PI,
            };
        }

        // Both operands are finite and non-zero.

        let y = BigFloat::from(self);
        let x = BigFloat::from(other);
        f256::from(&y.atan2(&x))
    }
}

#[cfg(test)]
mod sin_cos_tests {
    use core::str::FromStr;

    use crate::{
        consts::{FRAC_PI_3, FRAC_PI_4, FRAC_PI_6},
        ONE_HALF,
    };

    use super::*;

    //noinspection DuplicatedCode
    #[test]
    fn test_frac_pi_2_multiples() {
        const EXACT: [(f256, f256); 4] = [
            (f256::ZERO, f256::ONE),
            (f256::ONE, f256::ZERO),
            (f256::ZERO, f256::NEG_ONE),
            (f256::NEG_ONE, f256::ZERO),
        ];
        for i in 0_u32..=4_u32 {
            let eps = f256::EPSILON.mul_pow2(i as i32 / 4);
            let f = f256::from(i) * FRAC_PI_2;
            let (sin, cos) = f.sin_cos();
            let (exact_sin, exact_cos) = EXACT[(i % 4) as usize];
            if exact_sin.eq_zero() {
                let d = (exact_sin - sin).abs();
                assert!(d < eps, "{d:?} >= {eps:?}");
            } else {
                assert_eq!(exact_sin, sin);
            }
            if exact_cos.eq_zero() {
                let d = (exact_cos - cos).abs();
                assert!(d < eps, "{d:?} >= {eps:?}");
            } else {
                assert_eq!(exact_cos, cos);
            }
        }
    }

    #[test]
    fn test_signs() {
        let p = FRAC_PI_4;
        for i in 0..9 {
            let f = f256::from(i) * FRAC_PI_2 + p;
            let (sin, cos) = f.sin_cos();
            let quadrant = i % 4;
            match quadrant {
                0 => {
                    assert!(sin.is_sign_positive());
                    assert!(cos.is_sign_positive());
                }
                1 => {
                    assert!(sin.is_sign_positive());
                    assert!(cos.is_sign_negative());
                }
                2 => {
                    assert!(sin.is_sign_negative());
                    assert!(cos.is_sign_negative());
                }
                3 => {
                    assert!(sin.is_sign_negative());
                    assert!(cos.is_sign_positive());
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_frac_pi_4() {
        // sin(45°) = cos(45°)
        let f = FRAC_PI_4;
        let (sin, cos) = f.sin_cos();
        assert!((sin - cos).abs() <= sin.ulp());
    }

    #[test]
    fn test_frac_pi_3_and_frac_pi_6() {
        // sin(30°) = 0.5
        let sin = FRAC_PI_6.sin();
        assert_eq!(sin, ONE_HALF);
        // cos(60°) = 0.5
        let cos = FRAC_PI_3.cos();
        assert_eq!(cos, ONE_HALF);
        // sin(60°) = cos(30°)
        let sin = FRAC_PI_3.sin();
        let cos = FRAC_PI_6.cos();
        assert_eq!(sin, cos);
    }

    #[test]
    fn test_some_lt_2pi() {
        let f = f256::from_sign_exp_signif(
            0,
            -261,
            (
                0x0000100410f1f3ab981fc5a9fd008e6e,
                0x6ba97c4190d331836d7fd41d2009cdf8,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            0,
            -261,
            (
                0x0000100410f1f3ab977498bfffb5d0d5,
                0xd4afb6f12a8836a249b17fbeb758fa8e,
            ),
        );
        assert_eq!(f.sin(), sin_f);
        let f = f256::from_sign_exp_signif(
            0,
            -235,
            (
                0x000019412990c230cfe83e598062a70f,
                0x2e55ff0ee1b47200750f278655e459cc,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            1,
            -243,
            (
                0x00001f2ded8c6c188d6563858850bd6f,
                0xc0dd632c3566aef3b1af2c6bd810e0fe,
            ),
        );
        assert_eq!(f.sin(), sin_f);
        let f = f256::from_sign_exp_signif(
            0,
            -230,
            (
                0x000001709d10d3e7eab960be165f5516,
                0xe8df7f75d98f0fa868f6d4ae6add8617,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            1,
            -237,
            (
                0x00000fffffffffffffffffffffffffff,
                0xfffffffffffffffffffffffffffffffd,
            ),
        );
        assert_eq!(f.sin(), sin_f, "{f}\n{}\n{}", f.sin(), sin_f);
    }

    #[test]
    fn test_some_gt_2pi() {
        let f = f256::from_sign_exp_signif(
            0,
            -218,
            (
                0x00001b88030ccdd8b7632adb619b1f1f,
                0x0e1d0adefedbcedd03c621b5967e9c1d,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            0,
            -239,
            (
                0x00001ff3a6e68be32dc92aa6c6930521,
                0x192865a8b728d2d42fcb7319995fc955,
            ),
        );
        println!("{f}\n{sin_f}");
        assert_eq!(f.sin(), sin_f);
    }

    #[test]
    fn test_neg_values() {
        for f in [f256::MIN, -FRAC_PI_3, f256::NEG_ONE] {
            assert_eq!(f.sin(), -f.abs().sin());
            assert_eq!(f.cos(), f.abs().cos());
        }
    }

    #[test]
    fn test_continuity_near_zero() {
        let c = f256 {
            bits: SMALL_CUT_OFF,
        };
        let d = f256::encode(0, c.exponent(), u256::new(0, 1));
        let mut f = c;
        let mut g = f;
        for i in 0..1000 {
            g += d;
            assert!(f < g);
            assert!(
                f.sin() <= g.sin(),
                "    f: {}\nsin f: {}\n    g: {}\nsin g: {}",
                f,
                f.sin(),
                g,
                g.sin()
            );
            assert!(
                f.cos() >= g.cos(),
                "    f: {}\ncos f: {}\n    g: {}\ncos g: {}",
                f,
                f.cos(),
                g,
                g.cos()
            );
            f = g;
        }
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_continuity_near_one() {
        let c = f256::ONE;
        let d = f256::EPSILON;
        let mut f = c;
        let mut g = f;
        for i in 0..10 {
            g += d;
            assert!(f < g);
            assert!(f.sin() <= g.sin());
            assert!(f.cos() >= g.cos());
            f = g;
        }
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_continuity_near_pi_half() {
        let c = FRAC_PI_2;
        let d = f256::from_str("1.5e-36").unwrap();
        let mut f = c + d;
        let mut g = f;
        for i in 0..10 {
            g += d;
            assert!(f < g);
            assert!(f.sin() >= g.sin());
            assert!(f.cos() <= g.cos());
            f = g;
        }
        let mut f = c;
        let mut g = f;
        for i in 0..10 {
            g -= d;
            assert!(f > g);
            assert!(f.sin() >= g.sin());
            assert!(f.cos() <= g.cos());
            f = g;
        }
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_continuity_near_three() {
        let c = f256::from(3);
        let d = f256::EPSILON * f256::TWO;
        let mut f = c;
        let mut g = f;
        for i in 0..10 {
            g += d;
            assert!(f < g);
            assert!(f.sin() >= g.sin());
            assert!(f.cos() <= g.cos());
            f = g;
        }
    }

    #[test]
    fn test_small_value() {
        let f = f256::from_sign_exp_signif(
            0,
            -268,
            (
                511713792246730580583350097904921,
                338234285556250629981528767706881153057,
            ),
        );
        let sin = f256::from_sign_exp_signif(
            0,
            -268,
            (
                511713792246730580571854506161847,
                105438061704425261882515718706001931297,
            ),
        );
        assert_eq!(f.sin(), sin);
    }

    // f: 140844820278614289426057198173335166586563126037009815346672127671657710 * 2^185461
    // ε: 5.769198204535869190785720230896528973489817286545990660946235357113661705e-77
    // -log₂(ε): 253.26
}

#[cfg(test)]
mod atan_tests {
    use core::{ops::Neg, str::FromStr};

    use crate::{
        consts::{FRAC_1_PI, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6},
        EPSILON,
    };

    use super::*;

    #[test]
    fn test_atan_inf() {
        assert_eq!(f256::INFINITY.atan(), FRAC_PI_2);
    }

    #[test]
    fn test_atan_large_cutoff() {
        let f = f256 {
            bits: LARGE_CUT_OFF,
        };
        assert_eq!(f.atan(), FRAC_PI_2);
    }

    #[test]
    fn test_atan_zero() {
        assert_eq!(f256::ZERO.atan(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.atan(), f256::ZERO);
    }

    #[test]
    fn test_atan_one() {
        assert_eq!(f256::ONE.atan(), FRAC_PI_4);
        assert_eq!(f256::NEG_ONE.atan(), -FRAC_PI_4);
    }

    #[test]
    fn test_atan_sqrt_3() {
        let t = f256::from(3);
        let mut f = t.sqrt();
        // arctan √3 = ⅓π
        assert_eq!(f.atan(), FRAC_PI_3);
        assert_eq!(f.neg().atan(), -FRAC_PI_3);
        // arctan ⅓√3 = π/6
        f /= t;
        assert_eq!(f.atan(), FRAC_PI_6);
        assert_eq!(f.neg().atan(), -FRAC_PI_6);
    }

    #[test]
    fn test_atan_frac_1_pi() {
        let f1 = FRAC_1_PI.atan();
        let f2 = f256::ONE.atan2(&PI);
        let d = f1 - f2;
        assert!(d.abs() <= EPSILON);
    }

    #[test]
    fn test_atan_frac_pi_2() {
        let s = "1.00388482185388721414842394491713228829210446059487057472971282410801519";
        let a = f256::from_str(s).unwrap();
        let f1 = FRAC_PI_2.atan();
        assert_eq!(f1, a);
        let f2 = PI.atan2(&f256::TWO);
        assert_eq!(f1, f2);
    }

    #[test]
    fn test_atan_frac_5_pi_4() {
        let s = "1.32144796778372235539166569069508390109061014033053361477468861418765787";
        let a = f256::from_str(s).unwrap();
        let f = PI + FRAC_PI_4;
        assert_eq!(f.atan(), a);
        let f = f256::TEN * PI;
        assert_eq!(f.atan2(&f256::from(8_f64)), a);
    }

    #[test]
    fn test_atan_frac_51043_7() {
        let s = "1.570659187521027203661619536335073835579283228441242208112611672132902725";
        let a = f256::from_str(s).unwrap();
        let n = f256::from(51043);
        let d = f256::from(7);
        let f = n / d;
        assert_eq!(n.atan2(&d), a);
    }
}
