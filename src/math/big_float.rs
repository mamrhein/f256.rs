// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, Ordering},
    mem::swap,
    ops::{
        Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Shr, ShrAssign,
        Sub, SubAssign,
    },
};

use crate::{
    abs_bits, exp_bits, f256, norm_bit, signif, BigUInt, DivRem, HiLo, EMAX,
    EMIN, EXP_BIAS, FRACTION_BITS, HI_EXP_MASK, HI_FRACTION_BIAS,
    HI_FRACTION_BITS, HI_SIGN_SHIFT, SIGNIFICAND_BITS, U256, U512,
};

fn add_signifs(x: &U256, y: &U256) -> (U256, i32) {
    debug_assert!(x.hi.leading_zeros() == 1 || y.hi.leading_zeros() == 1);
    let mut sum = x + y;
    let mut exp_adj = 0;
    if sum.hi.leading_zeros() == 0 {
        // sum.idiv_pow2(1);
        sum >>= 1;
        exp_adj = 1;
    }
    (sum, exp_adj)
}

fn sub_signifs(x: &U256, y: &U256) -> (U256, i32) {
    debug_assert!(x >= y);
    debug_assert!(x.hi.leading_zeros() == 1);
    let mut diff = x - y;
    let shl = diff.leading_zeros() - 1;
    diff <<= shl;
    (diff, -(shl as i32))
}

fn mul_signifs(x: &U256, y: &U256) -> (U512, i32) {
    debug_assert!(x.hi.leading_zeros() == 1);
    debug_assert!(y.hi.leading_zeros() == 1);
    let (lo, hi) = x.widening_mul(y);
    let nlz = hi.leading_zeros();
    let res = &U512 { hi, lo } << (nlz - 1);
    // TODO: do rounding here
    (res, (nlz == 2) as i32)
}

fn div_signifs(x: &U256, y: &U256) -> (U256, i32) {
    debug_assert!(x.hi.leading_zeros() == 1);
    debug_assert!(y.hi.leading_zeros() == 1);
    // 2²⁵⁴ <= x < 2²⁵⁵ and 2²⁵⁴ <= y < 2²⁵⁵
    // => ½ < x/y < 2
    // => 2²⁵³ < (x/y⋅2²⁵⁴) < 2²⁵⁵
    const N: u32 = Float256::FRACTION_BITS - u128::BITS;
    let exp_adj = (x < y) as i32;
    let mut quot = U256::new(((exp_adj == 0) as u128) << N, 0);
    debug_assert!(quot.hi.leading_zeros() == 1 || quot.is_zero());
    let mut x_hat = &U512::from(&(x % y)) << Float256::FRACTION_BITS;
    let (q, r) = x_hat.div_rem_subuint_special(&y);
    debug_assert_eq!(q.hi, U256::ZERO);
    debug_assert!(q.lo.leading_zeros() >= 2);
    quot += &q.lo;
    let rnd = r > (y >> 1);
    quot.incr_if(rnd);
    let nlz = quot.hi.leading_zeros();
    quot <<= nlz - 1;
    (quot, exp_adj)
}

/// Representation of the number signum ⋅ signif ⋅ 2⁻²⁵⁴ ⋅ 2ᵉ.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct Float256 {
    signum: i32,
    exp: i32,
    // Layout of the 256 bits of the `signif` member: olfff…fff
    // o = reserved bit for overflow handling in addition
    // l = 1 leading bit (always 1 except for BigFloat::ZERO)
    // f = 254 fractional bits
    signif: U256,
}

const SIGNIF_ONE: U256 =
    U256::new(1_u128 << (Float256::FRACTION_BITS - 128), 0_u128);

const TIE: U256 = U256::new(1_u128 << 127, 0_u128);

