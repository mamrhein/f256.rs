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

use super::FP255;
use crate::{
    abs_bits, abs_bits_sticky,
    big_uint::u256,
    consts::{FRAC_3_PI_2, FRAC_PI_2, FRAC_PI_4, PI, TAU},
    exp_bits, f256, fast_mul, fast_sum, sign_bits_hi, signif,
    BinEncAnySpecial, EXP_BIAS, EXP_BITS, FRACTION_BITS, HI_EXP_MASK,
    HI_FRACTION_BITS,
};

// Number of bits to shift left for adjusting the radix point from f256 to
// FP255
const PREC_ADJ: u32 = FP255::FRACTION_BITS - FRACTION_BITS;

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

// Accurate range reduction algorithm, adapted from
// S. Boldo, M. Daumas, R.-C. Li,
// Formally verified argument reduction with a fused multiply-add
// IEEE Trans. Comput. 58(8), 1139–1145 (2009)
// For the input value f, calculate ⌊f/½π⌋ and f%½π
fn fast_reduce(f: &f256) -> (f256, f256) {
    // 1/½π
    const R: f256 = f256 {
        bits: u256 {
            hi: 85070031364429158754372282577210457766,
            lo: 206986278175927573935717840202627382232,
        },
    };
    // 3 × 2ᴾ⁻²
    const D: f256 = f256 {
        bits: u256 {
            hi: 85147015849621175360001085100787761152,
            lo: 0,
        },
    };
    const C1: f256 = f256 {
        bits: u256 {
            hi: 85070452445679362461652637654843486235,
            lo: 174929234171320688473765933587459524448,
        },
    };
    const C2: f256 = f256 {
        bits: u256 {
            hi: 84994005351571227158322827037086056448,
            lo: 0,
        },
    };
    let mut k = f.mul_add(R, D) - D;
    debug_assert!(k.fract().eq_zero());
    let mut u = f - k * C1;
    if u.is_sign_negative() {
        k -= f256::ONE;
        u += C1;
    }
    debug_assert!(u.is_sign_positive());
    let vh = u - k * C2;
    debug_assert!(vh.is_sign_positive());
    let (ph, pl) = fast_mul(&k, &C2);
    let (th, tl) = fast_sum(&u, &ph.neg());
    let vl = ((th - vh) + tl) - pl;
    debug_assert!(vh.eq_zero() || vh.abs() > vl.abs(), "vh: {vh}\nvl: {vl}");
    (k, vh + vl)
}

// Max input value for fast_reduce
const M: f256 = f256 {
    bits: u256 {
        hi: 85147038824342751169173462475699482651,
        lo: 174929234171320688473765933587459524448,
    },
};

fn rem_frac_pi_2(x: &f256) -> (u32, f256) {
    debug_assert!(x.is_finite());
    debug_assert!(x.is_sign_positive());
    if x < &FRAC_PI_2 {
        (0, *x)
    } else if x < &PI {
        (1, x - &FRAC_PI_2)
    } else if x < &FRAC_3_PI_2 {
        (2, x - &PI)
    } else if x < &TAU {
        (3, x - &FRAC_3_PI_2)
    } else if x <= &M {
        let (q, r) = fast_reduce(x);
        // x <= M => q < 2ᴾ⁻²
        let m = signif(&q.bits);
        let e =
            exp_bits(&q.bits) as i32 - EXP_BIAS as i32 - FRACTION_BITS as i32;
        ((&m >> e.unsigned_abs()).lo as u32 & 0x3, r)
    } else {
        // TAU <= x <= f256::MAX
        const D: u256 = signif(&TAU.bits);
        // x >= TAU => exp(x) >= 2 => following expression can't be < 0
        let sh = exp_bits(&x.bits) - EXP_BIAS - 2;
        let mut t = signif(&x.bits);
        t = t.lshift_rem(&D, sh);
        if t.is_zero() {
            return (0, f256::ZERO);
        }
        let shl = t.leading_zeros() - EXP_BITS;
        t <<= shl;
        let u = f256::from_sign_exp_signif(
            0,
            -((FRACTION_BITS + shl - 2) as i32),
            (t.hi, t.lo),
        );
        debug_assert!(u < TAU);
        debug_assert!(u <= *x);
        rem_frac_pi_2(&u)
    }
}

#[inline(always)]
fn sin_cos(f: &f256) -> (f256, f256) {
    let x = FP255::from(f);
    let (fp_sin_x, fp_cos_x) = x.sin_cos();
    (f256::from(&fp_sin_x), f256::from(&fp_cos_x))
}

impl f256 {
    /// Simultaneously computes the sine and cosine of the number x.
    ///
    /// Returns (sin(x), cos(x)).
    pub fn sin_cos(&self) -> (Self, Self) {
        let x = self.abs();
        // If x is NAN or infinite, both, sine x and cosine x, are NAN.
        if x.bits.hi > f256::MAX.bits.hi {
            return (f256::NAN, f256::NAN);
        }
        // Calculate ⌊x/½π⌋ % 4 and x % ½π.
        let (mut quadrant, mut x) = rem_frac_pi_2(&x);
        debug_assert!(x.is_sign_positive());
        // If x is zero or very small, sine x == x and cosine x == 1.
        // TODO: verify limit
        let (sin, cos) = if x.eq_zero() || x.bits < SMALL_CUT_OFF {
            (x, f256::ONE)
        } else {
            sin_cos(&x)
        };
        // Map result according to quadrant
        let (mut sin, cos) = match quadrant {
            0 => (sin, cos),
            1 => (cos, -sin),
            2 => (-sin, -cos),
            3 => (-cos, sin),
            _ => unreachable!(),
        };
        // sin(-x) = -sin(x)
        sin.bits.hi ^= sign_bits_hi(&self);
        (sin, cos)
    }

