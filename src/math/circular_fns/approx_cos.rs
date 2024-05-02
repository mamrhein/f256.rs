// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::FP509;

const N: usize = 33;

const COEFFS: [FP509; N] = [
    // 1 / 64! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x0000000000201b9ed4641c376e304c2d,
        0x540bb35fe7c1d6b891e808f219052511,
    ),
    // -1 / 62! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
        0xfffffffe064cfa6ed74396f887503614,
        0x47b6d97dd30a2505f9731af5eef838c1,
    ),
    // 1 / 60! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00001d2eeac43e7fcf77a460f5011064,
        0x88aee724382109bcc57fb2b798e98b59,
    ),
    // -1 / 58! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffffffff,
        0xfe6c73399e4fc09f1d92f3440d4951cd,
        0xed6bbf27d72559a4f62cad31828512ff,
    ),
    // 1 / 56! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000014,
        0x5b77f9e98e12792c1446732c6b2592a3,
        0xee8d677f97a853ace30b5ea07578ab1a,
    ),
    // -1 / 54! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffffffffff0b13,
        0x84893e0ab1be25ac10665196e3f3bbb1,
        0xeabac8e75ed147f45f35857aac356f80,
    ),
    // 1 / 52! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x0000000000000000000000000ab22bcc,
        0x49ac6470e436d660a81bdb179123a0f1,
        0xcbcdf559f84d91ff97a5be8ec29b74a1,
    ),
    // -1 / 50! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffffffff91324247b4,
        0xca1f7e83d7eb2eb27f6e57dc72e8b724,
        0xb6664ff7bc6b8c3906fdf117fd93d075,
    ),
    // 1 / 48! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000000000004246d01adbfc9,
        0x9e973637973b17b871fb243c48d742a6,
        0x60d6af16c2bc063b15b2a8672f5effa5,
    ),
    // -1 / 46! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xffffffffffffffffdb7eff6134d5df3a,
        0x6b72361b473ef6938ad0acbe1704a5ca,
        0x9c19076be7091750c993f2ae8ad3248e,
    ),
    // 1 / 44! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00000000000001272b1b03fec6a4fd9f,
        0x327e7f6de8e232fb8cab36f1e06b6bb5,
        0xcd9dfd81dc7d78c20db3b0a978b668e0,
    ),
    // -1 / 42! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xfffffffffff77a856c56790be49d936e,
        0xd11a37b2d83f34e45e9dec616617f45c,
        0x5c5a6c3e70af85d2bbfa5b7fdbe0eb60,
    ),
    // 1 / 40! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x000000003951da854255c600340e4895,
        0x75a556fd66d637db8db7f8d742de4aba,
        0xc7c3e3fe0355d47b8df485f2f910bef5,
    ),
    // -1 / 38! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xfffffea2b53463f3c5516ec2c8f5b13b,
        0x1875e7d5569b9e1866eba04085689dce,
        0xae4aac1bacf92f16f5efbf6241f45c00,
    ),
    // 1 / 36! ≈
    FP509::new(
        0x00000000000000000000000000000000,
        0x00077e60ca430b2a4ac1ae344a9c9d6f,
        0xa870ba4e5551b5fabde5dda34b6d4ade,
        0xc1e2bfffff6f5fe54542e259c3eeb876,
    ),
    // -1 / 34! ≈
    FP509::new(
        0xffffffffffffffffffffffffffffffff,
        0xdb1dfb9c7e050bd80eba96a0c5291e6e,
        0xf52b067411d451e158a12048c2137f9d,
        0xb7f70002c7d4038f1acdee2fa50c1916,
    ),
    // 1 / 32! ≈
    FP509::new(
        0x000000000000000000000000000000a1,
        0xa6973c1fade2170f7237d35fe1c89db1,
        0x796db749db7122598dd0811d668aaebf,
        0xb771f3d030c866cc8572132ea0fa0ead,
    ),
    // -1 / 30! ≈
    FP509::new(
        0xfffffffffffffffffffffffffffd8d9a,
        0x75f7053e33e6a42567acec75169cf049,
        0x76d9c1cda99ae4fa780bae12a69ad919,
        0x266f3942f771a77ae5f5ab50370722c6,
    ),
    // 1 / 28! ≈
    FP509::new(
        0x0000000000000000000000000850c513,
        0x1a842e9b9e2e28e1aa546a1526a76656,
        0x17f75f119b99d4cc084e6c9dcdc23487,
        0x6203666b13c0d4567f1bcd64fdbfd463,
    ),
    // -1 / 26! ≈
    FP509::new(
        0xffffffffffffffffffffffe7717a0395,
        0xb1a65c70dfaf4594feb6b989d9a5c9c1,
        0x397b40007db7957778673dfc5e7ce032,
        0x8df58bc9aa8cf090a1e571c2a580cb7b,
    ),
    // 1 / 24! ≈
    FP509::new(
        0x000000000000000000003e59e032e5ea,
        0xef9949680cf953b1440ce7fd610dbb64,
        0x0d0f7ec0cbde7ea849dc9d3816eebfa3,
        0x8e8b11f4f62530c4ef6d27c7c6fb58dd,
    ),
    // -1 / 22! ≈
    FP509::new(
        0xffffffffffffffffff798e3492403d6b,
        0x5d79b7a4066389c5442bc5a6ba63f043,
        0xd696b048683ee520bc4cff0e8d32c754,
        0xa42147cd3fced75bbca2393af20863a8,
    ),
    // 1 / 20! ≈
    FP509::new(
        0x0000000000000000f2a15d201011283d,
        0x4e5695fc785d5dfef9014d1b9fa46592,
        0xbc0ddd53de7e7eec2d0bb3bd2e5c453f,
        0xcbf06b96d8b75871933cb79f34dc26ae,
    ),
    // -1 / 18! ≈
    FP509::new(
        0xfffffffffffffe97d879c468268844ff,
        0xb7795d3d5568798662118aff07f93a30,
        0xdb6b7781bc3b996d22a1332f2f09354d,
        0x47204c164fd8b76971df6fad8936960d,
    ),
    // 1 / 16! ≈
    FP509::new(
        0x000000000001ae7f3e733b81f11d8656,
        0xb0ee8cafe91ebd5ec707db2878187199,
        0xb98b26ed00c29b8c9b50d199c6fe49a0,
        0xfb650d548ef4c3f5e2ec8291fcc0a4dd,
    ),
    // -1 / 14! ≈
    FP509::new(
        0xfffffffffe6c68b573f8362df4520eba,
        0x205c1b15732e772568a28a0f69157fe2,
        0x0d8b81cf498e2c2e643b7fd5719af914,
        0x514380b9fa88497b424597230b65706c,
    ),
    // 1 / 12! ≈
    FP509::new(
        0x000000011eed8eff8d897b544da987ac,
        0xfe84bec01cf74b679c71d90b4ab7154a,
        0x5ed1b6a1b4ec9704bdb31e413bd2eb8e,
        0x3a027bc7e31bc25ee2868d15e5e212f8,
    ),
    // -1 / 10! ≈
    FP509::new(
        0xffffff6c0d82443b051c6887f4960acc,
        0xc38da4f1107d1e93554c162d799905a7,
        0x1bddd49eb602218e2fa4665d273e8aaa,
        0x16b82ceee5afc713329f40b5776e3850,
    ),
    // 1 / 8! ≈
    FP509::new(
        0x00003403403403403403403403403403,
        0x40340340340340340340340340340340,
        0x34034034034034034034034034034034,
        0x03403403403403403403403403403403,
    ),
    // -1 / 6! ≈
    FP509::new(
        0xfff49f49f49f49f49f49f49f49f49f49,
        0xf49f49f49f49f49f49f49f49f49f49f4,
        0x9f49f49f49f49f49f49f49f49f49f49f,
        0x49f49f49f49f49f49f49f49f49f49f4a,
    ),
    // 1 / 4! ≈
    FP509::new(
        0x01555555555555555555555555555555,
        0x55555555555555555555555555555555,
        0x55555555555555555555555555555555,
        0x55555555555555555555555555555555,
    ),
    // -1 / 2! ≈
    FP509::new(
        0xf0000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
    // 1 / 0! ≈
    FP509::new(
        0x20000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
        0x00000000000000000000000000000000,
    ),
];

// 2.12787206838507003880796293968184411684206163544728281856444725083506186e-36
const SMALL_CUT_OFF: FP509 = FP509::new(
    0x0000000000000000000000000000005a,
    0x827999fcef32422cbec4d9baa55f4f8e,
    0xb7b05d449dd426768bd642c198000000,
    0x00000000000000000000000000000000,
);

pub(crate) fn approx_cos(x: &FP509) -> FP509 {
    let mut x_abs = *x;
    x_abs.iabs();
    // If x is zero or very small, cosine x == 1.
    if x_abs <= SMALL_CUT_OFF {
        return FP509::ONE;
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
        let cutoff = FP509::from(&BigFloat::from(&f));
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