impl Float256 {
    pub(crate) const FRACTION_BITS: u32 = 254;
    pub(crate) const ZERO: Self = Self {
        signum: 0,
        signif: U256::ZERO,
        exp: 0,
    };
    pub(crate) const ONE: Self = Self {
        signum: 1,
        signif: SIGNIF_ONE,
        exp: 0,
    };
    pub(crate) const NEG_ONE: Self = Self {
        signum: -1,
        signif: SIGNIF_ONE,
        exp: 0,
    };
    pub(crate) const TWO: Self = Self {
        signum: 1,
        signif: SIGNIF_ONE,
        exp: 1,
    };
    pub(crate) const ONE_HALF: Self = Self {
        signum: 1,
        signif: SIGNIF_ONE,
        exp: -1,
    };
    // 2^-254
    pub(crate) const EPSILON: Self = Self {
        signum: 1,
        signif: SIGNIF_ONE,
        exp: -(Self::FRACTION_BITS as i32),
    };
    // PI = ◯₂₅₅(π) =
    // 3.1415926535897932384626433832795028841971693993751058209749445923078164062862
    pub(crate) const PI: Float256 = Float256::new(
        1,
        1,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    );
    // FRAC_PI_2 = ◯₂₅₅(½π) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    pub(crate) const FRAC_PI_2: Float256 = Float256::new(
        1,
        0,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    );
    // FRAC_PI_4 = ◯₂₅₅(½π) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    pub(crate) const FRAC_PI_4: Float256 = Float256::new(
        1,
        -1,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    );
    // FRAC_3_PI_2 = ◯₂₅₅(3⋅½π) =
    // 4.7123889803846898576939650749192543262957540990626587314624168884617246094293
    pub(crate) const FRAC_3_PI_2: Float256 = Float256::new(
        1,
        2,
        (
            0x4b65f1fccc8748d3c9ca64f450528ace,
            0x6f60dd4333e6ecab80c4677e56275a2d,
        ),
    );
    // TAU = ◯₂₅₅(2⋅π) =
    // 6.2831853071795864769252867665590057683943387987502116419498891846156328125724
    pub(crate) const TAU: Float256 = Float256::new(
        1,
        2,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    );
    // FRAC_3_PI_4 = ◯₂₅₅(3⋅¼π) =
    // 2.3561944901923449288469825374596271631478770495313293657312084442308623047147
    pub(crate) const FRAC_3_PI_4: Float256 = Float256::new(
        1,
        1,
        (
            0x4b65f1fccc8748d3c9ca64f450528ace,
            0x6f60dd4333e6ecab80c4677e56275a2d,
        ),
    );
    // FRAC_5_PI_4 = ◯₂₅₅(5⋅¼π) =
    // 3.9269908169872415480783042290993786052464617492188822762186807403847705078577
    pub(crate) const FRAC_5_PI_4: Float256 = Float256::new(
        1,
        1,
        (
            0x7da9e8a554e17960fafbfd9730899202,
            0xb9a170c55680dfc881475727e4ec40f5,
        ),
    );
    // FRAC_7_PI_4 = ◯₂₅₅(7⋅¼π) =
    // 5.4977871437821381673096259207391300473450464489064351867061530365386787110009
    pub(crate) const FRAC_7_PI_4: Float256 = Float256::new(
        1,
        2,
        (
            0x57f6efa6ee9dd4f71616cb1d08604c9b,
            0x81f10223bc8d6972c0e52368b9d893df,
        ),
    );
    // FRAC_9_PI_4 = ◯₂₅₅(9⋅¼π) =
    // 7.0685834705770347865409476123788814894436311485939880971936253326925869141439
    pub(crate) const FRAC_9_PI_4: Float256 = Float256::new(
        1,
        2,
        (
            0x7118eafb32caed3daeaf976e787bd035,
            0xa7114be4cdda630141269b3d813b0743,
        ),
    );
    // SQRT_PI = ◯₂₅₅(√π) =
    // 1.77245385090551602729816748334114518279754945612238712821380778985291128459104
    pub(crate) const SQRT_PI: Float256 = Float256::new(
        1,
        0,
        (
            0x716fe246d3bdaa9e70ec1483576e4e0f,
            0xf8e48551bd8ec94b728402f4fa851d1c,
        ),
    );
    // SQRT_2 = ◯₂₅₅(√2) =
    // 1.41421356237309504880168872420969807856967187537694807317667973799073247846212
    pub(crate) const SQRT_2: Float256 = Float256::new(
        1,
        0,
        (
            0x5a827999fcef32422cbec4d9baa55f4f,
            0x8eb7b05d449dd426768bd642c199cc8b,
        ),
    );
    // FRAC_1_SQRT_2 = ◯₂₅₅(1/√2) =
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

    /// Construct a `BigFloat` value f from sign s, quantum exponent t and
    /// integral significand c, so that f = (-1)ˢ × 2ᵗ × c
    #[must_use]
    pub(crate) fn from_sign_exp_signif(
        s: u32,
        t: i32,
        c: (u128, u128),
    ) -> Self {
        debug_assert!(s == 0 || s == 1);
        if c.0 == 0 && c.1 == 0 {
            return Self::ZERO;
        }
        let (signif, exp) = match c.0.leading_zeros() {
            0 => (U256::new(c.0, c.1) >> 1, t + 1),
            1 => (U256::new(c.0, c.1), t),
            lz @ 2..=127 => {
                let sh = lz - 1;
                (U256::new(c.0, c.1) << sh, t - sh as i32)
            }
            128 => {
                let sh = 127 + c.1.leading_zeros();
                (U256::new(c.0, c.1) << sh, t - sh as i32)
            }
            _ => unreachable!(),
        };
        debug_assert!(signif.hi.0.leading_zeros() == 1);
        Self {
            signum: [1, -1][s as usize],
            exp: exp + Self::FRACTION_BITS as i32,
            signif,
        }
    }

    // TODO: remove (inline) this fn when trait fns can be constant!
    pub(crate) fn from_f256(f: &f256) -> Self {
        if f.eq_zero() {
            return Self::ZERO;
        }
        const PREC_ADJ: u32 = Float256::FRACTION_BITS - FRACTION_BITS;
        let abs_bits_f = abs_bits(f);
        debug_assert!(abs_bits_f.hi.0 < HI_EXP_MASK); // f is finite?
        debug_assert!(!abs_bits_f.is_zero());
        let signif_f = signif(&abs_bits_f);
        let shl = signif_f.leading_zeros()
            - (U256::BITS - Float256::FRACTION_BITS - 1);
        let exp_f = exp_bits(&abs_bits_f) as i32 + 1
            - norm_bit(&abs_bits_f) as i32
            - EXP_BIAS as i32
            - shl as i32
            + PREC_ADJ as i32;
        Self {
            signum: (-1_i32).pow(f.sign()),
            exp: exp_f,
            signif: signif_f.shift_left(shl),
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
    pub(crate) const fn signif(&self) -> U256 {
        self.signif
    }

    #[inline(always)]
    pub(crate) const fn quantum(&self) -> Self {
        Self {
            signum: 1,
            exp: self.exp - Self::FRACTION_BITS as i32,
            signif: SIGNIF_ONE,
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
        let sh = max(Self::FRACTION_BITS as i32 - self.exp, 0) as u32;
        Self {
            signum: self.signum,
            exp: self.exp,
            signif: &(&self.signif >> sh) << sh,
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
            0 => (self.signif, U256::ZERO),
            sh @ _ => self.signif.widening_shr(sh),
        };
        let (mut signif_other, rem_other) = match (exp - other.exp) as u32 {
            0 => (other.signif, U256::ZERO),
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
            *self = Float256::ZERO;
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
            let rnd = prod_signif.lo > TIE
                || (prod_signif.lo == TIE && prod_signif.hi.is_odd());
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
            div_signifs(&self.signif, &other.signif);
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
        const UPPER_LIM_PROD_TOO_SMALL: i32 =
            -(Float256::FRACTION_BITS as i32) - 2;
        const LOWER_LIM_ADDEND_TOO_SMALL: i32 =
            2 * Float256::FRACTION_BITS as i32 + 3;
        match exp_diff {
            i32::MIN..=UPPER_LIM_PROD_TOO_SMALL => {
                *self = *a;
            }
            LOWER_LIM_ADDEND_TOO_SMALL..=i32::MAX => {
                self.imul(f);
            }
            _ => {
                // -256 < exp_diff < 511
                let prod_sign = self.signum * f.signum;
                let (mut prod_signif, exp_adj) =
                    mul_signifs(&self.signif, &f.signif);
                prod_exp += exp_adj;
                exp_diff += exp_adj;
                let mut addend_signif = U512 {
                    hi: a.signif,
                    lo: U256::ZERO,
                };
                let (x_sign, mut x_signif, mut x_exp, y_signif) =
                    match exp_diff {
                        0 => {
                            // exponents equal => check significands
                            if prod_signif >= addend_signif {
                                // |prod| >= |addend|
                                (
                                    prod_sign,
                                    prod_signif,
                                    prod_exp,
                                    addend_signif,
                                )
                            } else {
                                // |addend| > |prod|
                                (a.signum, addend_signif, a.exp, prod_signif)
                            }
                        }
                        1..=i32::MAX => {
                            // |prod| > |addend|
                            let (q, r) =
                                addend_signif.widening_shr(exp_diff as u32);
                            addend_signif = q;
                            addend_signif.lo.lo.0 |=
                                (r != U512::ZERO) as u128;
                            (prod_sign, prod_signif, prod_exp, addend_signif)
                        }
                        i32::MIN..=-1 => {
                            // |addend| > |prod|
                            let (q, r) = prod_signif
                                .widening_shr(exp_diff.unsigned_abs());
                            prod_signif = q;
                            prod_signif.lo.lo.0 |= (r != U512::ZERO) as u128;
                            (a.signum, addend_signif, a.exp, prod_signif)
                        }
                    };
                self.signum = x_sign;
                self.exp = x_exp;
                if prod_sign == a.signum {
                    x_signif += &y_signif;
                    // addition may have overflowed
                    let mut shr = (x_signif.leading_zeros() == 0) as u32;
                    self.exp += shr as i32;
                    let mut rnd = (x_signif.hi.lo.0 & shr as u128) == 1;
                    x_signif.hi >>= shr;
                    rnd &= (x_signif.lo != U256::ZERO)
                        || (x_signif.lo == U256::ZERO
                            && x_signif.hi.is_odd());
                    rnd |= (x_signif.lo > TIE)
                        || (x_signif.lo == TIE && x_signif.hi.is_odd());
                    x_signif.hi.incr_if(rnd);
                    shr = (x_signif.hi.leading_zeros() == 0) as u32;
                    self.exp += shr as i32;
                    x_signif.hi >>= shr;
                    self.signif = x_signif.hi;
                } else {
                    x_signif -= &y_signif;
                    // subtraction may have cancelled some or all leading bits
                    let shl = x_signif.leading_zeros() - 1;
                    self.exp -= shl as i32;
                    match shl {
                        0..=256 => {
                            // shifting left by shl bits and then rounding the
                            // low 256 bits => shift right and round by
                            // 256 - shl bits
                            x_signif =
                                x_signif.rounding_div_pow2(U256::BITS - shl);
                            self.signif = x_signif.lo;
                        }
                        257..=510 => {
                            // less than 255 bits left => shift left, no
                            // rounding
                            self.signif = &x_signif.lo << shl - 256;
                        }
                        _ => {
                            // all bits cancelled => result is zero
                            *self = Float256::ZERO;
                        }
                    }
                };
            }
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
        let s = *self + rhs;
        let d = s - rhs;
        let t1 = *self - &d;
        let t2 = *rhs - &(s - &d);
        let r = t1 + &t2;
        (s, r)
    }

    /// Computes the rounded product of two BigFloat values and the remainder.
    #[inline]
    pub(crate) fn mul_exact(&self, rhs: &Self) -> (Self, Self) {
        let p = *self * rhs;
        let r = self.mul_add(rhs, &p.neg());
        (p, r)
    }

    /// Computes `self * self` .
    #[inline(always)]
    #[must_use]
    pub fn square(self) -> Self {
        self * &self
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
        let mut q = SIGNIF_ONE << 1;
        let mut r = (U512::from(&self.signif) << (1 + exp_is_odd as u32)) - q;
        let mut s = q;
        for _ in 0..=Float256::FRACTION_BITS {
            if r.is_zero() {
                break;
            }
            s >>= 1;
            let t = &r << 1;
            let u = (U512::from(&q) << 1) + s;
            if t < u {
                r = t;
            } else {
                q += &s;
                r = t - u;
            }
        }
        // Final rounding
        let rnd_bits = (q.lo.0 & 3_u128) as u32;
        q.incr_if(rnd_bits == 3 || rnd_bits == 1 && !r.is_zero());
        q >>= 1;
        Self::new(1, exp, (q.hi.0, q.lo.0))
    }
}

impl From<&U256> for Float256 {
    /// Convert a raw U256 into a Float, without any modification, i.e
    /// interptret the given value ui as ui⋅2⁻²⁵⁴
    #[inline(always)]
    fn from(ui: &U256) -> Self {
        Self {
            signum: 1,
            exp: 0,
            signif: *ui,
        }
    }
}

impl From<&f256> for Float256 {
    #[inline(always)]
    fn from(f: &f256) -> Self {
        Self::from_f256(f)
    }
}

impl From<&Float256> for f256 {
    fn from(fp: &Float256) -> Self {
        if fp.is_zero() {
            return Self::ZERO;
        }
        const PREC_ADJ: u32 = Float256::FRACTION_BITS - FRACTION_BITS;
        const EXP_UNDERFLOW: i32 = EMIN - SIGNIFICAND_BITS as i32;
        const EXP_LOWER_SUBNORMAL: i32 = EXP_UNDERFLOW + 1;
        const EXP_UPPER_SUBNORMAL: i32 = EMIN - 1;
        const EXP_OVERFLOW: i32 = f256::MAX_EXP;
        let mut f256_bits = match fp.exp {
            ..=EXP_UNDERFLOW => U256::ZERO,
            EXP_LOWER_SUBNORMAL..=EXP_UPPER_SUBNORMAL => {
                fp.signif.rounding_div_pow2(
                    PREC_ADJ.saturating_add_signed(EMIN - fp.exp),
                )
            }
            EMIN..=EMAX => {
                const TIE: U256 = U256::new(1_u128 << 127, 0);
                let (mut bits, rem) = fp.signif.widening_shr(PREC_ADJ);
                // -1 because we add the significand incl. hidden bit.
                let exp_bits = (EXP_BIAS.saturating_add_signed(fp.exp - 1)
                    as u128)
                    << HI_FRACTION_BITS;
                bits.hi.0 += exp_bits;
                // Final rounding. Possibly overflowing into the exponent,
                // but that is ok.
                if rem > TIE || (rem == TIE && bits.lo.is_odd()) {
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
mod from_into_f256_tests {
    use super::*;

    fn assert_normal_eq(f: &f256, g: &Float256) {
        const PREC_DIFF: u32 = Float256::FRACTION_BITS - FRACTION_BITS;
        debug_assert!(f.is_normal());
        assert_eq!((-1_i32).pow(f.sign()), g.signum);
        assert_eq!(f.quantum_exponent() + FRACTION_BITS as i32, g.exp);
        assert_eq!(&f.integral_significand() << PREC_DIFF, g.signif)
    }

    #[test]
    fn test_neg_one() {
        let fp = Float256::from(&f256::NEG_ONE);
        assert_eq!(fp, Float256::NEG_ONE);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_normal_gt_one() {
        let f = f256::from(1.5);
        let fp = Float256::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_normal_lt_one() {
        let f = f256::from(0.625);
        let fp = Float256::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_normal_lt_minus_one() {
        let f = f256::from(-7.5);
        let fp = Float256::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_min_f256() {
        let f = f256::MIN;
        let fp = Float256::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_epsilon() {
        let f = f256::EPSILON;
        let fp = Float256::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        let fp = Float256::from(&f);
        assert_eq!((-1_i32).pow(f.sign()), fp.signum);
        assert_eq!(f.quantum_exponent(), fp.exp);
        assert_eq!(
            &f.integral_significand() << Float256::FRACTION_BITS,
            fp.signif
        );
        let f = f256::from(&fp);
        assert_eq!((-1_i32).pow(f.sign()), fp.signum);
        assert_eq!(f.quantum_exponent(), fp.exp);
        assert_eq!(
            &f.integral_significand() << Float256::FRACTION_BITS,
            fp.signif
        )
    }
}

#[cfg(test)]
mod into_f256_tests {
    use super::*;
    use crate::consts::PI;

    #[test]
    fn test_overflow_1() {
        let fp = Float256 {
            signum: -1,
            exp: f256::MAX_EXP,
            signif: Float256::ONE.signif,
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::NEG_INFINITY);
    }

    #[test]
    fn test_overflow_2() {
        let fp = Float256 {
            signum: 1,
            exp: EMAX,
            signif: U256::new(u128::MAX >> 1, u128::MAX - 7),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::INFINITY);
    }

    #[test]
    fn test_overflow_3() {
        let sh = Float256::FRACTION_BITS - FRACTION_BITS - 1;
        let fp = Float256 {
            signum: 1,
            exp: 0,
            signif: U256::new(u128::MAX >> 1, (u128::MAX >> sh) << sh),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::TWO);
    }

    #[test]
    fn test_underflow() {
        let fp = Float256 {
            signum: 1,
            exp: EMIN - SIGNIFICAND_BITS as i32,
            signif: U256::new(1_u128 << 127, 0_u128),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::ZERO);
    }

    #[test]
    fn test_round_to_epsilon() {
        let fp = Float256 {
            signum: 1,
            exp: -237,
            signif: U256::new(u128::MAX >> 1, u128::MAX),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::EPSILON);
    }

    #[test]
    fn test_f256_pi() {
        let f = PI;
        let fp = Float256::from(&f);
        let g = f256::from(&fp);
        assert_eq!(f, g);
    }
}

impl Neg for Float256 {
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

impl PartialOrd for Float256 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.signum, self.exp, self.signif).cmp(&(
            other.signum,
            other.exp,
            other.signif,
        )))
    }
}

impl AddAssign<Self> for Float256 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.iadd(&rhs);
    }
}

impl AddAssign<&Self> for Float256 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Self) {
        self.iadd(rhs);
    }
}

impl Add<&Self> for Float256 {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        let mut res = self;
        res += rhs;
        res
    }
}

impl SubAssign<Self> for Float256 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.isub(&rhs);
    }
}

impl SubAssign<&Self> for Float256 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &Self) {
        self.isub(rhs);
    }
}

impl Sub<&Self> for Float256 {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        let mut res = self;
        res -= rhs;
        res
    }
}

impl MulAssign<Self> for Float256 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        self.imul(&rhs);
    }
}

