// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::math::BigFloat;

const N: usize = 32;

const COEFFS: [BigFloat; N] = [
    // -1 / 62! ≈
    // -3.1776321883905940513468372928764214080592493807301120491056222651235173770199e-86
    BigFloat::new(
        -0x7e6cc1644a2f1a41de2bf27aee1249a0,
        0x8b3d76be81a339428441f1cfba077d45,
        -285,
    ),
    // 1 / 60! ≈
    // 1.20178049364932267021937386416586257652800811579212837697174634066971427198893e-82
    BigFloat::new(
        0x74bbab10f9ff3dde9183d404419222bb,
        0x9c90e08426f315fecade63a62d64aa4b,
        -273,
    ),
    // -1 / 58! ≈
    // -4.2543029475186022525765834791471535209091487299041344544799820459707885228408e-79
    BigFloat::new(
        -0x64e331986c0fd8389b432efcadab8c84,
        0xa510360a36a996c274d4b39f5ebb402d,
        -261,
    ),
    // 1 / 56! ≈
    // 1.40647255444964990470181849820604895401256457010630685065108206439794268565115e-75
    BigFloat::new(
        0x516ddfe7a63849e4b05119ccb1ac964a,
        0x8fba359dfe5ea14eb38c2d7a81d5e2ac,
        -249,
    ),
    // -1 / 54! ≈
    // -4.33193546770492170648160097447463077835869887592742510000533275834566347180556e-72
    BigFloat::new(
        -0x7a763dbb60faa720ed29f7ccd7348e06,
        0x22270aa29b8c50975c05d0653d42a9e5,
        -238,
    ),
    // 1 / 52! ≈
    // 1.23979993085714859239503419889463932876625961829042906362152623543852888563075e-68
    BigFloat::new(
        0x55915e624d63238721b6b30540ded8bc,
        0x891d078e5e6faacfc26c8ffcbd2df476,
        -226,
    ),
    // -1 / 50! ≈
    // -3.2879494166331580670316306954685834998881205077062178767242875763829786046928e-65
    BigFloat::new(
        -0x6ecdbdb84b35e0817c2814d14d8091a8,
        0x238d1748db4999b008439473c6f9020f,
        -215,
    ),
    // 1 / 48! ≈
    // 8.0554760707512372642274952038980295747258952438802337979745045621382975814972e-62
    BigFloat::new(
        0x4246d01adbfc99e973637973b17b871f,
        0xb243c48d742a660d6af16c2bc063b15b,
        -203,
    ),
    // -1 / 46! ≈
    // -1.81731540156147912680972291799939547205816196701938074482304822921839993438576e-58
    BigFloat::new(
        -0x4902013d9654418b291b93c9718212d8,
        0xea5ea683d1f6b46ac7cdf12831edd15e,
        -192,
    ),
    // 1 / 44! ≈
    // 3.7618428812322617924961264402587486271603952717301181417837098344820878641786e-55
    BigFloat::new(
        0x49cac6c0ffb1a93f67cc9f9fdb7a388c,
        0xbee32acdbc781adaed73677f60771f5e,
        -181,
    ),
    // -1 / 42! ≈
    // -7.117406731291439311402671224969552402587467854113383524254779006840110239026e-52
    BigFloat::new(
        -0x442bd49d4c37a0db136489772e42693e,
        0x658dd0b109cf4cf405d1d1d2c9e0c7b,
        -170,
    ),
    // 1 / 40! ≈
    // 1.22561743912838584942353998493975692372556196447832464287667294497786698316025e-48
    BigFloat::new(
        0x72a3b50a84ab8c00681c912aeb4aadfa,
        0xcdac6fb71b6ff1ae85bc95758f87c7fc,
        -160,
    ),
    // -1 / 38! ≈
    // -1.91196320504028192510072237650602080101187666458618644288760979416547249373e-45
    BigFloat::new(
        -0x5752b2e7030eaba44f4dc293b139e286,
        0xaaa591879e64517efdea5d88c546d55,
        -149,
    ),
    // 1 / 36! ≈
    // 2.68822026628663638669161566136746524622269859040817813869997937059665432618437e-42
    BigFloat::new(
        0x77e60ca430b2a4ac1ae344a9c9d6fa87,
        0xba4e5551b5fabde5dda34b6d4adec1e,
        -139,
    ),
    // -1 / 34! ≈
    // -3.3871575355211618472314357333230062102406002239143044547619740069517844509924e-39
    BigFloat::new(
        -0x49c408c703f5e84fe28ad2be75adc322,
        0x15a9f317dc575c3d4ebdbf6e7bd900c5,
        -128,
    ),
    // 1 / 32! ≈
    // 3.8003907548547435925936708927884129678899534512318495982429348357999021540133e-36
    BigFloat::new(
        0x50d34b9e0fd6f10b87b91be9aff0e44e,
        0xd8bcb6dba4edb8912cc6e8408eb34557,
        -118,
    ),
    // -1 / 30! ≈
    // -3.7699876288159056438529215256461056641468338236219948014569913571135029367813e-33
    BigFloat::new(
        -0x4e4cb1411f5839832b7b530a62715d2c,
        0x61f6d124c7c64acca360b0fe8a3dab2d,
        -108,
    ),
    // 1 / 28! ≈
    // 3.2798892370698379101520417273121119278077454265511354772675824806887475549997e-30
    BigFloat::new(
        0x42862898d42174dcf171470d52a350a9,
        0x353b32b0bfbaf88cdccea660427364ee,
        -98,
    ),
    // -1 / 26! ≈
    // -2.4795962632247974600749435458479566174226555424726584208142923554006931515798e-27
    BigFloat::new(
        -0x623a17f1a939668e3c8142e9ac052519,
        0xd89968d8fb1a12fffe0921aa221e6308,
        -89,
    ),
    // 1 / 24! ≈
    // 1.61173757109611834904871330480117180132472610260722797352929003101045054852685e-24
    BigFloat::new(
        0x7cb3c065cbd5df3292d019f2a7628819,
        0xcffac21b76c81a1efd8197bcfd5093b9,
        -80,
    ),
    // -1 / 22! ≈
    // -8.8967913924505732867488974425024683433124880863918984138816809711776870278683e-22
    BigFloat::new(
        -0x4338e5b6dfe14a5143242dfcce3b1d5d,
        0xea1d2ca2ce07de14b4a7dbcbe08d6fa2,
        -70,
    ),
    // 1 / 20! ≈
    // 4.1103176233121648584779906184361403746103694959130570672133366086840914068751e-19
    BigFloat::new(
        0x7950ae900808941ea72b4afe3c2eaeff,
        0x7c80a68dcfd232c95e06eea9ef3f3f76,
        -62,
    ),
    // -1 / 18! ≈
    // -1.56192069685862264622163643500573334235194040844696168554106791129995473461256e-16
    BigFloat::new(
        -0x5a09e18ee5f65deec01221a8b0aaa5e1,
        0x9e677b9d403e01b173c925221f90f11a,
        -53,
    ),
    // 1 / 16! ≈
    // 4.7794773323873852974382074911175440275969376498477027577556678085778614879144e-14
    BigFloat::new(
        0x6b9fcf9ccee07c476195ac3ba32bfa47,
        0xaf57b1c1f6ca1e061c666e62c9bb4031,
        -45,
    ),
    // -1 / 14! ≈
    // -1.14707455977297247138516979786821056662326503596344866186136027405868675709947e-11
    BigFloat::new(
        -0x64e5d2a301f27482eb7c5177e8f93aa3,
        0x346236a5d75d7c25baa0077c9d1f8c2e,
        -37,
    ),
    // 1 / 12! ≈
    // 2.087675698786809897921009032120143231254342365453476564587675698786809897921e-9
    BigFloat::new(
        0x47bb63bfe3625ed5136a61eb3fa12fb0,
        0x73dd2d9e71c7642d2adc55297b46da8,
        -29,
    ),
    // -1 / 10! ≈
    // -2.75573192239858906525573192239858906525573192239858906525573192239858906525576e-7
    BigFloat::new(
        -0x49f93edde27d71cbbc05b4fa999e392d,
        0x8777c170b65559f4e943337d2c721116,
        -22,
    ),
    // 1 / 8! ≈
    // 2.4801587301587301587301587301587301587301587301587301587301587301587301587302e-5
    BigFloat::new(
        0x68068068068068068068068068068068,
        0x6806806806806806806806806806807,
        -16,
    ),
    // -1 / 6! ≈
    // -1.3888888888888888888888888888888888888888888888888888888888888888888888888889e-3
    BigFloat::new(
        -0x5b05b05b05b05b05b05b05b05b05b05b,
        0x5b05b05b05b05b05b05b05b05b05b06,
        -10,
    ),
    // 1 / 4! ≈
    // 4.1666666666666666666666666666666666666666666666666666666666666666666666666666e-2
    BigFloat::new(
        0x55555555555555555555555555555555,
        0x55555555555555555555555555555555,
        -5,
    ),
    // -1 / 2! ≈
    // -0.5
    BigFloat::new(
        -0x40000000000000000000000000000000,
        0x00000000000000000000000000000000,
        -1,
    ),
    // 1 / 0! = 1
    BigFloat::ONE,
];

