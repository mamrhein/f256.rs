// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::big_uint::{UInt, U128};
use crate::math::fp492::FP492;
use crate::{
    abs_bits, exp, exp_bits, f256, left_adj_signif, norm_bit, signif,
    BigUInt, DivRem, HiLo, EMAX, EMIN, EXP_BIAS, FRACTION_BITS, HI_EXP_MASK,
    HI_FRACTION_BIAS, HI_FRACTION_BITS, HI_SIGN_SHIFT, SIGNIFICAND_BITS,
    U256, U512,
};
use core::{
    cmp::{max, min, Ordering},
    fmt::Debug,
    mem::{size_of, swap},
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, Shl, Shr,
        Sub, SubAssign,
    },
};

fn add_signifs<T>(x: &T, y: &T) -> (T, i32)
where
    T: BigUInt + HiLo,
{
    debug_assert!(x.leading_zeros() == 1 || y.leading_zeros() == 1);
    let mut sum = *x;
    sum += y;
    let mut exp_adj = 0;
    if sum.leading_zeros() == 0 {
        // sum.idiv_pow2(1);
        sum >>= 1;
        exp_adj = 1;
    }
    (sum, exp_adj)
}

fn sub_signifs<T>(x: &T, y: &T) -> (T, i32)
where
    T: BigUInt + HiLo,
{
    debug_assert!(x >= y);
    debug_assert!(x.leading_zeros() == 1);
    let mut diff = *x;
    diff -= y;
    let shl = diff.leading_zeros() - 1;
    diff <<= shl;
    (diff, -(shl as i32))
}

fn mul_signifs<T: BigUInt + HiLo>(x: &T, y: &T) -> (UInt<T>, i32)
where
    T: BigUInt + HiLo,
{
    debug_assert!(x.leading_zeros() == 1);
    debug_assert!(y.leading_zeros() == 1);
    let (lo, hi) = x.widening_mul(y);
    let nlz = hi.leading_zeros();
    let res = &UInt::<T>::from_hi_lo(hi, lo) << (nlz - 1);
    (res, (nlz == 2) as i32)
}

fn div_signifs<T: BigUInt + HiLo>(x: &T, y: &T, sh: u32) -> (T, i32)
where
    T: BigUInt + HiLo,
{
    debug_assert!(x.leading_zeros() == 1);
    debug_assert!(y.leading_zeros() == 1);
    // 2ⁿ <= x < 2ⁿ⁺¹ and 2ⁿ <= y < 2ⁿ⁺¹
    // => ½ < x/y < 2
    // => 2ⁿ⁻¹ < (x/y⋅2ⁿ) < 2ⁿ⁺¹
    let exp_adj = (x < y) as i32;
    let x = UInt::<T>::from(x).shl(sh);
    let y = UInt::<T>::from(y);
    let q = x.rounding_div(&y);
    debug_assert!(q.hi().is_zero());
    debug_assert!(q.lo().leading_zeros() >= 2);
    let mut quot = q.lo_t();
    let nlz = quot.leading_zeros();
    quot <<= nlz - 1;
    (quot, exp_adj)
}

/// Representation of the number s⋅m⋅2⁻ⁿ⋅2ᵉ with
/// signum s, signif m, exp e and n fractional bits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Float<T> {
    signum: i32,
    exp: i32,
    // Layout of the bits of the `signif` member: olfff…fff
    // o = reserved bit for overflow handling in addition
    // l = 1 leading bit (always 1 except for BigFloat::ZERO)
    // f = n fractional bits
    signif: T,
}

