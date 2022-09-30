// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::cmp::Ordering;

use crate::f256;

impl PartialEq for f256 {
    fn eq(&self, other: &Self) -> bool {
        // NaN is not equal to any other value, incl. NaN.
        if self.is_nan() || other.is_nan() {
            return false;
        }
        // ±0 == ±0
        if self.is_zero() && other.is_zero() {
            return true;
        }
        // All other values are equal if their bit representations are equal.
        self.bits == other.bits
    }
}

impl PartialOrd for f256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // NaN is not ordered.
        if self.is_nan() || other.is_nan() {
            return None;
        }
        // ±0 == ±0
        if self.is_zero() && other.is_zero() {
            return Some(Ordering::Equal);
        }
        Some(self.total_cmp(other))
    }
}
