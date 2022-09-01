// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::f256;

impl PartialEq for f256 {
    fn eq(&self, other: &Self) -> bool {
        if self.is_nan() || other.is_nan() {
            return false;
        }
        self.repr == other.repr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ne_nan() {
        assert!(f256::NAN != f256::NAN);
        assert!(f256::NAN != f256::ONE);
        assert!(f256::ONE != f256::NAN);
        assert!(f256::NAN != f256::INFINITY);
        assert!(f256::INFINITY != f256::NAN);
        assert!(f256::NAN != f256::NEG_INFINITY);
        assert!(f256::NEG_INFINITY != f256::NAN);
    }

    #[test]
    fn test_eq() {
        assert_eq!(f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::ONE, f256::ONE);
        assert_ne!(f256::INFINITY, f256::ONE);
        assert_ne!(f256::ONE, f256::INFINITY);
        assert_ne!(f256::NEG_INFINITY, f256::ONE);
        assert_ne!(f256::ONE, f256::NEG_INFINITY);
        assert_ne!(f256::INFINITY, f256::NEG_INFINITY);
        assert_ne!(f256::NEG_INFINITY, f256::INFINITY);
    }
}