impl<T> Float<T>
where
    T: BigUInt + HiLo + for<'a> From<&'a [u128]>,
{
    const SIZE_OF_SIGNIF: usize = T::BITS as usize / 8;
    pub(crate) const FRACTION_BITS: u32 = T::BITS - 2;

    const SIGNIF_ONE: T = T::TIE2;

    pub(crate) const ZERO: Self = Self {
        signum: 0,
        signif: T::ZERO,
        exp: 0,
    };
    pub(crate) const ONE: Self = Self {
        signum: 1,
        signif: Self::SIGNIF_ONE,
        exp: 0,
    };
    pub(crate) const NEG_ONE: Self = Self {
        signum: -1,
        signif: Self::SIGNIF_ONE,
        exp: 0,
    };
    pub(crate) const TWO: Self = Self {
        signum: 1,
        signif: Self::SIGNIF_ONE,
        exp: 1,
    };
    pub(crate) const ONE_HALF: Self = Self {
        signum: 1,
        signif: Self::SIGNIF_ONE,
        exp: -1,
    };
    // 2^-254
    pub(crate) const EPSILON: Self = Self {
        signum: 1,
        signif: Self::SIGNIF_ONE,
        exp: -(Self::FRACTION_BITS as i32),
    };

    /// Construct a `Float` value f from sign s, quantum exponent t and
    /// integral significand c, so that f = (-1)ˢ × 2ᵗ × c
    #[must_use]
    pub(crate) fn from_sign_exp_signif(s: u32, t: i32, c: &[u128]) -> Self {
        debug_assert!(s == 0 || s == 1);
        let mut m = T::from(c);
        if m.is_zero() {
            return Self::ZERO;
        }
        let (signif, exp) = match m.leading_zeros() {
            0 => (m >> 1, t + 1),
            1 => (m, t),
            lz @ 2.. => {
                let sh = lz - 1;
                (m << sh, t - sh as i32)
            }
        };
        debug_assert!(signif.leading_zeros() == 1);
        Self {
            signum: [1, -1][s as usize],
            exp: exp + Self::FRACTION_BITS as i32,
            signif,
        }
    }

    #[inline(always)]
    pub(crate) const fn signum(&self) -> i32 {
        self.signum
    }

    #[inline(always)]
    pub(crate) const fn exp(&self) -> i32 {
        self.exp
    }

    #[inline(always)]
    pub(crate) const fn signif(&self) -> T {
        self.signif
    }

    #[inline(always)]
    pub(crate) const fn quantum(&self) -> Self {
        Self {
            signum: 1,
            exp: self.exp - Self::FRACTION_BITS as i32,
            signif: Self::SIGNIF_ONE,
        }
    }

    #[inline(always)]
    pub(crate) const fn is_zero(&self) -> bool {
        self.signum == 0
    }

    #[inline(always)]
    pub(crate) fn flip_sign(&mut self) {
        self.signum *= -1;
    }

    #[inline(always)]
    pub(crate) fn copy_sign(&mut self, other: &Self) {
        self.signum = other.signum;
    }

    #[inline(always)]
    pub(crate) const fn abs(&self) -> Self {
        Self {
            signum: self.signum.abs(),
            exp: self.exp,
            signif: self.signif,
        }
    }

    pub(crate) fn trunc(&self) -> Self {
        match self.exp {
            // `self`>= 2 => set fractional bits to 0
            1.. => {
                let sh =
                    Self::FRACTION_BITS.saturating_sub(self.exp() as u32);
                Self {
                    signum: self.signum,
                    exp: self.exp,
                    signif: (self.signif >> sh) << sh,
                }
            }
            // 1 <= `self` < 2
            0 if !self.is_zero() => Self {
                signum: self.signum,
                exp: 0,
                signif: Self::SIGNIF_ONE,
            },
            // `self` < 1
            _ => Self::ZERO,
        }
    }

    fn iadd(&mut self, other: &Self) {
        let exp = max(self.exp, other.exp);
        if self.is_zero() || (exp - self.exp) > Self::FRACTION_BITS as i32 {
            *self = *other;
            return;
        }
        if other.is_zero() || (exp - other.exp) > Self::FRACTION_BITS as i32 {
            return;
        }
        let (mut signif_self, rem_self) = match (exp - self.exp) as u32 {
            0 => (self.signif, T::ZERO),
            sh @ _ => self.signif.widening_shr(sh),
        };
        let (mut signif_other, rem_other) = match (exp - other.exp) as u32 {
            0 => (other.signif, T::ZERO),
            sh @ _ => other.signif.widening_shr(sh),
        };
        let op = [add_signifs, sub_signifs]
            [(self.signum != other.signum) as usize];
        if signif_self < signif_other {
            swap(&mut signif_self, &mut signif_other);
            self.signum = other.signum;
        }
        let mut exp_adj = 0_i32;
        (self.signif, exp_adj) = op(&signif_self, &signif_other);
        if self.signif.is_zero() {
            self.signum = 0;
            self.exp = 0;
        } else {
            self.exp = (exp + exp_adj)
        };
    }

    fn isub(&mut self, other: &Self) {
        if self == other {
            *self = Self::ZERO;
        } else {
            self.iadd(&other.neg());
        }
    }

    fn imul(&mut self, other: &Self) {
        self.signum *= other.signum;
        if self.signum == 0 {
            *self = Self::ZERO;
        } else {
            let (mut prod_signif, exp_adj) =
                mul_signifs(&self.signif, &other.signif);
            // round significand of product to 255 bits
            // TODO: do rounding in fn mul_signifs
            let rnd = prod_signif.lo > T::TIE
                || (prod_signif.lo == T::TIE && prod_signif.hi.is_odd());
            prod_signif.hi.incr_if(rnd);
            prod_signif.hi >>= (prod_signif.hi.leading_zeros() == 0) as u32;
            self.signif = prod_signif.hi;
            self.exp += other.exp + exp_adj;
        }
    }

    fn idiv(&mut self, other: &Self) {
        assert!(!other.is_zero(), "Division by zero.");
        if self.signum == 0 {
            return;
        }
        self.signum *= other.signum;
        let (mut quot_signif, exp_adj) =
            div_signifs(&self.signif, &other.signif, Self::FRACTION_BITS);
        self.signif = quot_signif;
        self.exp -= other.exp + exp_adj;
    }

    pub(crate) fn recip(&self) -> Self {
        let mut recip = Self::ONE;
        recip.idiv(self);
        recip
    }

    fn imul_add(&mut self, f: &Self, a: &Self) {
        if self.is_zero() || f.is_zero() {
            *self = *a;
            return;
        }
        if a.is_zero() {
            self.imul(f);
            return;
        }
        let mut prod_exp = self.exp + f.exp;
        let mut exp_diff = prod_exp - a.exp;
        if exp_diff <= -(Self::FRACTION_BITS as i32) - 2 {
            // Product is too small
            *self = *a;
        } else if exp_diff < (2 * Self::FRACTION_BITS as i32 + 3) {
            let prod_signum = self.signum * f.signum;
            let (mut prod_signif, exp_adj) =
                mul_signifs(&self.signif, &f.signif);
            prod_exp += exp_adj;
            exp_diff += exp_adj;
            let mut addend_signif = UInt::<T> {
                hi: a.signif,
                lo: T::ZERO,
            };
            let (x_sign, mut x_signif, mut x_exp, y_signif) = match exp_diff {
                0 => {
                    // exponents equal => check significands
                    if prod_signif >= addend_signif {
                        // |prod| >= |addend|
                        (prod_signum, prod_signif, prod_exp, addend_signif)
                    } else {
                        // |addend| > |prod|
                        (a.signum, addend_signif, a.exp, prod_signif)
                    }
                }
                1..=i32::MAX => {
                    // |prod| > |addend|
                    let (q, r) = addend_signif.widening_shr(exp_diff as u32);
                    addend_signif = q;
                    addend_signif |= !r.is_zero();
                    (prod_signum, prod_signif, prod_exp, addend_signif)
                }
                i32::MIN..=-1 => {
                    // |addend| > |prod|
                    let (q, r) =
                        prod_signif.widening_shr(exp_diff.unsigned_abs());
                    prod_signif = q;
                    prod_signif |= !r.is_zero();
                    (a.signum, addend_signif, a.exp, prod_signif)
                }
            };
            self.signum = x_sign;
            self.exp = x_exp;
            if prod_signum == a.signum {
                x_signif += &y_signif;
                // addition may have overflowed
                let mut ovl = (x_signif.leading_zeros() == 0) as u32;
                self.exp += ovl as i32;
                self.signif =
                    x_signif.rounding_div_pow2(T::BITS + ovl).lo_t();
                // rounding may have overflowed
                ovl = (self.signif.leading_zeros() == 0) as u32;
                self.exp += ovl as i32;
                self.signif >>= ovl;
            } else {
                x_signif -= &y_signif;
                // subtraction may have cancelled some or all leading bits
                let shl = x_signif.leading_zeros() - 1;
                self.exp -= shl as i32;
                if shl <= T::BITS {
                    // shifting left by shl bits and then rounding the
                    // low bits => shift right and round by
                    // T::BITS - shl bits
                    x_signif = x_signif.rounding_div_pow2(T::BITS - shl);
                    self.signif = x_signif.lo;
                } else if shl == UInt::<T>::BITS {
                    // all bits cancelled => result is zero
                    *self = Self::ZERO;
                } else {
                    // less than T::BITS - 1 bits left
                    // => shift left, no rounding
                    self.signif = x_signif.lo_t() << shl - T::BITS;
                }
            };
        } else {
            // Addend is too small
            self.imul(f);
        }
    }

    #[inline]
    pub(crate) fn mul_add(self, f: &Self, a: &Self) -> Self {
        let mut res = self;
        res.imul_add(f, a);
        res
    }

    /// Computes the rounded sum of two BigFloat values and the remainder.
    #[inline]
    pub(crate) fn sum_exact(&self, rhs: &Self) -> (Self, Self) {
        let s = self + rhs;
        let d = s - *rhs;
        let t1 = *self - d;
        let t2 = *rhs - (s - d);
        let r = t1 + t2;
        (s, r)
    }

    /// Computes the rounded product of two BigFloat values and the remainder.
    #[inline]
    pub(crate) fn mul_exact(&self, rhs: &Self) -> (Self, Self) {
        let p = *self * *rhs;
        let r = self.mul_add(rhs, &p.neg());
        (p, r)
    }

    /// Computes `self` * 2ⁿ
    #[inline(always)]
    pub fn mul_pow2(&self, n: i32) -> Self {
        Self {
            signum: self.signum,
            exp: self.exp + n,
            signif: self.signif,
        }
    }

    /// Computes `self * self` .
    #[inline(always)]
    #[must_use]
    pub fn square(self) -> Self {
        self * self
    }

    /// Returns the square root of `self`.
    #[must_use]
    pub fn sqrt(&self) -> Self {
        debug_assert!(self.signum >= 0);
        if self.signum == 0 {
            return *self;
        }
        // Calculate the exponent
        let mut exp = self.exp;
        let exp_is_odd = exp & 1;
        exp = (exp - exp_is_odd) / 2;
        // Calculate the significand, gain extra bit for final rounding
        let mut q = Self::SIGNIF_ONE << 1;
        let mut r =
            (UInt::<T>::from(&self.signif) << (1 + exp_is_odd as u32)) - q;
        let mut s = q;
        for _ in 0..=Self::FRACTION_BITS {
            if r.is_zero() {
                break;
            }
            s >>= 1;
            let t = &r << 1;
            let u = (UInt::<T>::from(&q) << 1) + s;
            if t < u {
                r = t;
            } else {
                q += &s;
                r = t - u;
            }
        }
        // Final rounding
        let rnd_bits = (q.last_chunk() & 3_u128) as u32;
        q.incr_if(rnd_bits == 3 || rnd_bits == 1 && !r.is_zero());
        q >>= 1;
        Self {
            signum: 1,
            exp,
            signif: q,
        }
    }
}

