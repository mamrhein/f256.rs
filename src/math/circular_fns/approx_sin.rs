// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::FP492;

const N: usize = 31;

const COEFFS: [FP492; N] = [
    // 1 / 61! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000003d3cadac93eed0b7e79d49,
        0x738b50dba9c371c58446cb0fbc380ff1,
    ),
    // -1 / 59! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
        0xfffffffc94807d00ad05affabca349e0,
        0x1437fb80e9c16c20dbe0db090e7c14a2,
    ),
    // 1 / 57! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00002db6f27910f72df9a65a714a7eb1,
        0xbbac1acb587ca0c4d8501cf0616436ed,
    ),
    // -1 / 55! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
        0xfdc5fee0aa7475fabf2dc84b672447e3,
        0xf611e888ae0b6996d9172ac1a67326cd,
    ),
    // 1 / 53! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000019,
        0xd4f1058674df40f206da45356515f54b,
        0x343c3e4ccf98ffed6939f55aec0fd65d,
    ),
    // -1 / 51! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xfffffffffffffffffffffffffffee9e7,
        0x8d40847dcc88d26e3a2eed2bbf9b4261,
        0xa7714d1514dcc81f2c0a992aa5803c36,
    ),
    // 1 / 49! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x0000000000000000000000000ad21786,
        0xff5842eca51fea0870918e396b78c746,
        0x1d6a300230ce997f4e6e513374a83c91,
    ),
    // -1 / 47! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffff9c95c7d7b6,
        0x051921d2eac9d275c6b550749a592bd1,
        0xc066ebdf95ddbe5f6a75f74036538f18,
    ),
    // 1 / 45! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x0000000000000000000347970e4440c8,
        0xf1c058bd238c9957d8be87407aebee95,
        0x1acbf9c0554e3e2ee7bde3b532518708,
    ),
    // -1 / 43! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffe6a24bada81aedd2,
        0x3451a9210c8dfc8f9e61e9494736b6c4,
        0xbe60546c36d70d379f52d28ed16fa053,
    ),
    // 1 / 41! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000b2f30e1ce812063f12,
        0xe7e8d8d96e5442d0a9443d0b9c02a008,
        0xf46c6c951ee0c19a05b694767e82f68d,
    ),
    // -1 / 39! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xfffffffffffb859aed96d14c87fbeee2,
        0x5452cf153433f743a2d8eda08f2ec6a2,
        0x296864b23027bd4b6658e8e589048ab1,
    ),
    // 1 / 37! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x0000000019ec8d1c94e85af4c78b15c3,
        0xd89d2f3fcb2a927344305c831b36193c,
        0x49a9107539f22981814bbf34cbb51add,
    ),
    // -1 / 35! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffff791d31c7493706be61c052c0fc,
        0xee262812e67e0041345ea5d66a84b250,
        0xbc565e0e80000a2b41e1214c15b03937,
    ),
    // 1 / 33! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x000273024a9ba1aa36a7059bff52e844,
        0xfaa1b824924ad0e690091d4cdb2b1cb4,
        0x8686c898ffd0baebc37f38532ed60a32,
    ),
    // -1 / 31! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xf5e5968c3e0521de8f08dc82ca01e376,
        0x24e869248b6248edda6722f7ee299755,
        0x140488e0c2fcf3799337a8decd15f05f,
    ),
    // 1 / 29! ≈
    FP492::new(
        0x00000000000000000000000000000024,
        0xb3f31686b15af57c61ceecde2523accd,
        0xebb2093da4f30fec9552f750cce83ced,
        0x4786bf7ba51380572fcc869af64cc695,
    ),
    // -1 / 27! ≈
    FP492::new(
        0xffffffffffffffffffffffffffff8b95,
        0x38f48cc5737d5979c3a8af6232d7e2d8,
        0x674ab078cd097d965cd78bb60f5ebf61,
        0x2098a3d06624eb7463450c7ac47a1f82,
    ),
    // 1 / 25! ≈
    FP492::new(
        0x000000000000000000000000013f3ccd,
        0xd165fa8d4e44a419776f10b893fff294,
        0xc13014bdbff99dad68eee2c1da2f33a8,
        0x9d6eca87e6c256d7c8a7c759391d9876,
    ),
    // -1 / 23! ≈
    FP492::new(
        0xfffffffffffffffffffffffd13c97d9d,
        0x38fcc4d08f1f645013b0cf65201f735b,
        0x374f63460ef67192101c89a8a15eeccf,
        0x0455517b28847641b6c4c6e222a2ac38,
    ),
    // 1 / 21! ≈
    FP492::new(
        0x0000000000000000000005c6e3bdb73d,
        0x5c62fbc51bf3b9b91486121e81d5fdb4,
        0xad15c7866ce3854c2797e8b10a5feed1,
        0x6f5cf291ea2e421cbf0ee5078a7799a4,
    ),
    // -1 / 19! ≈
    FP492::new(
        0xfffffffffffffffffff685b25cbf5f54,
        0x6d9af09e24234c5a540a45f2fcebc394,
        0x0844a7755ab94f0f0ac63d8afa9c3065,
        0x4b82089bcc1b88d68b903fa0d3c7ef66,
    ),
    // 1 / 17! ≈
    FP492::new(
        0x00000000000000000ca963b81856a535,
        0x93028cbbb8d7ff53ba468d621d08b83c,
        0xf4484938cc7061e79b29c855335758ad,
        0x20487fdd533731618d4aff2512e62d15,
    ),
    // -1 / 15! ≈
    FP492::new(
        0xfffffffffffffff28c060c6623f07713,
        0xcd4a788b9a80b70a1509c7c126bc3f3c,
        0x733233a6c897f9eb239b25797331c80d,
        0xb2f824d7955b8859e050e89beb7019fb,
    ),
    // 1 / 13! ≈
    FP492::new(
        0x0000000000000b092309d43684be51c1,
        0x98e91d7b4269d9babdfa238e39942069,
        0x80d1a12f7354fd1ccabb425f8129e4c3,
        0x3071c7277aea2645fda13018de0ab03a,
    ),
    // -1 / 11! ≈
    FP492::new(
        0xfffffffffff9466ea602aec71c062e06,
        0xd1f208e3877f52343b925554e9bc3fb5,
        0x8041c715b835c27475e38dcd4a78990e,
        0x7aaaa3f11950ad5971c6b0d8b17c9cb4,
    ),
    // 1 / 9! ≈
    FP492::new(
        0x0000000002e3bc74aad8e671f5583911,
        0xca002e3bc74aad8e671f5583911ca002,
        0xe3bc74aad8e671f5583911ca002e3bc7,
        0x4aad8e671f5583911ca002e3bc74aad9,
    ),
    // -1 / 7! ≈
    FP492::new(
        0xffffffff2ff2ff2ff2ff2ff2ff2ff2ff,
        0x2ff2ff2ff2ff2ff2ff2ff2ff2ff2ff2f,
        0xf2ff2ff2ff2ff2ff2ff2ff2ff2ff2ff2,
        0xff2ff2ff2ff2ff2ff2ff2ff2ff2ff2ff,
    ),
    // 1 / 5! ≈
    FP492::new(
        0x00000022222222222222222222222222,
        0x22222222222222222222222222222222,
        0x22222222222222222222222222222222,
        0x22222222222222222222222222222222,
    ),
    // -1 / 3! ≈
    FP492::new(
        0xfffffd55555555555555555555555555,
        0x55555555555555555555555555555555,
        0x55555555555555555555555555555555,
        0x55555555555555555555555555555555,
    ),
    // 1 / 1! ≈
    FP492::new(
        0x00001000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
];

