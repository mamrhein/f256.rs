// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{
    approx_cos::approx_cos, approx_sin::approx_sin, reduce::reduce, Float256,
    FP492,
};
use crate::{f256, HI_ABS_MASK};

impl f256 {
    /// Computes the sine of a number (in radians).
    #[inline(always)]
    pub fn sin(&self) -> Self {
        if self.is_special() {
            // x is NAN or infinite => sine x is NAN
            if (self.bits.hi.0 & HI_ABS_MASK) > Self::MAX.bits.hi.0 {
                return Self::NAN;
            }
            // x = 0 => sine x = 0
            return Self::ZERO;
        }
        // Calculate ⌈x/½π⌋ % 4 and x % ½π.
        let (quadrant, fx) = reduce(&self.abs());
        // Map result according to quadrant and sign
        match (quadrant, self.sign()) {
            (0, 0) => Self::from(&approx_sin(&fx)),
            (0, 1) => -Self::from(&approx_sin(&fx)),
            (1, 0) => Self::from(&approx_cos(&fx)),
            (1, 1) => -Self::from(&approx_cos(&fx)),
            (2, 0) => -Self::from(&approx_sin(&fx)),
            (2, 1) => Self::from(&approx_sin(&fx)),
            (3, 0) => -Self::from(&approx_cos(&fx)),
            (3, 1) => Self::from(&approx_cos(&fx)),
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod sin_tests {
    use super::*;
    use crate::consts::FRAC_PI_3;

    #[test]
    fn test_neg_values() {
        // sin(-x) = -sin(x)
        for f in [f256::MIN, -FRAC_PI_3, f256::NEG_ONE] {
            assert_eq!(f.sin(), -f.abs().sin());
        }
    }

    #[test]
    fn test_small_value() {
        let f = f256::from_sign_exp_signif(
            0,
            -268,
            (
                511713792246730580583350097904921,
                338234285556250629981528767706881153057,
            ),
        );
        let sin = f256::from_sign_exp_signif(
            0,
            -268,
            (
                511713792246730580571854506161847,
                105438061704425261882515718706001931297,
            ),
        );
        assert_eq!(f.sin(), sin);
    }

    #[test]
    fn test_some_lt_2pi() {
        let f = f256::from_sign_exp_signif(
            0,
            -261,
            (
                0x0000100410f1f3ab981fc5a9fd008e6e,
                0x6ba97c4190d331836d7fd41d2009cdf8,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            0,
            -261,
            (
                0x0000100410f1f3ab977498bfffb5d0d5,
                0xd4afb6f12a8836a249b17fbeb758fa8e,
            ),
        );
        assert_eq!(f.sin(), sin_f, "{f}\n{}\n{}", f.sin(), sin_f);
        let f = f256::from_sign_exp_signif(
            0,
            -235,
            (
                0x000019412990c230cfe83e598062a70f,
                0x2e55ff0ee1b47200750f278655e459cc,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            1,
            -243,
            (
                0x00001f2ded8c6c188d6563858850bd6f,
                0xc0dd632c3566aef3b1af2c6bd810e0fe,
            ),
        );
        assert_eq!(f.sin(), sin_f, "{f}\n{}\n{}", f.sin(), sin_f);
        let f = f256::from_sign_exp_signif(
            0,
            -230,
            (
                0x000001709d10d3e7eab960be165f5516,
                0xe8df7f75d98f0fa868f6d4ae6add8617,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            1,
            -237,
            (
                0x00000fffffffffffffffffffffffffff,
                0xfffffffffffffffffffffffffffffffd,
            ),
        );
        assert_eq!(f.sin(), sin_f, "{f}\n{}\n{}", f.sin(), sin_f);
        let f = f256::from_sign_exp_signif(
            0,
            -236,
            (
                0x00001dd7c55c574a05bc3cfce7f8b6a2,
                0x621b3e52a18d6fb6f7ba91cfa8cde8ef,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            0,
            -237,
            (
                0x00001e9f97e097ab51eacfb059e3245d,
                0x3ee3796a535e4c29d2039ba678a55710,
            ),
        );
        assert_eq!(f.sin(), sin_f, "{f}\n{}\n{}", f.sin(), sin_f);
    }

    #[test]
    fn test_some_gt_2pi() {
        // 451072.762503992264821001752482581001682512026226517387166060002390623476
        let f = f256::from_sign_exp_signif(
            0,
            -218,
            (
                0x00001b88030ccdd8b7632adb619b1f1f,
                0x0e1d0adefedbcedd03c621b5967e9c1d,
            ),
        );
        // 0.249623167582990240382008743809080852087294584792829298735057191909919803
        let sin_f = f256::from_sign_exp_signif(
            0,
            -239,
            (
                0x00001ff3a6e68be32dc92aa6c6930521,
                0x192865a8b728d2d42fcb7319995fc955,
            ),
        );
        assert_eq!(f.sin(), sin_f);
        // 2.16570771504765655239445414400334979116856541624320036012358643343267191e72080
        let f = f256::from_sign_exp_signif(
            0,
            239209,
            (
                0x000019d8cce297b67f216c4c503ced7a,
                0xc2b9ba4acc896934b14162d8144bc69b,
            ),
        );
        // 8.66266758462195759181693148633759722506161200889060317576189227767715537e-1
        let sin_f = f256::from_sign_exp_signif(
            0,
            -237,
            (
                0x00001bb87510a6a0409133589aff0e34,
                0x84d9e6557596b89f83c0ef5fe227db66,
            ),
        );
        assert_eq!(f.sin(), sin_f);
        // 1.64760097351429814415521962742312041989746625767819932300237402156549037e72
        let f = f256::from_sign_exp_signif(
            0,
            3,
            (
                0x00001dd71d552efac6c6246fb9a1e568,
                0x0e751b9e3f84e2a77bbc0298a0f4b498,
            ),
        );
        assert_eq!(f.sin(), f256::NEG_ONE);
    }
}