impl<T: BigUInt> From<&T> for Float<T> {
    /// Convert a raw BigUInt into a Float, without any modification, i.e
    /// interpret the given value ui as ui⋅2⁻ⁿ with n = number of fractional
    /// digits of Self
    #[inline(always)]
    fn from(ui: &T) -> Self {
        Self {
            signum: 1,
            exp: 0,
            signif: *ui,
        }
    }
}

impl<T> From<i32> for Float<T>
where
    T: BigUInt + HiLo + From<u128>,
{
    #[inline(always)]
    fn from(i: i32) -> Self {
        Self::from_sign_exp_signif(
            (i < 0) as u32,
            0,
            &*T::from(i.unsigned_abs() as u128).as_vec_u128(),
        )
    }
}

impl<T> From<&f256> for Float<T>
where
    T: BigUInt + HiLo + for<'a> From<&'a [u128]>,
{
    fn from(f: &f256) -> Self {
        if f.eq_zero() {
            return Self::ZERO;
        }
        let abs_bits_f = abs_bits(f);
        debug_assert!(abs_bits_f.hi.0 < HI_EXP_MASK); // f is finite?
        debug_assert!(!abs_bits_f.is_zero());
        let mut signif_f = signif(&abs_bits_f);
        let shl = signif_f.leading_zeros() - 1;
        signif_f <<= shl;
        let exp_adj: u32 =
            shl - ((Self::FRACTION_BITS % 256) - FRACTION_BITS);
        let exp_f = exp(&abs_bits_f) - exp_adj as i32;
        let mut t = T::default().as_vec_u128();
        t[0] = signif_f.hi.0;
        t[1] = signif_f.lo.0;
        Self {
            signum: (-1_i32).pow(f.sign()),
            exp: exp_f,
            signif: T::from(&t),
        }
    }
}

#[inline(always)]
fn extract_signif<T: BigUInt + HiLo>(f256_signif: T) -> U256 {
    let [hi, lo] = match T::N_CHUNKS {
        2.. => *f256_signif.as_vec_u128().last_chunk::<2>().unwrap(),
        _ => [f256_signif.as_vec_u128()[0], 0_u128],
    };
    U256::new(hi, lo)
}