// 4.34011192927290911129289122290072654677561270925175391536078950032040224e-36
// 4.34011192927290911129289122290072654677561270925175391536078950032040224e-36
const SMALL_CUT_OFF: FP492 = FP492::new(
    0x00000000000000000000000000000000,
    0x005c4dd12448fbd973779fc5b15b8c99,
    0xf02871b5584367dcb9d6fe24cc9ce800,
    0x00000000000000000000000000000000,
);

pub(crate) fn approx_sin(x: &FP492) -> FP492 {
    let mut x_abs = *x;
    x_abs.iabs();
    // If x is zero or very small, sine x == x.
    if x_abs <= SMALL_CUT_OFF {
        return *x;
    };
    let mut x2 = x_abs;
    x2.imul_round(&x_abs);
    let mut sin = COEFFS[0];
    for coeff in &COEFFS[1..N] {
        sin.imul_round(&x2);
        sin += coeff;
    }
    sin.imul_round(x);
    sin
}

#[cfg(test)]
mod test_approx_sin {
    use super::*;
    use crate::{f256, math::Float256};

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
        let cutoff = FP492::from(&f);
        // println!("\n{lf:?}\n{:?}", lf.sin());
        // println!("\n{f:?}\n{:?}", f.sin());
        // println!("\n{uf:?}\n{:?}", uf.sin());
        // println!("\n// {f:e}");
        // println!("{:?};", cutoff);

        assert_eq!(f, f.sin());
        // assert_eq!(cutoff, SMALL_CUT_OFF);
        let g = f + f.ulp();
        assert_ne!(g, g.sin());
    }
}