impl MulAssign<&Self> for Float256 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &Self) {
        self.imul(rhs);
    }
}

impl Mul<&Self> for Float256 {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self::Output {
        let mut res = self;
        res *= rhs;
        res
    }
}

impl DivAssign<Self> for Float256 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        self.idiv(&rhs);
    }
}

impl DivAssign<&Self> for Float256 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: &Self) {
        self.idiv(rhs);
    }
}

impl Div<&Self> for Float256 {
    type Output = Self;

    fn div(self, rhs: &Self) -> Self::Output {
        let mut res = self;
        res *= rhs;
        res
    }
}

#[cfg(test)]
mod add_sub_tests {
    use super::*;

    #[test]
    fn test_add_same_sign() {
        let mut f = Float256::ONE;
        f += f;
        assert_eq!(f.signum, 1);
        assert_eq!(f.exp, 1);
        assert_eq!(f.signif, Float256::ONE.signif);
        f.flip_sign();
        f += &f.clone();
        assert_eq!(f.signum, -1);
        assert_eq!(f.exp, 2);
        assert_eq!(f.signif, Float256::ONE.signif);
    }

    #[test]
    fn test_add_diff_sign() {
        let mut f = Float256::ONE;
        f += &Float256::NEG_ONE;
        assert_eq!(f, Float256::ZERO);
        let mut f = Float256 {
            signum: -1,
            exp: 0,
            signif: U256::new(Float256::ONE.signif.hi.0, 1),
        };
        f += &Float256::EPSILON;
        assert_eq!(f, Float256::NEG_ONE);
        let mut g = Float256::EPSILON;
        g += &f.clone();
        assert_eq!(g.signum, -1);
        assert_eq!(g.exp, -1);
        assert_eq!(g.signif, U256::new(u128::MAX >> 1, u128::MAX - 1));
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

    #[test]
    fn test_sub_diff_sign() {
        let mut f = Float256::NEG_ONE;
        f -= &Float256::ONE;
        assert_eq!(f.signum, -1);
        assert_eq!(f.exp, 1);
        assert_eq!(f.signif, Float256::ONE.signif);
        let mut g = f;
        g.flip_sign();
        f -= &g;
        assert_eq!(f.signum, -1);
        assert_eq!(f.exp, 2);
        assert_eq!(f.signif, Float256::ONE.signif);
    }

    #[test]
    fn test_sub_same_sign() {
        let mut f = Float256::NEG_ONE;
        f -= f;
        assert_eq!(f, Float256::ZERO);
        let mut f = Float256 {
            signum: 1,
            exp: 0,
            signif: U256::new(Float256::ONE.signif.hi.0, 1),
        };
        f -= &Float256::EPSILON;
        assert_eq!(f, Float256::ONE);
        let mut g = Float256::EPSILON;
        g -= &f.clone();
        assert_eq!(g.signum, -1);
        assert_eq!(g.exp, -1);
        assert_eq!(g.signif, U256::new(u128::MAX >> 1, u128::MAX - 1));
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
        assert_eq!(a - &b, a);
    }
}

#[cfg(test)]
mod mul_tests {
    use super::*;

    #[test]
    fn test_imul_same_sign() {
        let mut x = Float256::NEG_ONE;
        x += &Float256::EPSILON;
        let y = x;
        x *= x;
        assert_eq!(x.signum, 1);
        assert_eq!(x.exp, -1);
        assert_eq!(x.signif, U256::new(y.signif.hi.0, y.signif.lo.0 - 2));
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

    #[test]
    fn test_idiv_by_one() {
        let mut x = Float256::ONE;
        x.idiv(&Float256::ONE);
        assert_eq!(x, Float256::ONE);
        let mut x = -Float256::ONE_HALF;
        x.idiv(&Float256::NEG_ONE);
        assert_eq!(x, Float256::ONE_HALF);
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
        let d = (q - &z.recip()).abs();
        assert!(d <= Float256::EPSILON);
    }
}

#[cfg(test)]
mod sqrt_tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(Float256::ZERO.sqrt(), Float256::ZERO);
    }

    #[test]
    fn test_one() {
        assert_eq!(Float256::ONE.sqrt(), Float256::ONE);
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