impl<T> From<&Float<T>> for f256
where
    T: BigUInt + HiLo,
{
    fn from(fp: &Float<T>) -> Self {
        if fp.is_zero() {
            return Self::ZERO;
        }
        const EXP_UNDERFLOW: i32 = EMIN - SIGNIFICAND_BITS as i32 - 1; // -262380
        const EXP_LOWER_SUBNORMAL: i32 = EXP_UNDERFLOW + 1; // -262379
        const EXP_UPPER_SUBNORMAL: i32 = EMIN - 1; // -262143
        const EXP_OVERFLOW: i32 = f256::MAX_EXP; //  262144
        let prec_adj: u32 =
            Float::<T>::FRACTION_BITS.saturating_sub(FRACTION_BITS);
        let mut f256_bits = match fp.exp {
            ..=EXP_UNDERFLOW => U256::ZERO,
            EXP_LOWER_SUBNORMAL..=EXP_UPPER_SUBNORMAL => {
                let f256_signif = fp.signif.rounding_div_pow2(
                    prec_adj.saturating_add_signed(EMIN - fp.exp),
                );
                extract_signif(f256_signif)
            }
            EMIN..=EMAX => {
                let (f256_signif, rem) = fp.signif.widening_shr(prec_adj);
                // -1 because we add the significand incl. hidden bit.
                let exp_bits = (EXP_BIAS.saturating_add_signed(fp.exp - 1)
                    as u128)
                    << HI_FRACTION_BITS;
                let mut bits = extract_signif(f256_signif);
                bits.hi.0 += exp_bits;
                // Final rounding. Possibly overflowing into the exponent,
                // but that is ok.
                if rem > T::TIE || (rem == T::TIE && bits.is_odd()) {
                    bits.incr();
                }
                bits
            }
            EXP_OVERFLOW.. => f256::INFINITY.bits,
        };
        f256_bits.hi.0 |= ((fp.signum < 0) as u128) << HI_SIGN_SHIFT;
        f256 { bits: f256_bits }
    }
}

#[cfg(test)]
mod from_i32_tests {
    use super::*;
    use crate::big_uint::U1024;

    #[test]
    fn from_i32() {
        assert_eq!(Float256::from(1_i32), Float256::ONE);
        assert_eq!(Float256::from(-1_i32), Float256::NEG_ONE);
        assert_eq!(Float512::from(0_i32), Float512::ZERO);
        assert_eq!(Float512::from(-2_i32), -Float512::TWO);
        let i = -262142_i32;
        assert_eq!(
            Float512::from(i),
            Float512::new(
                -1,
                17,
                &[(i.unsigned_abs() as u128) << 109, 0_u128, 0_u128, 0_u128,]
            )
        );
        assert_eq!(
            Float::<U1024>::from(i32::MIN),
            Float::<U1024> {
                signum: -1,
                exp: 31,
                signif: Float::<U1024>::SIGNIF_ONE,
            }
        );
    }
}

#[cfg(test)]
mod from_into_f256_tests {
    use super::*;
    use crate::big_uint::U1024;
    use crate::norm_signif_exp;

    fn assert_eq<T: BigUInt + HiLo>(f: &f256, g: &Float<T>) {
        debug_assert!(!f.is_special());
        assert_eq!((-1_i32).pow(f.sign()), g.signum);
        let f_abs_bits = abs_bits(f);
        let (f_signif, f_exp) = norm_signif_exp(&f_abs_bits);
        assert_eq!(f_exp, g.exp);
        let shl: u32 = (Float::<T>::FRACTION_BITS % 256) - FRACTION_BITS;
        let f_adj_signif = f_signif << shl;
        let g_signif = g.signif.as_vec_u128();
        let (g_signif_hi, g_signif_lo) = g_signif.split_at(2);
        assert_eq!(f_adj_signif, U256::from(g_signif_hi));
        assert!(g_signif_lo.iter().all(|x| *x == 0_u128));
    }

    fn test_neg_one_<T: BigUInt + HiLo>() {
        let fp = Float::<T>::from(&f256::NEG_ONE);
        assert_eq!(fp, Float::<T>::NEG_ONE);
        let f = f256::from(&fp);
        assert_eq(&f, &fp);
    }

    #[test]
    fn test_neg_one() {
        test_neg_one_::<U256>();
        test_neg_one_::<U512>();
        test_neg_one_::<U1024>();
    }

    fn test_normal_gt_one_<T: BigUInt + HiLo>() {
        let f = f256::from(1.5);
        let fp = Float::<T>::from(&f);
        assert_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_eq(&f, &fp);
    }

    #[test]
    fn test_normal_gt_one() {
        test_normal_gt_one_::<U256>();
        test_normal_gt_one_::<U512>();
        test_normal_gt_one_::<U1024>();
    }

    fn test_normal_lt_one_<T: BigUInt + HiLo>() {
        let f = f256::from(0.625);
        let fp = Float::<T>::from(&f);
        assert_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_eq(&f, &fp);
    }

    #[test]
    fn test_normal_lt_one() {
        test_normal_lt_one_::<U256>();
        test_normal_lt_one_::<U512>();
        test_normal_lt_one_::<U1024>();
    }

    fn test_normal_lt_minus_one_<T: BigUInt + HiLo>() {
        let f = f256::from(-7.5);
        let fp = Float::<T>::from(&f);
        assert_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_eq(&f, &fp);
    }

    #[test]
    fn test_normal_lt_minus_one() {
        test_normal_lt_minus_one_::<U256>();
        test_normal_lt_minus_one_::<U512>();
        test_normal_lt_minus_one_::<U1024>();
    }

    fn test_min_f256_<T: BigUInt + HiLo>() {
        let f = f256::MIN;
        let fp = Float::<T>::from(&f);
        assert_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_eq(&f, &fp);
    }

    #[test]
    fn test_min_f256() {
        test_min_f256_::<U256>();
        test_min_f256_::<U512>();
        test_min_f256_::<U1024>();
    }

    fn test_epsilon_<T: BigUInt + HiLo>() {
        let f = f256::EPSILON;
        let fp = Float::<T>::from(&f);
        assert_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_eq(&f, &fp);
    }

