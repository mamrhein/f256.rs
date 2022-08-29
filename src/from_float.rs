// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{f256, float256, uint256::u256};

impl From<f64> for f256 {
    fn from(f: f64) -> Self {
        let bits = f.to_bits();
        // sign bit at pos 63
        let sign = (bits >> 63) as u32;
        // biased exponent at bit pos 52 .. 62
        let biased_exp = ((bits >> 52) & 0x7ff) as u32;
        // fraction at bit pos 0 .. 51
        let fraction = bits & 0xfffffffffffff;
        // check special values
        if biased_exp == 0x7ff {
            return if fraction != 0 {
                f256::NAN
            } else {
                // +/- inf
                f256 {
                    bits: u256 {
                        hi: ((sign as u128) << float256::HI_SIGN_SHIFT)
                            | ((float256::EXP_MAX as u128)
                                << float256::HI_FRACTION_BITS),
                        lo: 0,
                    },
                }
            };
        }
        if biased_exp == 0 {
            return if fraction == 0 {
                // +/- zero
                f256 {
                    bits: u256 {
                        hi: ((sign as u128) << float256::HI_SIGN_SHIFT),
                        lo: 0,
                    },
                }
            } else {
                // subnormal f64
                f256 {
                    bits: u256 {
                        hi: ((sign as u128) << float256::HI_SIGN_SHIFT)
                            | ((fraction as u128)
                                << (float256::HI_FRACTION_BITS - 52)),
                        lo: 0,
                    },
                }
            };
        }
        // normal f64
        f256 {
            bits: u256 {
                hi: ((sign as u128) << float256::HI_SIGN_SHIFT)
                    | (((biased_exp + (float256::EXP_BIAS - 0x3ff)) as u128)
                        << float256::HI_FRACTION_BITS)
                    | ((fraction as u128) << (float256::HI_FRACTION_BITS - 52)),
                lo: 0,
            },
        }
    }
}

#[cfg(test)]
mod from_f64_tests {
    use super::*;

    #[test]
    fn test_nan() {
        assert!(f256::from(f64::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::from(f64::INFINITY), f256::INFINITY);
        assert_eq!(f256::from(f64::NEG_INFINITY), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::from(0_f64), f256::ZERO);
        assert_eq!(f256::from(-0_f64), f256::NEG_ZERO);
    }

    #[test]
    fn test_normal_values() {
        assert_eq!(f256::from(1_f64), f256::ONE);
        assert_eq!(f256::from(-1_f64), f256::NEG_ONE);
        assert_eq!(f256::from(2_f64), f256::TWO);
        // TODO: more tests
    }

    // TODO: test subnormal values
}
