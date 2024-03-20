// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::fp509::FP509;

const N: usize = 31;

const COEFFS: [FP509; N] = [
    // 1 / 61! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x0000007a795b5927dda16fcf3a92e716,
        0xa1b75386e38b088d961f78701fe2413c,
    ),
    // -1 / 59! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
        0xfff92900fa015a0b5ff5794693c0286f,
        0xf701d382d841b7c1b6121cf82943571a,
    ),
    // 1 / 57! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x5b6de4f221ee5bf34cb4e294fd637758,
        0x3596b0f94189b0a039e0c2c86dd9b229,
    ),
    // -1 / 55! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xfffffffffffffffffffffffffffffb8b,
        0xfdc154e8ebf57e5b9096ce488fc7ec23,
        0xd1115c16d32db22e55834ce64d9a9251,
    ),
    // 1 / 53! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x0000000000000000000000000033a9e2,
        0x0b0ce9be81e40db48a6aca2bea966878,
        0x7c999f31ffdad273eab5d81facba7af5,
    ),
    // -1 / 51! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xfffffffffffffffffffffffdd3cf1a81,
        0x08fb9911a4dc745dda577f3684c34ee2,
        0x9a2a29b9903e581532554b00786c4f62,
    ),
    // 1 / 49! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x0000000000000000000015a42f0dfeb0,
        0x85d94a3fd410e1231c72d6f18e8c3ad4,
        0x6004619d32fe9cdca266e95079214923,
    ),
    // -1 / 47! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffff392b8faf6c0a32,
        0x43a5d593a4eb8d6aa0e934b257a380cd,
        0xd7bf2bbb7cbed4ebee806ca71e30111e,
    ),
    // 1 / 45! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000000068f2e1c888191e380,
        0xb17a471932afb17d0e80f5d7dd2a3597,
        0xf380aa9c7c5dcf7bc76a64a30e0f6e6b,
    ),
    // -1 / 43! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffcd44975b5035dba468a3,
        0x5242191bf91f3cc3d2928e6d6d897cc0,
        0xa8d86dae1a6f3ea5a51da2df40a5f991,
    ),
    // 1 / 41! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x000000000165e61c39d0240c7e25cfd1,
        0xb1b2dca885a152887a1738054011e8d8,
        0xd92a3dc183340b6d28ecfd05ed196251,
    ),
    // -1 / 39! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xfffffff70b35db2da2990ff7ddc4a8a5,
        0x9e2a6867ee8745b1db411e5d8d4452d0,
        0xc964604f7a96ccb1d1cb1209156229be,
    ),
    // 1 / 37! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x000033d91a3929d0b5e98f162b87b13a,
        0x5e7f965524e68860b906366c32789352,
        0x20ea73e4530302977e69976a35ba5803,
    ),
    // -1 / 35! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xfef23a638e926e0d7cc380a581f9dc4c,
        0x5025ccfc008268bd4bacd50964a178ac,
        0xbc1d0000145683c242982b60726e0f58,
    ),
    // 1 / 33! ≈
    FP509::new(
        0x00000000000000000000000000000004,
        0xe604953743546d4e0b37fea5d089f543,
        0x70492495a1cd20123a99b65639690d0d,
        0x9131ffa175d786fe70a65dac1464ab1d,
    ),
    // -1 / 31! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffebcb,
        0x2d187c0a43bd1e11b9059403c6ec49d0,
        0xd24916c491dbb4ce45efdc532eaa2809,
        0x11c185f9e6f3266f51bd9a2be0be2a69,
    ),
    // 1 / 29! ≈
    FP509::new(
        0x000000000000000000000000004967e6,
        0x2d0d62b5eaf8c39dd9bc4a47599bd764,
        0x127b49e61fd92aa5eea199d079da8f0d,
        0x7ef74a2700ae5f990d35ec998d29ecd7,
    ),
    // -1 / 27! ≈
    FP509::new(
        0xffffffffffffffffffffffff172a71e9,
        0x198ae6fab2f387515ec465afc5b0ce95,
        0x60f19a12fb2cb9af176c1ebd7ec24131,
        0x47a0cc49d6e8c68a18f588f43f04c52a,
    ),
    // 1 / 25! ≈
    FP509::new(
        0x00000000000000000000027e799ba2cb,
        0xf51a9c894832eede217127ffe5298260,
        0x297b7ff33b5ad1ddc583b45e67513add,
        0x950fcd84adaf914f8eb2723b30eb5579,
    ),
    // -1 / 23! ≈
    FP509::new(
        0xfffffffffffffffffffa2792fb3a71f9,
        0x89a11e3ec8a027619eca403ee6b66e9e,
        0xc68c1dece3242039135142bdd99e08aa,
        0xa2f65108ec836d898dc44545586fab4a,
    ),
    // 1 / 21! ≈
    FP509::new(
        0x00000000000000000b8dc77b6e7ab8c5,
        0xf78a37e77372290c243d03abfb695a2b,
        0x8f0cd9c70a984f2fd16214bfdda2deb9,
        0xe523d45c84397e1dca0f14ef33476f8e,
    ),
    // -1 / 19! ≈
    FP509::new(
        0xffffffffffffffed0b64b97ebea8db35,
        0xe13c484698b4a8148be5f9d787281089,
        0x4eeab5729e1e158c7b15f53860ca9704,
        0x1137983711ad17207f41a78fdeccfa6c,
    ),
    // 1 / 17! ≈
    FP509::new(
        0x0000000000001952c77030ad4a6b2605,
        0x197771affea7748d1ac43a117079e890,
        0x927198e0c3cf365390aa66aeb15a4090,
        0xffbaa66e62c31a95fe4a25cc5a29731c,
    ),
    // -1 / 15! ≈
    FP509::new(
        0xffffffffffe5180c18cc47e0ee279a94,
        0xf11735016e142a138f824d787e78e664,
        0x674d912ff3d647364af2e663901b65f0,
        0x49af2ab710b3c0a1d137d6e033f5b229,
    ),
    // 1 / 13! ≈
    FP509::new(
        0x0000000016124613a86d097ca38331d2,
        0x3af684d3b3757bf4471c732840d301a3,
        0x425ee6a9fa39957684bf0253c98660e3,
        0x8e4ef5d44c8bfb426031bc156073da13,
    ),
    // -1 / 11! ≈
    FP509::new(
        0xfffffff28cdd4c055d8e380c5c0da3e4,
        0x11c70efea4687724aaa9d3787f6b0083,
        0x8e2b706b84e8ebc71b9a94f1321cf555,
        0x47e232a15ab2e38d61b162f939671c64,
    ),
    // 1 / 9! ≈
    FP509::new(
        0x000005c778e955b1cce3eab072239400,
        0x5c778e955b1cce3eab0722394005c778,
        0xe955b1cce3eab0722394005c778e955b,
        0x1cce3eab0722394005c778e955b1cce4,
    ),
    // -1 / 7! ≈
    FP509::new(
        0xfffe5fe5fe5fe5fe5fe5fe5fe5fe5fe5,
        0xfe5fe5fe5fe5fe5fe5fe5fe5fe5fe5fe,
        0x5fe5fe5fe5fe5fe5fe5fe5fe5fe5fe5f,
        0xe5fe5fe5fe5fe5fe5fe5fe5fe5fe5fe6,
    ),
    // 1 / 5! ≈
    FP509::new(
        0x00444444444444444444444444444444,
        0x44444444444444444444444444444444,
        0x44444444444444444444444444444444,
        0x44444444444444444444444444444444,
    ),
    // -1 / 3! ≈
    FP509::new(
        0xfaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab,
    ),
    // 1 / 1! ≈
    FP509::new(
        0x20000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
];