    #[test]
    fn test_epsilon() {
        test_epsilon_::<U256>();
        test_epsilon_::<U512>();
        test_epsilon_::<U1024>();
    }

    fn test_min_gt_zero_<T: BigUInt + HiLo>() {
        let f = f256::MIN_GT_ZERO;
        let fp = Float::<T>::from(&f);
        assert_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_eq(&f, &fp);
    }

    #[test]
    fn test_min_gt_zero() {
        test_min_gt_zero_::<U256>();
        test_min_gt_zero_::<U512>();
        test_min_gt_zero_::<U1024>();
    }
}

#[cfg(test)]
mod into_f256_tests {
    use super::*;
    use crate::consts::PI;

    fn test_overflow_1_<T: BigUInt + HiLo>() {
        let fp = Float::<T> {
            signum: -1,
            exp: f256::MAX_EXP,
            signif: Float::<T>::ONE.signif,
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::NEG_INFINITY);
    }

    #[test]
    fn test_overflow_1() {
        test_overflow_1_::<U256>();
        test_overflow_1_::<U512>();
    }

    fn test_overflow_2_<T: BigUInt + HiLo>() {
        let fp = Float::<T> {
            signum: 1,
            exp: EMAX,
            signif: (T::MAX >> 1) - T::from(&131071_u128),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::INFINITY);
    }

    #[test]
    fn test_overflow_2() {
        test_overflow_2_::<U256>();
        test_overflow_2_::<U512>();
    }

    fn test_overflow_3_<T: BigUInt + HiLo>() {
        let sh = Float::<T>::FRACTION_BITS - FRACTION_BITS;
        let fp = Float::<T> {
            signum: 1,
            exp: 0,
            signif: (T::MAX >> sh) << (sh - 1),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::TWO);
    }

    #[test]
    fn test_overflow_3() {
        test_overflow_3_::<U256>();
        test_overflow_3_::<U512>();
    }

    fn test_underflow_<T: BigUInt + HiLo>() {
        let fp = Float::<T> {
            signum: 1,
            exp: EMIN - SIGNIFICAND_BITS as i32,
            signif: Float::<T>::SIGNIF_ONE,
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::ZERO);
    }

    #[test]
    fn test_underflow() {
        test_underflow_::<U256>();
        test_underflow_::<U512>();
    }

    fn test_round_to_min_gt_zero_<T: BigUInt + HiLo>() {
        let fp = Float::<T> {
            signum: 1,
            exp: EMIN - SIGNIFICAND_BITS as i32,
            signif: Float::<T>::SIGNIF_ONE + T::ONE,
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::MIN_GT_ZERO);
    }

    #[test]
    fn test_round_to_min_gt_zero() {
        test_round_to_min_gt_zero_::<U256>();
        test_round_to_min_gt_zero_::<U512>();
    }

    fn test_round_to_epsilon_<T: BigUInt + HiLo>() {
        let fp = Float::<T> {
            signum: 1,
            exp: -237,
            signif: T::MAX >> 1,
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::EPSILON);
    }

    #[test]
    fn test_round_to_epsilon() {
        test_round_to_epsilon_::<U256>();
        test_round_to_epsilon_::<U512>();
    }

    fn test_f256_pi_<T: BigUInt + HiLo>() {
        let f = PI;
        let fp = Float::<T>::from(&f);
        let g = f256::from(&fp);
        assert_eq!(f, g);
    }

    #[test]
    fn test_f256_pi() {
        test_f256_pi_::<U256>();
        test_f256_pi_::<U512>();
    }
}

impl<T: BigUInt + HiLo> Neg for Float<T> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self::Output {
            signum: self.signum * -1,
            exp: self.exp,
            signif: self.signif,
        }
    }
}

impl<T: BigUInt + HiLo> AddAssign<Self> for Float<T> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.iadd(&rhs);
    }
}

impl<T: BigUInt + HiLo> AddAssign<&Self> for Float<T> {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Self) {
        self.iadd(rhs);
    }
}

impl<T: BigUInt + HiLo> Add for Float<T> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a, T: BigUInt + HiLo> Add for &'a Float<T> {
    type Output = <Float<T> as Add>::Output;

    fn add(self, rhs: Self) -> Self::Output {
        let mut res = *self;
        res += rhs;
        res
    }
}

impl<T: BigUInt + HiLo> SubAssign<Self> for Float<T> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.isub(&rhs);
    }
}

impl<T: BigUInt + HiLo> SubAssign<&Self> for Float<T> {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &Self) {
        self.isub(rhs);
    }
}

impl<T: BigUInt + HiLo> Sub for Float<T> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl<'a, T: BigUInt + HiLo> Sub for &'a Float<T> {
    type Output = <Float<T> as Sub>::Output;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = *self;
        res -= rhs;
        res
    }
}

impl<T: BigUInt + HiLo> MulAssign<Self> for Float<T> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        self.imul(&rhs);
    }
}

impl<T: BigUInt + HiLo> MulAssign<&Self> for Float<T> {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &Self) {
        self.imul(rhs);
    }
}

impl<T: BigUInt + HiLo> Mul for Float<T> {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self *= rhs;
        self
    }
}

impl<'a, T: BigUInt + HiLo> Mul for &'a Float<T> {
    type Output = <Float<T> as Mul>::Output;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut res = *self;
        res *= rhs;
        res
    }
}

impl<T: BigUInt + HiLo> DivAssign<Self> for Float<T> {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        self.idiv(&rhs);
    }
}

impl<T: BigUInt + HiLo> DivAssign<&Self> for Float<T> {
    #[inline(always)]
    fn div_assign(&mut self, rhs: &Self) {
        self.idiv(rhs);
    }
}

impl<T: BigUInt + HiLo> Div for Float<T> {
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self::Output {
        self /= rhs;
        self
    }
}

impl<'a, T: BigUInt + HiLo> Div for &'a Float<T> {
    type Output = <Float<T> as Div>::Output;

    fn div(self, rhs: Self) -> Self::Output {
        let mut res = *self;
        res /= rhs;
        res
    }
}

