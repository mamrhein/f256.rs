// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::math::BigFloat;

const N: usize = 31;

const COEFFS: [BigFloat; N] = [
    // 1 / 61! ≈
    // 1.97013195680216831183503912158338127299673461605266947044548580437658077375233e-84
    BigFloat::new(
        0x7a795b5927dda16fcf3a92e716a1b753,
        0x86e38b088d961f78701fe2413c37415b,
        -279,
    ),
    // -1 / 59! ≈
    // -7.2106829618959360213162431849951754591680486947527702618304780440182856319335e-81
    BigFloat::new(
        -0x6d6ff05fea5f4a00a86b96c3fd79008f,
        0xe2c7d27be483e49ede307d6bca8e5fa6,
        -267,
    ),
    // 1 / 57! ≈
    // 2.46749570956078930649441841790534904212730626334439798359838958666305734324766e-77
    BigFloat::new(
        0x5b6de4f221ee5bf34cb4e294fd637758,
        0x3596b0f94189b0a039e0c2c86dd9b229,
        -255,
    ),
    // -1 / 55! ≈
    // -7.8762463049180394663301835899538741424703615925953183636460595606284790396465e-74
    BigFloat::new(
        -0x474023eab17140a81a46f6931b770381,
        0x3dc2eeea3e92cd24dd1aa7cb319b2657,
        -243,
    ),
    // 1 / 53! ≈
    // 2.339245152560657721500064526216300620313697393000809554002879689506658274775e-70
    BigFloat::new(
        0x6753c41619d37d03c81b6914d59457d5,
        0x2cd0f0f9333e63ffb5a4e7d56bb03f59,
        -232,
    ),
    // -1 / 51! ≈
    // -6.44695964045717268045417783425212450958455001511023113083193642428035020528e-67
    BigFloat::new(
        -0x45861cafdee08cddcb64717444b51019,
        0x2f679623acbabac8cdf834fd59b556a0,
        -220,
    ),
    // 1 / 49! ≈
    // 1.6439747083165790335158153477342917499440602538531089383621437881914893023464e-63
    BigFloat::new(
        0x5690bc37fac2176528ff5043848c71cb,
        0x5bc63a30eb5180118674cbfa7372899c,
        -209,
    ),
    // -1 / 47! ≈
    // -3.8666285139605938868291976978710541958684297170625122230277621898263828391187e-60
    BigFloat::new(
        -0x636a382849fae6de2d15362d8a394aaf,
        0x8b65a6d42e3f9914206a2241a0958a09,
        -198,
    ),
    // 1 / 45! ≈
    // 8.3596508471828039833247254227972191714675450482891514261860218544046396981747e-57
    BigFloat::new(
        0x68f2e1c888191e380b17a471932afb17,
        0xd0e80f5d7dd2a3597f380aa9c7c5dcf8,
        -187,
    ),
    // -1 / 43! ≈
    // -1.6552108677421951886982956337138493959505739195612519823848323271721186602386e-53
    BigFloat::new(
        -0x6576d1495f9448b72eb95b7bcdc80dc1,
        0x86785adae32524ed067eae4f24a3cb22,
        -176,
    ),
    // 1 / 41! ≈
    // 2.9893108271424045107891219144872120090867364987276210801870071828728463003909e-50
    BigFloat::new(
        0x5979870e7409031f8973f46c6cb72a21,
        0x6854a21e85ce0150047a36364a8f7061,
        -165,
    ),
    // -1 / 39! ≈
    // -4.9024697565135433976941599397590276949022478579132985715066917799114679326411e-47
    BigFloat::new(
        -0x47a6512692eb37804111dabad30eacbc,
        0xc08bc5d27125f70d1395dd6979b4dcfe,
        -154,
    ),
    // 1 / 37! ≈
    // 7.265460179153071315382745030722879043845131325427508482972917217828795476174e-44
    BigFloat::new(
        0x67b2347253a16bd31e2c570f6274bcff,
        0x2caa49cd10c1720c6cd864f126a441d5,
        -144,
    ),
    // -1 / 35! ≈
    // -9.6775929586318909920898163809228748864017149254694412993199257341479555742638e-41
    BigFloat::new(
        -0x4371671c5b647ca0cf1fd69f8188eceb,
        0xf68cc0ffdf65d0ad14cabda6d7a1d4d1,
        -133,
    ),
    // 1 / 33! ≈
    // 1.1516335620771950280586881493298221114818040761308635146190711623636067133374e-37
    BigFloat::new(
        0x4e604953743546d4e0b37fea5d089f54,
        0x370492495a1cd20123a99b65639690d1,
        -123,
    ),
    // -1 / 31! ≈
    // -1.21612504155351794962997468569229214972478510439419187143773914745596868928427e-34
    BigFloat::new(
        -0x50d34b9e0fd6f10b87b91be9aff0e44e,
        0xd8bcb6dba4edb8912cc6e8408eb34557,
        -113,
    ),
    // 1 / 29! ≈
    // 1.1309962886447716931558764576938316992440501470865984404370974071340508810344e-31
    BigFloat::new(
        0x4967e62d0d62b5eaf8c39dd9bc4a4759,
        0x9bd764127b49e61fd92aa5eea199d07a,
        -103,
    ),
    // -1 / 27! ≈
    // -9.1836898637955461484257168364739133978616871943431793363492309459284931539991e-29
    BigFloat::new(
        -0x746ac70b733a8c82a6863c57509dcd28,
        0x1d2798b54f8732f68269a3287449f0a1,
        -94,
    ),
    // 1 / 25! ≈
    // 6.4469502843844733961948532192046872052989044104289118941171601240418021941075e-26
    BigFloat::new(
        0x4fcf3374597ea3539129065ddbc42e24,
        0xfffca5304c052f6ffe676b5a3bb8b077,
        -84,
    ),
    // -1 / 23! ≈
    // -3.8681701706306840377169119315228123231793426462573471364702960744250813164645e-23
    BigFloat::new(
        -0x5d86d04c58e06765ee1c1375fd89e613,
        0x5bfc1194991613973e2131cdbdfc6ecb,
        -75,
    ),
    // 1 / 21! ≈
    // 1.95729410633912612308475743735054303552874737900621765105396981365909114613103e-20
    BigFloat::new(
        0x5c6e3bdb73d5c62fbc51bf3b9b914861,
        0x21e81d5fdb4ad15c7866ce3854c2797f,
        -66,
    ),
    // -1 / 19! ≈
    // -8.2206352466243297169559812368722807492207389918261141344266732173681828137503e-18
    BigFloat::new(
        -0x4bd26d1a05055c93287b0edee59d2d5f,
        0xadd06818a1e35fbddac4552a358787aa,
        -57,
    ),
    // 1 / 17! ≈
    // 2.8114572543455207631989455830103200162334927352045310339739222403399185223026e-15
    BigFloat::new(
        0x654b1dc0c2b529ac981465ddc6bffa9d,
        0xd2346b10e845c1e7a24249c663830f3d,
        -49,
    ),
    // -1 / 15! ≈
    // -7.6471637318198164759011319857880704441551002397563244124090684937245783806631e-13
    BigFloat::new(
        -0x6b9fcf9ccee07c476195ac3ba32bfa47,
        0xaf57b1c1f6ca1e061c666e62c9bb4031,
        -41,
    ),
    // 1 / 13! ≈
    // 1.60590438368216145993923771701549479327257105034882812660590438368216145993924e-10
    BigFloat::new(
        0x5849184ea1b425f28e0cc748ebda134e,
        0xcdd5efd11c71cca1034c068d097b9aa8,
        -33,
    ),
    // -1 / 11! ≈
    // -2.50521083854417187750521083854417187750521083854417187750521083854417187750523e-8
    BigFloat::new(
        -0x6b99159fd5138e3f9d1f92e0df71c788,
        0xadcbc46daaab1643c04a7fbe38ea47d,
        -26,
    ),
    // 1 / 9! ≈
    // 2.7557319223985890652557319223985890652557319223985890652557319223985890652557e-6
    BigFloat::new(
        0x5c778e955b1cce3eab0722394005c778,
        0xe955b1cce3eab0722394005c778e955b,
        -19,
    ),
    // -1 / 7! ≈
    // -1.98412698412698412698412698412698412698412698412698412698412698412698412698415e-4
    BigFloat::new(
        -0x68068068068068068068068068068068,
        0x6806806806806806806806806806807,
        -13,
    ),
    // 1 / 5! ≈
    // 8.3333333333333333333333333333333333333333333333333333333333333333333333333333e-3
    BigFloat::new(
        0x44444444444444444444444444444444,
        0x44444444444444444444444444444444,
        -7,
    ),
    // -1 / 3! ≈
    // -0.166666666666666666666666666666666666666666666666666666666666666666666666666665
    BigFloat::new(
        -0x55555555555555555555555555555555,
        0x55555555555555555555555555555555,
        -3,
    ),
    // 1 / 1! = 1
    BigFloat::ONE,
];

