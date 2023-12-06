// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{cmp::min, ops::Neg};

use crate::{
    big_uint::u256,
    math::{cordic::atan_table::ATANS, fp255::SIGNIF_ONE, FP255},
};

// Cordic gain factor
// ≈1.64676025812106564836605122228229843565237672570102740901240531755172816243915
pub(crate) const K: FP255 = FP255 {
    sign: 1,
    exp: 0,
    signif: u256::new(
        0x696485233ee13440f04d9b77cec48780,
        0x192cfbb73a9222fe2b92fafe0d59ae2a,
    ),
};
// 1 / K
// ≈0.607252935008881256169446752504928263112390852150089772456976013110147881208421
pub(crate) const P: FP255 = FP255 {
    sign: 1,
    exp: -1,
    signif: u256::new(
        0x4dba76d421af2d33fafc8495ebfea074,
        0xe144628d1decfa24da40026b571d5faf,
    ),
};

fn no_op(fp: &mut FP255) {}

const OPS: [fn(&mut FP255); 2] = [no_op, FP255::flip_sign];
const MAX_ABS_COORD: FP255 = FP255::FRAC_PI_2;

// Circular coordinates, vector mode
pub(crate) fn cordic_circ_vm(
    mut x: FP255,
    mut y: FP255,
    mut z: FP255,
) -> (FP255, FP255) {
    debug_assert!(x.sign >= 0);
    debug_assert!(y.sign >= 0);
    debug_assert!(y <= MAX_ABS_COORD);

    for i in 0..=FP255::FRACTION_BITS {
        let op = OPS[(y >= FP255::ZERO) as usize];
        let mut dx = &y >> i;
        op(&mut dx);
        let mut dy = &x >> i;
        op(&mut dy);
        // println!("=== {i} ===");
        // let xx = &x.signif >> (6 - x.exp) as u32;
        // println!("{} {:032x} {:032x}", x.sign, xx.hi, xx.lo);
        // let dxx = &dx.signif >> (6 - dx.exp) as u32;
        // println!("{} {:032x} {:032x}", dx.sign, dxx.hi, dxx.lo);
        // println!("{x:?}");
        // println!("{dx:?}");
        x -= &dx;
        // println!("***");
        // let yy = &y.signif >> (6 - y.exp) as u32;
        // println!("{} {:032x} {:032x}", y.sign, yy.hi, yy.lo);
        // let dyy = &dy.signif >> (6 - dy.exp) as u32;
        // println!("{} {:032x} {:032x}", dy.sign, dyy.hi, dyy.lo);
        // println!("{y:?}");
        // println!("{dy:?}");
        // debug_assert!(y.sign != dy.sign || dy < y);
        // let t = y.clone();
        y += &dy;
        // assert!(
        //     t.sign != y.sign || t.abs() > y.abs(),
        //     " t: {:?}\ndy: {:?}\n y: {:?}",
        //     t,
        //     dy,
        //     y
        // );
        let mut a = ATANS[i as usize];
        op(&mut a);
        // println!("{i}: {z:?}");
        // println!("{a:?}");
        z -= &a;
    }
    (x, z)
}

pub(crate) fn cordic_atan(mut f: FP255) -> FP255 {
    if f.is_zero() {
        return FP255::ZERO;
    };
    let f_sign = f.sign;
    f.sign = 1;
    // Convert f into a fraction of two values x and y, so that
    // f = y / x and y < ½π.
    let x = FP255 {
        sign: 1,
        exp: min(-f.exp, 0) - 1,
        signif: SIGNIF_ONE,
    };
    let y = FP255 {
        sign: 1,
        exp: min(f.exp, 0) - 1,
        signif: f.signif,
    };
    let mut a = cordic_circ_vm(x, y, FP255::ZERO).1;
    a.sign = f_sign;
    a
}

