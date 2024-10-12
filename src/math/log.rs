// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::BigFloat;
use crate::{f256, signif};

mod approx_ln;

// LN_2 = ◯₂₅₅(ln(2)) =
// 0.6931471805599453094172321214581765680755001343602552541206800094933936219697
pub(crate) const LN_2: BigFloat = BigFloat::new(
    1,
    -1,
    (
        0x58b90bfbe8e7bcd5e4f1d9cc01f97b57,
        0xa079a193394c5b16c5068badc5d57d16,
    ),
);
// LN_10 = ◯₂₅₅(ln(10)) =
// 2.3025850929940456840179914546843642076011014886287729760333279009675726096773
pub(crate) const LN_10: BigFloat = BigFloat::new(
    1,
    1,
    (
        0x49aec6eed554560b752b6b15c1698514,
        0x7147f67ced2efc8741e30f4100f816b9,
    ),
);

impl f256 {
    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// The result might not be correctly rounded owing to implementation
    /// details; self.log2() can produce more accurate results for base 2, and
    /// self.log10() can produce more accurate results for base 10.
    pub fn log(self, base: Self) -> Self {
        unimplemented!()
    }

    /// Returns the natural logarithm of the number.
    pub fn ln(&self) -> Self {
        //     // x <= 0 or x is infinite or nan => ln x is nan
        //     if self.is_special() || self.is_sign_negative() {
        //         return Self::NAN;
        //     }
        //     // x = m⋅2ᵉ => ln x = ln m + ln 2ᵉ = ln m + e⋅ln 2
        //     let exp = self.quantum_exponent();
        //     let signif = signif(&self.bits);
        //
        //     let m = BigFloat::from(&signif);
        //
        unimplemented!()
    }

    /// Returns ln(1+n) (natural logarithm) more accurately than if the
    /// operations were performed separately.
    pub fn ln_1p(&self) -> Self {
        unimplemented!()
    }

    /// Returns the base 2 logarithm of the number.
    pub fn log2(&self) -> Self {
        unimplemented!()
    }

    /// Returns the base 10 logarithm of the number.
    pub fn log10(&self) -> Self {
        unimplemented!()
    }
}