// Cut-off for small values
// 1.1754943508222875079687365372222456778186655567720875215087517062784172594547e-38
const SMALL_CUT_OFF: BigFloat = BigFloat::new(
    0x7fffffffffffffffffffffffffffffff,
    0xffffffffffffffffffffffffffffffff,
    -127,
);

pub(crate) fn approx_sin(x: &BigFloat) -> BigFloat {
    debug_assert!(x.abs() < BigFloat::FRAC_PI_2);
    // If x is zero or very small, sine x == x.
    // if x.abs() <= SMALL_CUT_OFF {
    if x.is_zero() {
        return *x;
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
    res *= x;
    res
}

#[cfg(test)]
mod test_approx_sin {
    use std::ops::Neg;

    use super::*;

    const FRAC_PI_6: BigFloat = BigFloat::new(
        0x430548e0b5cd961196eccb83d59eb445,
        0xb8561a02d8cd4426ab593f8cbe5bde61,
        -1,
    );
    const FRAC_1_SQRT_2: BigFloat = BigFloat::new(
        0x5a827999fcef32422cbec4d9baa55f4f,
        0x8eb7b05d449dd426768bd642c199cc8b,
        -1,
    );

    #[test]
    fn test_approx_sin() {
        let x = FRAC_PI_6;
        assert_eq!(approx_sin(&x), BigFloat::ONE_HALF);
        assert_eq!(approx_sin(&x.neg()), BigFloat::ONE_HALF.neg());
        let x = BigFloat::FRAC_PI_4;
        assert_eq!(approx_sin(&x), FRAC_1_SQRT_2);
        let x = BigFloat::ZERO;
        assert_eq!(approx_sin(&x), BigFloat::ZERO);
        let x = BigFloat::ZERO.neg();
        assert_eq!(approx_sin(&x), BigFloat::ZERO);
    }

    #[test]
    fn test_small_cutoff() {
        let mut f = SMALL_CUT_OFF;
        // println!("{:?}\n{:?}\n", f, approx_sin(&f));
        assert_eq!(approx_sin(&f), f);
        f += &f.quantum();
        // println!("{:?}\n{:?}\n", f, approx_sin(&f));
        assert_ne!(approx_sin(&f), f);
        assert_ne!(approx_sin(&f.neg()), -f);
    }

    #[test]
    fn calc_small_cutoff() {
        let mut lf = BigFloat::new(
            0x5c4dd12448fbd973779fc5b15b8c99f0,
            0x2862530abd5bb3667d15d0a1f4814bff,
            -127,
        );
        let mut uf = BigFloat::new(
            0x5c4dd12448fbd973779fc5b15b8c99f0,
            0x2862530abd5bb3667d15d0a1f4814bff,
            -125,
        );
        assert_eq!(approx_sin(&lf), lf);
        assert_ne!(approx_sin(&uf), uf);
        let mut f = &(lf + &uf) >> 1_u32;
        while lf < f && f < uf {
            if approx_sin(&f) == f {
                lf = f;
            } else {
                uf = f;
            }
            f = &(lf + &uf) >> 1_u32;
        }
        println!("\n{lf:?}\n{:?}", approx_sin(&lf));
        println!("\n{f:?}\n{:?}", approx_sin(&f));
        println!("\n{uf:?}\n{:?}", approx_sin(&uf));
    }
}