pub(crate) type Float256 = Float<U256>;

impl Float256 {
    // PI = ◯₂₅₄(π) =
    // 3.1415926535897932384626433832795028841971693993751058209749445923078164062862
    pub(crate) const PI: Float256 = Float256::new(
        1,
        1,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    );
    // FRAC_PI_2 = ◯₂₅₄(½π) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    pub(crate) const FRAC_PI_2: Float256 = Float256::new(
        1,
        0,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    );
    // FRAC_PI_4 = ◯₂₅₄(½π) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    pub(crate) const FRAC_PI_4: Float256 = Float256::new(
        1,
        -1,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    );
    // FRAC_3_PI_2 = ◯₂₅₄(3⋅½π) =
    // 4.7123889803846898576939650749192543262957540990626587314624168884617246094293
    pub(crate) const FRAC_3_PI_2: Float256 = Float256::new(
        1,
        2,
        (
            0x4b65f1fccc8748d3c9ca64f450528ace,
            0x6f60dd4333e6ecab80c4677e56275a2d,
        ),
    );
    // TAU = ◯₂₅₄(2⋅π) =
    // 6.2831853071795864769252867665590057683943387987502116419498891846156328125724
    pub(crate) const TAU: Float256 = Float256::new(
        1,
        2,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    );
    // FRAC_3_PI_4 = ◯₂₅₄(3⋅¼π) =
    // 2.3561944901923449288469825374596271631478770495313293657312084442308623047147
    pub(crate) const FRAC_3_PI_4: Float256 = Float256::new(
        1,
        1,
        (
            0x4b65f1fccc8748d3c9ca64f450528ace,
            0x6f60dd4333e6ecab80c4677e56275a2d,
        ),
    );
    // FRAC_5_PI_4 = ◯₂₅₄(5⋅¼π) =
    // 3.9269908169872415480783042290993786052464617492188822762186807403847705078577
    pub(crate) const FRAC_5_PI_4: Float256 = Float256::new(
        1,
        1,
        (
            0x7da9e8a554e17960fafbfd9730899202,
            0xb9a170c55680dfc881475727e4ec40f5,
        ),
    );
    // FRAC_7_PI_4 = ◯₂₅₄(7⋅¼π) =
    // 5.4977871437821381673096259207391300473450464489064351867061530365386787110009
    pub(crate) const FRAC_7_PI_4: Float256 = Float256::new(
        1,
        2,
        (
            0x57f6efa6ee9dd4f71616cb1d08604c9b,
            0x81f10223bc8d6972c0e52368b9d893df,
        ),
    );
    // FRAC_9_PI_4 = ◯₂₅₄(9⋅¼π) =
    // 7.0685834705770347865409476123788814894436311485939880971936253326925869141439
    pub(crate) const FRAC_9_PI_4: Float256 = Float256::new(
        1,
        2,
        (
            0x7118eafb32caed3daeaf976e787bd035,
            0xa7114be4cdda630141269b3d813b0743,
        ),
    );
    // SQRT_PI = ◯₂₅₄(√π) =
    // 1.77245385090551602729816748334114518279754945612238712821380778985291128459104
    pub(crate) const SQRT_PI: Float256 = Float256::new(
        1,
        0,
        (
            0x716fe246d3bdaa9e70ec1483576e4e0f,
            0xf8e48551bd8ec94b728402f4fa851d1c,
        ),
    );
    // SQRT_2 = ◯₂₅₄(√2) =
    // 1.41421356237309504880168872420969807856967187537694807317667973799073247846212
    pub(crate) const SQRT_2: Float256 = Float256::new(
        1,
        0,
        (
            0x5a827999fcef32422cbec4d9baa55f4f,
            0x8eb7b05d449dd426768bd642c199cc8b,
        ),
    );
    // FRAC_1_SQRT_2 = ◯₂₅₄(1/√2) =
    // 0.70710678118654752440084436210484903928483593768847403658833986899536623923106
    pub(crate) const FRAC_1_SQRT_2: Float256 = Float256::new(
        1,
        -1,
        (
            0x5a827999fcef32422cbec4d9baa55f4f,
            0x8eb7b05d449dd426768bd642c199cc8b,
        ),
    );

    /// Raw assembly from signum, exponent and significand.
    #[inline]
    pub(crate) const fn new(
        signum: i32,
        exp: i32,
        signif: (u128, u128),
    ) -> Self {
        debug_assert!(signif.0.leading_zeros() == 1);
        Self {
            signum,
            exp,
            signif: U256::new(signif.0, signif.1),
        }
    }
}

pub(crate) type Float512 = Float<U512>;

impl Float512 {
    pub(crate) const THREE_HALF: Self = Self {
        signum: 1,
        signif: U512::new(3_u128 << 125, 0, 0, 0),
        exp: 0,
    };
    // LN_2 = ◯₅₁₀(logₑ(2)) =
    // 6.9314718055994530941723212145817656807550013436025525412068000949339362196969471560586332699641868754200148102057068573368552023575813055703267075163507602e-1
    pub(crate) const LN_2: Float512 = Float512::new(
        1,
        -1,
        &[
            0x58b90bfbe8e7bcd5e4f1d9cc01f97b57,
            0xa079a193394c5b16c5068badc5d57d15,
            0xf3dc3b1036f5d64c2acaa97da57d0d88,
            0x7697571ae09c10a213ab9d9488b4dc13,
        ],
    );
    // LN_10 = ◯₅₁₀(logₑ(10)) =
    // 2.3025850929940456840179914546843642076011014886287729760333279009675726096773524802359972050895982983419677840422862486334095254650828067566662873690987815
    pub(crate) const LN_10: Float512 = Float512::new(
        1,
        1,
        &[
            0x49aec6eed554560b752b6b15c1698514,
            0x7147f67ced2efc8741e30f4100f816b9,
            0x4b17816bd8d4082e663865e0162f86b4,
            0x1631120c2085f16d7dc7bc4201728b6b,
        ],
    );
    // LOG2_E = ◯₅₁₀(log₂(E)) =
    // 1.4426950408889634073599246810018921374266459541529859341354494069311092191811850798855266228935063444969975183096525442555931016871683596427206621582234795
    pub(crate) const LOG2_E: Float512 = Float512::new(
        1,
        0,
        &[
            0x5c551d94ae0bf85ddf43ff68348e9f44,
            0x75abbd546eb4ad2c45928b3668d09923,
            0xef0e21fbaa8bb66b126c97bae0b5f059,
            0xf5485cf30625484fe25fd781a9ef9cda,
        ],
    );
    // LOG10_E = ◯₅₁₀(log₁₀(E)) =
    // 4.3429448190325182765112891891660508229439700580366656611445378316586464920887077472922494933843174831870610674476630373364167928715896390656922106466281229e-1
    pub(crate) const LOG10_E: Float512 = Float512::new(
        1,
        -2,
        &[
            0x6f2dec549b9438ca9aadd557d699ee19,
            0x1f71a30122e4d1011d1f96a27bc7529e,
            0x3aa1277d0a0179f94911aac96323250a,
            0x8c671decfe9c6e5e37d15c696466d3da,
        ],
    );

