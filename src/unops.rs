// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::Neg;

use crate::f256;

impl f256 {
    /// Computes the absolute value of `self`.
    #[inline(always)]
    pub const fn abs(&self) -> Self {
        Self {
            repr: self.repr.abs(),
        }
    }

    /// Returns the smallest integer greater than or equal to `self`.
    #[inline]
    pub const fn ceil(&self) -> Self {
        unimplemented!()
    }

    /// Returns the largest integer less than or equal to `self`.
    #[inline]
    pub const fn floor(&self) -> Self {
        unimplemented!()
    }

    /// Returns the fractional part of `self`.
    #[inline]
    pub const fn fract(&self) -> Self {
        unimplemented!()
    }

    /// Performs the unary `-` operation.
    #[inline(always)]
    pub(crate) const fn neg(&self) -> Self {
        Self {
            repr: self.repr.neg(),
        }
    }

    /// Returns the nearest integer to `self`. Rounds half-way cases away from
    /// 0.0.
    #[inline]
    pub const fn round(&self) -> Self {
        unimplemented!()
    }

    /// Returns the integer part of `self`. This means that non-integer numbers
    /// are always truncated towards zero.
    #[inline]
    pub const fn trunc(&self) -> Self {
        unimplemented!()
    }
}

impl Neg for f256 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        f256::neg(&self)
    }
}

impl Neg for &f256 {
    type Output = <f256 as Neg>::Output;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        self.neg()
    }
}