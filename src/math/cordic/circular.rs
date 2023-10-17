// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{
    big_uint::u256,
    math::{cordic::atan_table::ATANS, FP248},
};

// Cordic gain factor
// ≈1.64676025812106564836605122228229843565237672570102740901240531755172816
pub(crate) const K: FP248 = FP248 {
    sign: 0,
    signif: u256::new(
        0x01a592148cfb84d103c1366ddf3b121e,
        0x0064b3eedcea488bf8ae4bebf8356685,
    ),
};
// 1 / K
// ≈0.60725293500888125616944675250492826311239085215008977245697601311014788
pub(crate) const P: FP248 = FP248 {
    sign: 0,
    signif: u256::new(
        0x009b74eda8435e5a67f5f9092bd7fd40,
        0xe9c288c51a3bd9f449b48004d6ae3ad2,
    ),
};

fn no_op(fp: &mut FP248) {}

const OPS: [fn(&mut FP248); 2] = [no_op, FP248::flip_sign];
const MAX_ABS_COORD: u256 = u256::new(1_u128 << 127, 0_u128);

// Circular coordinates, vector mode
pub(crate) fn cordic_circ_vm(
    mut x: FP248,
    mut y: FP248,
    mut z: FP248,
) -> (FP248, FP248) {
    debug_assert!(x.signif <= MAX_ABS_COORD);
    debug_assert!(y.signif <= MAX_ABS_COORD);

    for i in 0..=FP248::FRACTION_BITS {
        let op = OPS[(y >= FP248::ZERO) as usize];
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
    (x, z)
}

#[inline(always)]
pub(crate) fn cordic_atan(a: &FP248) -> FP248 {
    cordic_circ_vm(FP248::ONE, *a, FP248::ZERO).1
}

#[inline(always)]
pub(crate) fn cordic_atan2(y: &FP248, x: &FP248) -> FP248 {
    cordic_circ_vm(*x, *y, FP248::ZERO).1
}

#[cfg(test)]
mod vector_mode_tests {
    use std::ops::Neg;

    use super::*;

    #[test]
    fn test_scale_factor() {
        let (k, _) = cordic_circ_vm(FP248::ONE, FP248::ZERO, FP248::ZERO);
        assert_eq!(k, K);
    }

    #[test]
    fn test_atan_inf() {
        let e = FP248 {
            sign: 0,
            signif: u256::new(0_u128, 0x7ff_u128),
        };
        let (_, mut a) = cordic_circ_vm(FP248::ZERO, FP248::ONE, FP248::ZERO);
        a -= &FP248::FRAC_PI_2;
        assert!(a.signif < e.signif);
    }

    #[test]
    fn test_atan_one() {
        let a = cordic_atan(&FP248::ONE);
        assert_eq!(a, ATANS[0]);
    }

    #[test]
    fn test_atan_max() {
        let e = FP248 {
            sign: 0,
            signif: u256::new(0_u128, 0x7ff_u128),
        };
        let mut a = cordic_atan2(&FP248::ONE, &FP248::EPSILON);
        a -= &FP248::FRAC_PI_2;
        assert!(a.signif < e.signif);
    }

    #[test]
    fn test_atan_signs() {
        let m = FP248 {
            sign: 0,
            signif: MAX_ABS_COORD,
        };
        for f in
            [FP248::ZERO, FP248::EPSILON, FP248::ONE, FP248::FRAC_PI_2, m]
        {
            assert_eq!(f.atan().neg(), f.neg().atan());
        }
    }
}

// Circular coordinates, rotation mode
pub(crate) fn cordic_circ_rm(
    mut x: FP248,
    mut y: FP248,
    mut z: FP248,
) -> (FP248, FP248) {
    debug_assert!(z >= FP248::ZERO);
    debug_assert!(z <= FP248::FRAC_PI_2);

    for i in 0..=FP248::FRACTION_BITS {
        let op = OPS[(z < FP248::ZERO) as usize];
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
pub(crate) fn cordic_sin_cos(a: &FP248) -> (FP248, FP248) {
    cordic_circ_rm(P, FP248::ZERO, *a)
}

#[cfg(test)]
mod rotation_mode_tests {
    use super::*;

    #[test]
    fn test_sin_cos_zero() {
        let e = FP248 {
            sign: 0,
            signif: u256::new(0_u128, 0xff_u128),
        };
        let (sin0, cos0) = cordic_sin_cos(&FP248::ZERO);
        let mut d = cos0.clone();
        d -= &FP248::ONE;
        assert!(d.signif <= e.signif);
        assert!(sin0.signif < e.signif);
    }

    #[test]
    fn test_sin_cos_pi_half() {
        let e = u256::new(0_u128, 0x7ff_u128);
        let (sin, cos) = cordic_sin_cos(&FP248::FRAC_PI_2);
        assert_eq!(sin.sign, 0);
        assert_eq!(cos.sign, 0);
        let mut d = sin.clone();
        d -= &FP248::ONE;
        assert!(d.signif < e);
        assert!(cos.signif < e);
    }

    #[test]
    fn test_special_values() {
        let e = u256::new(0_u128, 0x7ff_u128);
        let f = &FP248::FRAC_PI_2 >> 1;
        let (sin, cos) = cordic_sin_cos(&f);
        let d = &cos.signif - &sin.signif;
        assert!(d < e);
    }
}