// 4.34011192927290911129289122290072654677560440564151412595797421911000298e-36
const SMALL_CUT_OFF: FP509 = FP509::new(
    0x000000000000000000000000000000b8,
    0x9ba24891f7b2e6ef3f8b62b71933e050,
    0xc4a624dd04ec913a3e682736b0000000,
    0x00000000000000000000000000000000,
);

pub(crate) fn approx_sin(x: &FP509) -> FP509 {
    // debug_assert!(x.abs() < FP510::FRAC_PI_2);
    // If x is zero or very small, sine x == x.
    if x <= &SMALL_CUT_OFF {
        return *x;
    };
    let mut x2 = *x;
    x2.imul_round(x);
    let mut sin = COEFFS[0];
    for coeff in &COEFFS[1..N] {
        sin.imul_round(&x2);
        sin += &coeff;
    }
    sin.imul_round(x);
    sin
}

#[cfg(test)]
mod test_approx_sin {
    use crate::f256;

    #[test]
    fn calc_small_cutoff() {
        let mut lf = f256::from(1e-36_f64);
        let mut uf = f256::from(1e-35_f64);
        assert_eq!(lf, lf.sin());
        assert_ne!(uf, uf.sin());
        let mut f = (lf + uf) / f256::TWO;
        while lf < f && f < uf {
            if f == f.sin() {
                lf = f;
            } else {
                uf = f;
            }
            f = (lf + uf) / f256::TWO;
        }
        // println!("\n{lf:?}\n{:?}", lf.sin());
        // println!("\n{f:?}\n{:?}", f.sin());
        // println!("\n{uf:?}\n{:?}", uf.sin());
        // println!("\n// {f:e}");
        // println!("{:?};", FP509::from(&BigFloat::from(&f)));

        assert_eq!(f, f.sin());
        let g = f + f.ulp();
        assert_ne!(g, g.sin());
    }
}
