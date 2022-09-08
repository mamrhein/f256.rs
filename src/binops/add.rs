// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{Add, Sub};

use crate::{
    f256,
    float256::{add, add_special},
};

impl Add for f256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_special() || rhs.is_special() {
            Self::Output {
                repr: add_special(self.repr, rhs.repr),
            }
        } else {
            Self::Output {
                repr: add(self.repr, rhs.repr),
            }
        }
    }
}

forward_ref_binop!(impl Add, add);

impl Sub for f256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_special() || rhs.is_special() {
            Self::Output {
                repr: add_special(self.repr, -rhs.repr),
            }
        } else {
            Self::Output {
                repr: add(self.repr, -rhs.repr),
            }
        }
    }
}

forward_ref_binop!(impl Sub, sub);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nan() {
        assert!((f256::NAN + f256::ONE).is_nan());
        assert!((f256::ONE + f256::NAN).is_nan());
        assert!((f256::NAN + f256::NAN).is_nan());
        assert!((f256::NAN + f256::INFINITY).is_nan());
        assert!((f256::INFINITY + f256::NAN).is_nan());
        assert!((f256::NAN + f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY + f256::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY + f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::INFINITY + f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE + f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY + f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_INFINITY + f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE + f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert!((f256::INFINITY + f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY + f256::INFINITY).is_nan());
    }

    #[test]
    fn test_pos_add_pos() {
        assert_eq!(f256::ONE + f256::ONE, f256::TWO);
    }
}