    /// Raw assembly from signum, exponent and significand.
    #[inline]
    pub(crate) const fn new(signum: i32, exp: i32, signif: &[u128]) -> Self {
        debug_assert!(signif[0].leading_zeros() == 1);
        debug_assert!(signif.len() == 4);
        Self {
            signum,
            exp,
            signif: U512::new(signif[0], signif[1], signif[2], signif[3]),
        }
    }
}

#[cfg(test)]
mod add_sub_tests {
    use super::*;

    fn test_add_same_sign_<T: BigUInt + HiLo>() {
        let mut f = Float::<T>::ONE;
        f += f;
        assert_eq!(f.signum, 1);
        assert_eq!(f.exp, 1);
        assert_eq!(f.signif, Float::<T>::ONE.signif);
        f.flip_sign();
        f += &f.clone();
        assert_eq!(f.signum, -1);
        assert_eq!(f.exp, 2);
        assert_eq!(f.signif, Float::<T>::ONE.signif);
    }

    #[test]
    fn test_add_same_sign() {
        test_add_same_sign_::<U256>();
        test_add_same_sign_::<U512>();
    }

    fn test_add_diff_sign_<T: BigUInt + HiLo + From<u128>>() {
        let mut f = Float::<T>::ONE;
        f += &Float::<T>::NEG_ONE;
        assert_eq!(f, Float::<T>::ZERO);
        let mut f = Float::<T> {
            signum: -1,
            exp: 0,
            signif: Float::<T>::ONE.signif + <T>::from(1_u128),
        };
        f += &Float::<T>::EPSILON;
        assert_eq!(f, Float::<T>::NEG_ONE);
        let mut g = Float::<T>::EPSILON;
        g += &f.clone();
        assert_eq!(g.signum, -1);
        assert_eq!(g.exp, -1);
        assert_eq!(g.signif, (T::MAX >> 1) - T::from(1_u128));
    }

    #[test]
    fn test_add_diff_sign() {
        test_add_diff_sign_::<U256>();
        test_add_diff_sign_::<U512>();
    }

    #[test]
    fn test_add_small_values() {
        let mut a = Float256 {
            signum: 1,
            exp: -31,
            signif: U256::new(
                0x762786250e76f22407ff39555489fe14,
                0xc7367a8a3b4fb9bb64012ff173ba3820,
            ),
        };
        let b = Float256 {
            signum: -1,
            exp: -29,
            signif: U256::new(
                0x4a861bd3d04784350b3364f35c90403f,
                0xbc67c2e66a540f53f03c854744f355a4,
            ),
        };
        let d = Float256 {
            signum: -1,
            exp: -30,
            signif: U256::new(
                0x59f8749519538f5812672d3c0edb8175,
                0x15344887b70041ca2e787295d0098f38,
            ),
        };
        a += &b;
        assert_eq!(a, d);
    }

    fn test_sub_diff_sign_<T: BigUInt + HiLo>() {
        let mut f = Float::<T>::NEG_ONE;
        f -= &Float::<T>::ONE;
        assert_eq!(f.signum, -1);
        assert_eq!(f.exp, 1);
        assert_eq!(f.signif, Float::<T>::ONE.signif);
        let mut g = f;
        g.flip_sign();
        f -= &g;
        assert_eq!(f.signum, -1);
        assert_eq!(f.exp, 2);
        assert_eq!(f.signif, Float::<T>::ONE.signif);
    }

    #[test]
    fn test_sub_diff_sign() {
        test_sub_diff_sign_::<U256>();
        test_sub_diff_sign_::<U512>();
    }

    fn test_sub_same_sign_<T: BigUInt + HiLo + From<u128>>() {
        let mut f = Float::<T>::NEG_ONE;
        f -= f;
        assert_eq!(f, Float::<T>::ZERO);
        let mut f = Float::<T> {
            signum: 1,
            exp: 0,
            signif: Float::<T>::ONE.signif + <T>::from(1_u128),
        };
        f -= &Float::<T>::EPSILON;
        assert_eq!(f, Float::<T>::ONE);
        let mut g = Float::<T>::EPSILON;
        g -= &f.clone();
        assert_eq!(g.signum, -1);
        assert_eq!(g.exp, -1);
        assert_eq!(g.signif, (T::MAX >> 1) - T::from(1_u128));
    }

    #[test]
    fn test_sub_same_sign() {
        test_sub_same_sign_::<U256>();
        test_sub_same_sign_::<U512>();
    }