// Cut-off for small values
// 5.8774717541114375398436826861112283890933277838604376075437585313920862972735e-39
const SMALL_CUT_OFF: BigFloat = BigFloat::new(
    0x7fffffffffffffffffffffffffffffff,
    0xffffffffffffffffffffffffffffffff,
    -128,
);

pub(crate) fn approx_cos(x: &BigFloat) -> BigFloat {
    // debug_assert!(x.abs() < FRAC_PI_2);
    // If x is zero or very small, cosine x == 1.
    // if x.abs() <= SMALL_CUT_OFF {
    if x.is_zero() {
        return BigFloat::ONE;
    };
    let x2 = *x * x;
    debug_assert!(
        x2.signif.hi.leading_zeros() == 1,
        "Non-normalzed x^2: {:?}",
        x2
    );
    let mut res = COEFFS[0];
    for coeff in &COEFFS[1..N] {
        debug_assert!(
            coeff.signif.hi.leading_zeros() == 1,
            "Non-normalzed coeff: {:?}",
            coeff
        );
        res.imul_add(&x2, coeff);
    }
    res
}

#[cfg(test)]
mod test_approx_cos {
    use std::ops::Neg;

    use super::*;

    // FRAC_PI_3 = ◯₂₅₅(⅓π) =
    const FRAC_PI_3: BigFloat = BigFloat::new(
        0x430548e0b5cd961196eccb83d59eb445,
        0xb8561a02d8cd4426ab593f8cbe5bde61,
        0,
    );
    // FRAC_1_SQRT_2 = ◯₂₅₅(1/√2) =
    const FRAC_1_SQRT_2: BigFloat = BigFloat::new(
        0x5a827999fcef32422cbec4d9baa55f4f,
        0x8eb7b05d449dd426768bd642c199cc8b,
        -1,
    );

