// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::FP492;

const N: usize = 33;

const COEFFS: [FP492; N] = [
    // 1 / 64! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000100dcf6a320e1bb718,
        0x2616aa05d9aff3e0eb5c48f404790c83,
    ),
    // -1 / 62! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffff03267d376ba1cb7c43a8,
        0x1b0a23db6cbee9851282fcb98d7af77c,
    ),
    // 1 / 60! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x000000000e9775621f3fe7bbd2307a80,
        0x8832445773921c1084de62bfd95bcc75,
    ),
    // -1 / 58! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
        0xffffff36399ccf27e04f8ec979a206a4,
        0xa8e6f6b5df93eb92acd27b165698c143,
    ),
    // 1 / 56! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x000a2dbbfcf4c7093c960a2339963592,
        0xc951f746b3bfcbd429d67185af503abc,
    ),
    // -1 / 54! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
        0x8589c2449f0558df12d6083328cb71f9,
        0xddd8f55d6473af68a3fa2f9ac2bd561b,
    ),
    // 1 / 52! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000559,
        0x15e624d63238721b6b30540ded8bc891,
        0xd078e5e6faacfc26c8ffcbd2df47614e,
    ),
    // -1 / 50! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffc89921,
        0x23da650fbf41ebf597593fb72bee3974,
        0x5b925b3327fbde35c61c837ef88bfeca,
    ),
    // 1 / 48! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x000000000000000000000002123680d6,
        0xdfe4cf4b9b1bcb9d8bdc38fd921e246b,
        0xa153306b578b615e031d8ad9543397af,
    ),
    // -1 / 46! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffedbf7fb09a6a,
        0xef9d35b91b0da39f7b49c568565f0b82,
        0x52e54e0c83b5f3848ba864c9f957456a,
    ),
    // 1 / 44! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000000000000000093958d81ff6352,
        0x7ecf993f3fb6f471197dc6559b78f035,
        0xb5dae6cefec0ee3ebc6106d9d854bc5b,
    ),
    // -1 / 42! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xfffffffffffffffbbd42b62b3c85f24e,
        0xc9b7688d1bd96c1f9a722f4ef630b30b,
        0xfa2e2e2d361f3857c2e95dfd2dbfedf0,
    ),
    // 1 / 40! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x0000000000001ca8ed42a12ae3001a07,
        0x244abad2ab7eb36b1bedc6dbfc6ba16f,
        0x255d63e1f1ff01aaea3dc6fa42f97c88,
    ),
    // -1 / 38! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffff515a9a31f9e2a8b761647a,
        0xd89d8c3af3eaab4dcf0c3375d02042b4,
        0x4ee75725560dd67c978b7af7dfb120fa,
    ),
    // 1 / 36! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x00000003bf30652185952560d71a254e,
        0x4eb7d4385d272aa8dafd5ef2eed1a5b6,
        0xa56f60f15fffffb7aff2a2a1712ce1f7,
    ),
    // -1 / 34! ≈
    FP492::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffed8efdce3f0285ec075d4b506294,
        0x8f377a95833a08ea28f0ac5090246109,
        0xbfcedbfb800163ea01c78d66f717d286,
    ),
    // 1 / 32! ≈
    FP492::new(
        0x00000000000000000000000000000000,
        0x0050d34b9e0fd6f10b87b91be9aff0e4,
        0x4ed8bcb6dba4edb8912cc6e8408eb345,
        0x575fdbb8f9e81864336642b90997507d,
    ),
    // -1 / 30! ≈
    FP492::new(
        0xfffffffffffffffffffffffffffffffe,
        0xc6cd3afb829f19f35212b3d6763a8b4e,
        0x7824bb6ce0e6d4cd727d3c05d709534d,
        0x6c8c93379ca17bb8d3bd72fad5a81b84,
    ),
    // 1 / 28! ≈
    FP492::new(
        0x00000000000000000000000000000428,
        0x62898d42174dcf171470d52a350a9353,
        0xb32b0bfbaf88cdccea660427364ee6e1,
        0x1a43b101b33589e06a2b3f8de6b27ee0,
    ),
    // -1 / 26! ≈
    FP492::new(
        0xfffffffffffffffffffffffffff3b8bd,
        0x01cad8d32e386fd7a2ca7f5b5cc4ecd2,
        0xe4e09cbda0003edbcabbbc339efe2f3e,
        0x701946fac5e4d546784850f2b8e152c0,
    ),
    // 1 / 24! ≈
    FP492::new(
        0x0000000000000000000000001f2cf019,
        0x72f577cca4b4067ca9d8a20673feb086,
        0xddb20687bf6065ef3f5424ee4e9c0b77,
        0x5fd1c74588fa7b12986277b693e3e37e,
    ),
    // -1 / 22! ≈
    FP492::new(
        0xffffffffffffffffffffffbcc71a4920,
        0x1eb5aebcdbd20331c4e2a215e2d35d31,
        0xf821eb4b5824341f72905e267f874699,
        0x63aa5210a3e69fe76badde511c9d7904,
    ),
    // 1 / 20! ≈
    FP492::new(
        0x000000000000000000007950ae900808,
        0x941ea72b4afe3c2eaeff7c80a68dcfd2,
        0x32c95e06eea9ef3f3f761685d9de972e,
        0x229fe5f835cb6c5bac38c99e5bcf9a6e,
    ),
    // -1 / 18! ≈
    FP492::new(
        0xffffffffffffffffff4bec3ce2341344,
        0x227fdbbcae9eaab43cc33108c57f83fc,
        0x9d186db5bbc0de1dccb6915099979784,
        0x9aa6a390260b27ec5bb4b8efb7d6c49b,
    ),
    // 1 / 16! ≈
    FP492::new(
        0x0000000000000000d73f9f399dc0f88e,
        0xc32b58774657f48f5eaf6383ed943c0c,
        0x38ccdcc5937680614dc64da868cce37f,
        0x24d07db286aa477a61faf1764148fe60,
    ),
    // -1 / 14! ≈
    FP492::new(
        0xffffffffffffff36345ab9fc1b16fa29,
        0x075d102e0d8ab9973b92b4514507b48a,
        0xbff106c5c0e7a4c71617321dbfeab8cd,
        0x7c8a28a1c05cfd4424bda122cb9185b3,
    ),
    // 1 / 12! ≈
    FP492::new(
        0x0000000000008f76c77fc6c4bdaa26d4,
        0xc3d67f425f600e7ba5b3ce38ec85a55b,
        0x8aa52f68db50da764b825ed98f209de9,
        0x75c71d013de3f18de12f7143468af2f1,
    ),
    // -1 / 10! ≈
    FP492::new(
        0xffffffffffb606c1221d828e3443fa4b,
        0x056661c6d278883e8f49aaa60b16bccc,
        0x82d38deeea4f5b0110c717d2332e939f,
        0x45550b5c167772d7e389994fa05abbb7,
    ),
    // 1 / 8! ≈
    FP492::new(
        0x000000001a01a01a01a01a01a01a01a0,
        0x1a01a01a01a01a01a01a01a01a01a01a,
        0x01a01a01a01a01a01a01a01a01a01a01,
        0xa01a01a01a01a01a01a01a01a01a01a0,
    ),
    // -1 / 6! ≈
    FP492::new(
        0xfffffffa4fa4fa4fa4fa4fa4fa4fa4fa,
        0x4fa4fa4fa4fa4fa4fa4fa4fa4fa4fa4f,
        0xa4fa4fa4fa4fa4fa4fa4fa4fa4fa4fa4,
        0xfa4fa4fa4fa4fa4fa4fa4fa4fa4fa4fa,
    ),
    // 1 / 4! ≈
    FP492::new(
        0x000000aaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,
        0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab,
    ),
    // -1 / 2! ≈
    FP492::new(
        0xfffff800000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
    // 1 / 0! ≈
    FP492::new(
        0x00001000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
];

// 2.12787206838507003880796293968184411684206163544728281856444725083506186e-36
pub(crate) const SMALL_CUT_OFF: FP492 = FP492::new(
    0x00000000000000000000000000000000,
    0x002d413cccfe779921165f626cdd52af,
    0xa7c75bd82ea24eea133b45eb2160cc00,
    0x00000000000000000000000000000000,
);

pub(crate) fn approx_cos(x: &FP492) -> FP492 {
    let mut x_abs = *x;
    x_abs.iabs();
    // If x is zero or very small, cosine x == 1.
    if x_abs <= SMALL_CUT_OFF {
        return FP492::ONE;
    };
    let mut x2 = x_abs;
    x2.imul_round(&x_abs);
    let mut cos = COEFFS[0];
    for coeff in &COEFFS[1..N] {
        cos.imul_round(&x2);
        cos += coeff;
    }
    cos
}

#[cfg(test)]
mod test_approx_cos {
    use super::*;
    use crate::{f256, math::BigFloat};

    #[test]
    fn calc_small_cutoff() {
        let mut lf = f256::from(1e-36_f64);
        let mut uf = f256::from(1e-35_f64);
        assert_eq!(lf.cos(), f256::ONE);
        assert_ne!(uf.cos(), f256::ONE);
        let mut f = (lf + uf) / f256::TWO;
        while lf < f && f < uf {
            if f.cos() == f256::ONE {
                lf = f;
            } else {
                uf = f;
            }
            f = (lf + uf) / f256::TWO;
        }
        let cutoff = FP492::from(&BigFloat::from(&f));
        // println!("\n{lf:?}\n{:?}", lf.cos());
        // println!("\n{f:?}\n{:?}", f.cos());
        // println!("\n{uf:?}\n{:?}", uf.cos());
        // println!("\n// {f:e}");
        // println!("{:?};", cutoff);

        assert_eq!(f.cos(), f256::ONE);
        assert_eq!(cutoff, SMALL_CUT_OFF);
        let g = f + f.ulp();
        assert_ne!(g.cos(), f256::ONE);
    }
}
