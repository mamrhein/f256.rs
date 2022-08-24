// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::u256::u256;

/// Internal representation of an unsigned finite `f256` value
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RawFloat {
    pub(crate) significand: u256,
    pub(crate) exponent: i32,
    pub(crate) normalized: bool,
}

impl RawFloat {
    pub(crate) fn normalize(&mut self) {
        let shift = self.significand.msb();
        self.significand <<= shift;
        self.exponent += shift as i32;
        self.normalized = true;
    }
}
