// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{
    approx_cos::{approx_cos, SMALL_CUT_OFF},
    approx_sin::approx_sin,
    FP509,
};

pub(crate) fn approx_sin_cos(x: &FP509) -> (FP509, FP509) {
    let mut x_abs = *x;
    x_abs.iabs();
    // If x is zero or very small, cosine x == 1 and sine x == x.
    if x_abs <= SMALL_CUT_OFF {
        return (*x, FP509::ONE);
    };
    (approx_sin(x), approx_cos(x))
}
