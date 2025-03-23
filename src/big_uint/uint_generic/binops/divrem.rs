// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::Ordering,
    convert::From,
    ops::{Div, Rem},
};

use super::{BigUInt, DivRem, HiLo, UInt, U128, U256};

impl<SubUInt> DivRem<u128> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo + for<'a> From<&'a [u128]>,
{
    type Output = (Self, u128);

    #[inline(always)]
    fn div_rem(self, rhs: u128) -> Self::Output {
        (&self).div_rem(&rhs)
    }
}

impl<SubUInt> DivRem<&u128> for &UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo + for<'a> From<&'a [u128]>,
{
    type Output = (UInt<SubUInt>, u128);

    #[inline(always)]
    fn div_rem(self, rhs: &u128) -> Self::Output {
        let mut chunks = self.as_vec_u128();
        let mut rem = U128::new(chunks[0] % *rhs);
        chunks[0] /= *rhs;
        let rhs = U128::new(*rhs);
        for c in &mut chunks[1..] {
            let mut t = U256::from_hi_lo(rem, U128::from(*c));
            (t, rem) = t.div_rem(rhs);
            debug_assert!(t.hi.is_zero());
            *c = t.lo.into();
        }
        (UInt::<SubUInt>::from(&chunks[..]), rem.into())
    }
}

impl<SubUInt> DivRem<SubUInt> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = (Self, SubUInt);

    #[inline(always)]
    fn div_rem(self, rhs: SubUInt) -> Self::Output {
        (&self).div_rem(&rhs)
    }
}

impl<SubUInt> DivRem<&SubUInt> for &UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = (UInt<SubUInt>, SubUInt);

    fn div_rem(self, rhs: &SubUInt) -> Self::Output {
        if self.hi.is_zero() {
            let (quot, rem) = self.lo.div_rem(*rhs);
            (UInt::<SubUInt>::from_hi_lo(SubUInt::ZERO, quot), rem)
        } else if self.hi < *rhs {
            self.div_rem_subuint_special(rhs)
        } else {
            let mut quot = *self;
            let rem;
            quot.hi = quot.hi % *rhs;
            (quot, rem) = quot.div_rem_subuint_special(rhs);
            (quot.hi, _) = self.hi.div_rem(*rhs);
            (quot, rem)
        }
    }
}

impl<SubUInt> DivRem for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = (Self, Self);

    #[inline(always)]
    fn div_rem(self, rhs: Self) -> Self::Output {
        (&self).div_rem(&rhs)
    }
}

impl<SubUInt> DivRem for &UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = <UInt<SubUInt> as DivRem>::Output;

    fn div_rem(self, rhs: Self) -> Self::Output {
        debug_assert!(!rhs.is_zero());
        if rhs.hi.is_zero() {
            let (quot, rem) = self.div_rem(&rhs.lo);
            (quot, UInt::<SubUInt>::from_hi_lo(SubUInt::ZERO, rem))
        } else if rhs.hi > self.hi {
            // self < rhs
            return (UInt::<SubUInt>::ZERO, *self);
        } else {
            // estimate the quotient
            let nlz = self.hi.leading_zeros();
            let mut quot = UInt::<SubUInt>::from_hi_lo(
                SubUInt::ZERO,
                (self << nlz).hi / (rhs << nlz).hi,
            );
            // trim the estimate
            let mut t = *rhs;
            t *= &quot;
            match (&t).cmp(&self) {
                Ordering::Greater => {
                    let mut d = &t - self;
                    let (mut n, _) = d.div_rem(*rhs);
                    n.incr();
                    debug_assert!(n.hi.is_zero());
                    quot -= &n;
                    d = *rhs;
                    d *= &n;
                    t -= &d;
                }
                Ordering::Less => {
                    let (_, ovl) = t.overflowing_add(&rhs);
                    if ovl {
                        let mut d = self - &t;
                        let (n, _) = d.div_rem(*rhs);
                        debug_assert!(n.hi.is_zero());
                        quot += &n;
                        d = *rhs;
                        d *= &n;
                        t += &d;
                    }
                }
                Ordering::Equal => return (quot, UInt::<SubUInt>::ZERO),
            }
            let rem = self - &t;
            (quot, rem)
        }
    }
}

impl<SubUInt> Div for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        self.div_rem(rhs).0
    }
}

impl<SubUInt> Div for &UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = UInt<SubUInt>;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        self.div_rem(rhs).0
    }
}