pub(crate) fn cordic_atan2(y: &FP255, x: &FP255) -> FP255 {
    let mut y_dash = y.abs();
    let mut x_dash = x.abs();
    // Assure y' < ½π.
    x_dash.exp -= y_dash.exp + 1;
    y_dash.exp = -1;
    let mut a = cordic_circ_vm(x_dash, y_dash, FP255::ZERO).1;
    match (y.sign, x.sign) {
        (-1, 1) => a.flip_sign(),
        (1, -1) => {
            a.flip_sign();
            a += &FP255::PI;
        }
        (-1, -1) => a -= &FP255::PI,
        _ => {}
    }
    a
}

const MAX_ERR: FP255 = FP255 {
    sign: 1,
    exp: -248,
    signif: FP255::ONE.signif,
};

#[cfg(test)]
mod vector_mode_tests {
    use core::ops::Neg;

    use super::*;

    #[test]
    fn test_scale_factor() {
        let (k, _) = cordic_circ_vm(FP255::ONE, FP255::ZERO, FP255::ZERO);
        let mut d = k.clone();
        d -= &K;
        assert!(d.abs() <= MAX_ERR, "{:?}\n{:?}", d.abs(), MAX_ERR);
    }

    #[test]
    fn test_atan_inf() {
        let (_, mut a) = cordic_circ_vm(FP255::ZERO, FP255::ONE, FP255::ZERO);
        a -= &FP255::FRAC_PI_2;
        assert!(a.abs() < MAX_ERR, "{:?}\n{:?}", a.abs(), MAX_ERR);
    }

    #[test]
    fn test_atan_one() {
        let mut a = cordic_atan(FP255::ONE);
        a -= &ATANS[0];
        assert!(a.abs() < MAX_ERR, "{:?}\n{:?}", a.abs(), MAX_ERR);
    }

    #[test]
    fn test_atan_max() {
        let mut a = cordic_atan2(&FP255::ONE, &FP255::EPSILON);
        a -= &FP255::FRAC_PI_2;
        assert!(a < MAX_ERR, "{:?}\n{:?}", a, MAX_ERR);
    }

    #[test]
    fn test_atan_signs() {
        for f in [FP255::ZERO, FP255::EPSILON, FP255::ONE, FP255::FRAC_PI_2] {
            assert_eq!(f.atan().neg(), f.neg().atan(), "f: {f:?}");
        }
    }
}

// Circular coordinates, rotation mode
pub(crate) fn cordic_circ_rm(
    mut x: FP255,
    mut y: FP255,
    mut z: FP255,
) -> (FP255, FP255) {
    debug_assert!(z >= FP255::ZERO);
    debug_assert!(z <= MAX_ABS_COORD);

    for i in 0..=FP255::FRACTION_BITS {
        let op = OPS[(z < FP255::ZERO) as usize];
        let mut dx = &y >> i;
        op(&mut dx);
        let mut dy = &x >> i;
        op(&mut dy);
        x -= &dx;
        y += &dy;
        let mut a = ATANS[i as usize];
        op(&mut a);
        z -= &a;
    }
    (y, x)
}

#[inline(always)]
pub(crate) fn cordic_sin_cos(a: &FP255) -> (FP255, FP255) {
    cordic_circ_rm(P, FP255::ZERO, *a)
}

#[cfg(test)]
mod rotation_mode_tests {
    use super::*;

    #[test]
    fn test_sin_cos_zero() {
        let (sin0, cos0) = cordic_sin_cos(&FP255::ZERO);
        let mut d = cos0.clone();
        d -= &FP255::ONE;
        assert!(d.abs() <= MAX_ERR);
        assert!(sin0.abs() < MAX_ERR);
    }

    #[test]
    fn test_sin_cos_pi_half() {
        let (sin, cos) = cordic_sin_cos(&FP255::FRAC_PI_2);
        let mut d = sin.clone();
        d -= &FP255::ONE;
        assert!(d.abs() < MAX_ERR);
        assert!(cos.abs() < MAX_ERR);
    }

    #[test]
    fn test_special_values() {
        let f = &FP255::FRAC_PI_2 >> 1;
        let (sin, cos) = cordic_sin_cos(&f);
        let mut d = cos.clone();
        d -= &sin;
        assert!(d.abs() < MAX_ERR);
    }
}