    #[test]
    fn test_approx_cos() {
        let x = FRAC_PI_3;
        // assert_eq!(approx_cos(&x), BigFloat::ONE_HALF);
        assert!(
            (approx_cos(&x) - &BigFloat::ONE_HALF).abs()
                <= BigFloat::ONE_HALF.quantum()
        );
        let x = BigFloat::FRAC_PI_4.neg();
        assert_eq!(approx_cos(&x), FRAC_1_SQRT_2);
        let x = BigFloat::ZERO;
        assert_eq!(approx_cos(&x), BigFloat::ONE);
        let x = BigFloat::ZERO.neg();
        assert_eq!(approx_cos(&x), BigFloat::ONE);
    }

    #[test]
    fn test_small_cutoff() {
        let mut f = SMALL_CUT_OFF;
        println!("{:?}\n{:?}\n", f, approx_cos(&f));
        assert_eq!(approx_cos(&f), BigFloat::ONE);
        f += &f.quantum();
        println!("{:?}\n{:?}\n", f, approx_cos(&f));
        assert_ne!(approx_cos(&f), BigFloat::ONE);
        assert_ne!(approx_cos(&f.neg()), BigFloat::ONE);
    }

    #[test]
    fn calc_small_cutoff() {
        let mut lf = BigFloat::new(
            0x571cbec554b60dbbd5f64baf0506840d,
            0x451db70d5904029b0aa6cf6bb1066de9,
            -128,
        );
        let mut uf = BigFloat::new(
            0x571cbec554b60dbbd5f64baf0506840d,
            0x451db70d5904029b0aa6cf6bb1066de9,
            -127,
        );
        assert_eq!(approx_cos(&lf), BigFloat::ONE);
        assert_ne!(approx_cos(&uf), BigFloat::ONE);
        let mut f = &(lf + &uf) >> 1_u32;
        while lf < f && f < uf {
            if approx_cos(&f) == BigFloat::ONE {
                lf = f;
            } else {
                uf = f;
            }
            f = &(lf + &uf) >> 1_u32;
        }
        println!("\n{lf:?}\n{:?}", approx_cos(&lf));
        println!("\n{f:?}\n{:?}", approx_cos(&f));
        println!("\n{uf:?}\n{:?}", approx_cos(&uf));
    }
}