    #[test]
    fn test_sub_small_value() {
        let mut a = Float256 {
            signum: 1,
            exp: -1,
            signif: U256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x033cee6a628198d4c2836363a132d844,
            ),
        };
        let b = Float256 {
            signum: -1,
            exp: -128,
            signif: U256::new(
                0x40000000000000000000000000000000,
                0x00000000000000000000000000000000,
            ),
        };
        let d = Float256 {
            signum: 1,
            exp: -1,
            signif: U256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x833cee6a628198d4c2836363a132d844,
            ),
        };
        a -= &b;
        assert_eq!(a, d);
    }

    #[test]
    fn test_sub_very_small_value() {
        let mut a = Float256 {
            signum: 1,
            exp: 7,
            signif: U256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x033cee6a628198d4c2836363a132d844,
            ),
        };
        let b = Float256 {
            signum: 1,
            exp: -248,
            signif: U256::new(
                0x40000000000000000000000000000000,
                0x00000000000000000000000000000000,
            ),
        };
        assert_eq!(a - b, a);
    }
}

#[cfg(test)]
mod mul_tests {
    use super::*;

    fn test_imul_same_sign_<T: BigUInt + HiLo + From<u128>>() {
        let mut x = Float::<T>::NEG_ONE;
        x += &Float::<T>::EPSILON;
        let y = x;
        x *= x;
        assert_eq!(x.signum, 1);
        assert_eq!(x.exp, -1);
        assert_eq!(x.signif, (T::MAX >> 1) - T::from(3_u128));
    }

    #[test]
    fn test_imul_same_sign() {
        test_imul_same_sign_::<U256>();
        test_imul_same_sign_::<U512>();
    }

    #[test]
    fn test_imul_diff_sign() {
        let mut x = Float256::FRAC_PI_2;
        let y = -Float256::PI;
        x.imul(&y);
        assert_eq!(x.signum, -1);
        assert_eq!(x.exp, 2);
        assert_eq!(
            x.signif,
            U256::new(
                0x4ef4f326f91779692b71366cc0460d63,
                0x842b351ff06851143341108129e39b47
            )
        );
    }

    #[test]
    fn test_imul_add() {
        let mut x = Float256::PI;
        let y = Float256 {
            signum: 1,
            exp: -1,
            signif: U256::new(
                0x7fffffffffffffffffffffffffffffff,
                0xffffffffffffffffffffffffffffc000,
            ),
        };
        let mut p = x;
        p.imul(&y);
        assert_eq!(
            p.signif,
            U256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x948127044533e63a0105df531d899b4d,
            )
        );
        let a = Float256 {
            signum: 1,
            exp: -255,
            signif: U256::new(
                0x7fffffffffffffffffffffffffffffff,
                0xfffffffffffffffffffff80000000000,
            ),
        };
        p.iadd(&a);
        assert_eq!(
            p.signif,
            U256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x948127044533e63a0105df531d899b4d,
            )
        );
        x.imul_add(&y, &a);
        assert_eq!(
            x,
            Float256 {
                signum: p.signum,
                exp: p.exp,
                signif: U256::new(
                    0x6487ed5110b4611a62633145c06e0e68,
                    0x948127044533e63a0105df531d899b4e,
                ),
            }
        );
    }
}

#[cfg(test)]
mod div_tests {
    use super::*;

    fn test_idiv_by_one_<T: BigUInt + HiLo>() {
        let mut x = Float::<T>::ONE;
        x.idiv(&Float::<T>::ONE);
        assert_eq!(x, Float::<T>::ONE);
        let mut x = -Float::<T>::ONE_HALF;
        x.idiv(&Float::<T>::NEG_ONE);
        assert_eq!(x, Float::<T>::ONE_HALF);
    }

    #[test]
    fn test_idiv_by_one() {
        test_idiv_by_one_::<U256>();
        test_idiv_by_one_::<U512>();
    }

    #[test]
    fn test_idiv_by_one_half() {
        let mut x = Float256::PI;
        x.idiv(&Float256::ONE_HALF);
        assert_eq!(x, Float256::TAU);
        let mut x = -Float256::FRAC_PI_4;
        x.idiv(&Float256::ONE_HALF);
        assert_eq!(x, -Float256::FRAC_PI_2);
    }

    #[test]
    fn test_idiv_by_pi() {
        let mut x = Float256::FRAC_3_PI_4;
        let three = Float256::from(&f256::from(3_f64));
        let four = Float256::from(&f256::from(4_f64));
        let mut q = three;
        q.idiv(&four);
        x.idiv(&Float256::PI);
        assert_eq!(x, q);
    }

    #[test]
    fn test_idiv_and_recip() {
        let x = Float256::from(&f256::from(5.99999_f64));
        let y = Float256::from(&f256::from(6_f64));
        let z = Float256::new(
            1,
            -1,
            (
                0x7ffff204dc4f6aaaaaaaaaaaaaaaaaaa,
                0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
            ),
        );
        let mut q = x;
        q.idiv(&y);
        assert_eq!(q, z);
        let mut q = y;
        q.idiv(&x);
        let d = (q - z.recip()).abs();
        assert!(d <= Float256::EPSILON);
    }
}

#[cfg(test)]
mod sqrt_tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(Float512::ZERO.sqrt(), Float512::ZERO);
    }

    #[test]
    fn test_one() {
        assert_eq!(Float512::ONE.sqrt(), Float512::ONE);
    }

    #[test]
    fn test_nine() {
        let nine = Float256::from(&f256::from(9));
        let three = Float256::from(&f256::from(3));
        assert_eq!(nine.sqrt(), three);
    }

    #[test]
    fn test_nine_quarter() {
        let nine = f256::from(9);
        let four = f256::from(4);
        let three = f256::from(3);
        let x = nine / four;
        let y = three / f256::TWO;
        assert_eq!(Float256::from(&x).sqrt(), Float256::from(&y));
    }

    #[test]
    fn test_one_half() {
        let r = Float256::ONE_HALF.sqrt();
        assert_eq!(r, Float256::FRAC_1_SQRT_2);
    }

    #[test]
    fn test_two() {
        let sqrt2 = Float256::TWO.sqrt();
        assert_eq!(sqrt2, Float256::SQRT_2);
    }

    #[test]
    fn test_pi() {
        let sqrt_pi = Float256::PI.sqrt();
        assert_eq!(sqrt_pi, Float256::SQRT_PI);
    }
}