impl<SubUInt> Rem<u128> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = u128;

    #[inline(always)]
    fn rem(self, rhs: u128) -> Self::Output {
        self.div_rem(rhs).1
    }
}

impl<SubUInt> Rem<SubUInt> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = SubUInt;

    #[inline(always)]
    fn rem(self, rhs: SubUInt) -> Self::Output {
        self.div_rem(rhs).1
    }
}

impl<SubUInt> Rem<&SubUInt> for &UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = SubUInt;

    #[inline(always)]
    fn rem(self, rhs: &SubUInt) -> Self::Output {
        self.div_rem(rhs).1
    }
}

impl<SubUInt> Rem for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = Self;

    #[inline(always)]
    fn rem(self, rhs: Self) -> Self::Output {
        self.div_rem(rhs).1
    }
}

impl<SubUInt> Rem for &UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = UInt<SubUInt>;

    #[inline(always)]
    fn rem(self, rhs: Self) -> Self::Output {
        self.div_rem(rhs).1
    }
}

#[cfg(test)]
mod u512_div_rem_tests {
    use super::*;
    use crate::{U128, U256, U512};

    #[test]
    fn test_div_rem_1() {
        let v = U512::MAX;
        let t = U512::TWO.lo;
        assert_eq!(
            v.div_rem(t),
            (U512::from_hi_lo(&U256::MAX >> 1, U256::MAX), U256::ONE)
        );
    }

    #[test]
    fn test_div_rem_2() {
        let num = U512::from_hi_lo(
            U256::from_hi_lo(
                U128::from(134028236692093846346337460743176821145_u128),
                U128::from(204169420152563078078024764459060926873_u128),
            ),
            U256::from_hi_lo(
                U128::from(74093960915244683744395534668339322228_u128),
                U128::from(287120322474436508255079181112596091133_u128),
            ),
        );
        let den = U256::from(261829273180548883101888490383232063939_u128);
        let (quot, rem) = num.div_rem(den);
        assert_eq!(
            quot,
            U512::from_hi_lo(
                U256::from(174187725695499550061058271888170382541_u128),
                U256::from_hi_lo(
                    U128::from(309703370695449177740528536891543921158_u128),
                    U128::from(180178614806750523772503248486955614504_u128)
                )
            )
        );
        assert_eq!(
            rem,
            U256::from(101817417540912105633242427321208801157_u128)
        );
        assert_eq!(rem, num % den);
    }

    #[test]
    fn test_div_rem_by_10() {
        let v = U512::ZERO;
        assert_eq!(v.div_rem(10_u128), (U512::ZERO, 0_u128));
        let v = U512::from_hi_lo(
            U256::ZERO,
            U256::from_hi_lo(0.into(), 7.into()),
        );
        assert_eq!(v.div_rem(10_u128), (U512::ZERO, 7_u128));
        let v = U512::MAX;
        assert_eq!(
            v.div_rem(10_u128),
            (
                U512::from_hi_lo(
                    U256::from_hi_lo(
                        34028236692093846346337460743176821145.into(),
                        204169420152563078078024764459060926873.into()
                    ),
                    U256::from_hi_lo(
                        204169420152563078078024764459060926873.into(),
                        204169420152563078078024764459060926873.into()
                    )
                ),
                5
            )
        );
    }

    #[test]
    fn test_div_rem_by_pow10() {
        let v = U512::ZERO;
        assert_eq!(v.div_rem(10_u128.pow(10)), (U512::ZERO, 0));
        let v = U512::from_hi_lo(
            U256::ZERO,
            U256::from_hi_lo(2730.into(), 490003.into()),
        );
        assert_eq!(
            v.div_rem(10_u128.pow(5)),
            (
                U512::from_hi_lo(
                    U256::ZERO,
                    U256::from(9289708616941620052550126782887272177_u128)
                ),
                64883
            )
        );
        let v = U512::from_hi_lo(U256::ZERO, U256::MAX);
        let d = 10_u128.pow(38);
        let (q, r) = U256::MAX.div_rem(d);
        assert_eq!(v.div_rem(d), (U512::from_hi_lo(U256::ZERO, q), r));
        let v = U512::MAX;
        assert_eq!(
            v.div_rem(10_u128.pow(27)),
            (
                U512::from_hi_lo(
                    U256::from_hi_lo(
                        340282366920.into(),
                        319342568585932861179998458207245230120.into(),
                    ),
                    U256::from_hi_lo(
                        191932681663488487842607845281633842426.into(),
                        86732842386697408091259742201350722586.into(),
                    )
                ),
                811946569946433649006084095,
            )
        );
    }
}