    /// Computes the sine of a number (in radians).
    #[inline(always)]
    pub fn sin(&self) -> Self {
        self.sin_cos().0
    }

    /// Computes the cosine of a number (in radians).
    #[inline(always)]
    pub fn cos(&self) -> Self {
        self.sin_cos().1
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
        f256::from(&FP255::from(self).atan())
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

        let y = FP255::from(self);
        let x = FP255::from(other);
        f256::from(&y.atan2(&x))
    }
}

#[cfg(test)]
mod sin_cos_tests {
    use super::*;
    use crate::{
        consts::{FRAC_PI_3, FRAC_PI_4, FRAC_PI_6},
        ONE_HALF,
    };

    #[test]
    fn test_frac_pi_2_multiples() {
        const EXPECTED: [(f256, f256); 4] = [
            (f256::ZERO, f256::ONE),
            (f256::ONE, f256::ZERO),
            (f256::ZERO, f256::NEG_ONE),
            (f256::NEG_ONE, f256::ZERO),
        ];
        for i in 0_u32..=4_u32 {
            let f = f256::from(i) * FRAC_PI_2;
            let (sin, cos) = f.sin_cos();
            assert_eq!((sin, cos), EXPECTED[(i % 4) as usize]);
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
    fn test_multiples_of_tau_plus_delta() {
        let eps = f256::from_sign_exp_signif(
            0,
            -470,
            (
                0x00001000000000000000000000000000,
                0x00000000000000000000000000000000,
            ),
        );
        let d = f256::from(0.082735);
        let (sin_d, cos_d) = d.sin_cos();
        for i in [17_u128, 53904_u128, u128::MAX] {
            let f = f256::from(i) * TAU + d;
            let (sin_f, cos_f) = f.sin_cos();
            let (k, r) = fast_reduce(&f);
            assert!(
                (sin_d - sin_f).abs() <= f * eps,
                "{:?} !<= {:?}",
                (sin_d - sin_f).abs(),
                f * eps
            );
            assert!(
                (cos_d - cos_f).abs() <= f * eps,
                "{:?} !<= {:?}",
                (cos_d - cos_f).abs(),
                f * eps
            );
        }
    }

    #[test]
    fn test_frac_pi_4() {
        // sin(45°) = cos(45°)
        let f = FRAC_PI_4;
        let (sin, cos) = f.sin_cos();
        let d = cos - sin;
        assert!(d < f256::EPSILON);
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
        let d = sin - cos;
        assert!(d < f256::EPSILON);
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
}

#[cfg(test)]
mod atan_tests {
    use core::{ops::Neg, str::FromStr};

    use super::*;
    use crate::{
        consts::{FRAC_1_PI, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6},
        EPSILON,
    };

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
#[cfg(test)]
mod calc_reduce_consts {
    use super::*;
    use crate::{consts::FRAC_1_PI, SIGNIFICAND_BITS};

    fn print_as_const(s: &str, f: f256) {
        println!(
            "const {}: f256 = f256 {{\nbits: u256 {{\nhi: {},\nlo: \
             {},\n}},\n}};",
            s, f.bits.hi, f.bits.lo
        )
    }

    fn calc_reduce_consts(r: f256) {
        let c = f256::ONE / r;
        let mut m = c.significand();
        m.idiv_pow2(2);
        m <<= 2;
        let c1 = f256::from_sign_exp_signif(0, c.exponent(), (m.hi, m.lo));
        let d = c - c1;
        let ulp_c1 = c1.ulp();
        let f = (f256::from(8) * f256::EPSILON) * ulp_c1;
        let c2 = (d / f) * f;
        assert_eq!(c, c1 + c2);
        let f = f256::from(f64::from(SIGNIFICAND_BITS - 2).exp2());
        // 3 × 2ᴾ⁻²
        let d = f256::from(3) * f;
        // Upper limit
        let m = f.mul_add(PI, -FRAC_PI_2);
        println!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            r,
            c,
            c1,
            d,
            ulp_c1,
            f,
            c2,
            c1 + c2,
            m,
        );
        println!(
            "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n",
            r,
            c,
            c1,
            d,
            ulp_c1,
            f,
            c2,
            c1 + c2,
            m,
        );
        print_as_const("R", r);
        print_as_const("D", d);
        print_as_const("C1", c1);
        print_as_const("C2", c2);
        print_as_const("M", m);
    }

    #[test]
    fn calc_rem_frac_pi_2_consts() {
        // f256::ONE / FRAC_PI_2;
        let r = FRAC_1_PI * f256::TWO;
        calc_reduce_consts(r);
    }
}
