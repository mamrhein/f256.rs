// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use std::ops::Neg;

use super::{u256, BigFloat};

// Accurate range reduction algorithm, adapted from
// S. Boldo, M. Daumas, R.-C. Li,
// Formally verified argument reduction with a fused multiply-add
// IEEE Trans. Comput. 58(8), 1139–1145 (2009)
// For the input value f, calculate ⌊f/½π⌋ and f%½π
fn fast_reduce(x: &BigFloat) -> (u32, BigFloat, BigFloat) {
    // R = ◯₂₅₅(1/½π) =
    // 0.6366197723675813430755350534900574481378385829618257949906693762355871905369
    const R: BigFloat = BigFloat::new(
        0x517cc1b727220a94fe13abe8fa9a6ee0,
        0x6db14acc9e21c820ff28b1d5ef5de2b1,
        -1,
    );
    // C = ◯₂₅₅(1/R) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    // C1 = ◯₂₅₃(C) =
    // 1.57079632679489661923132169163975144209858469968755291048747229615390820314306
    const C1: BigFloat = BigFloat::new(
        0x6487ed5110b4611a62633145c06e0e68,
        0x948127044533e63a0105df531d89cd90,
        0,
    );
    // C2 = ⌈(C - C1) / 8⋅ulp(ulp(C1))⌋ ⋅ 8⋅ulp(ulp(C1)) =
    // 4.0029261425274538885256060583180088389717792640288565295989842465635080655216e-77
    const C2: BigFloat = BigFloat::new(
        0x4a29410f31c6809bbdf2a33679a74863,
        0x6605614dbe4be286e9fc26adadaa3848,
        -254,
    );
    // D = 3⋅2²⁵³ =
    // 43422033463993573283839119378257965444976244249615211514796594002967423614976
    const D: BigFloat = BigFloat::new(
        0x60000000000000000000000000000000,
        0x00000000000000000000000000000000,
        254,
    );

    let z = x.mul_add(&R, &D) - &D;
    let u = *x - &(z * &C1);
    let v1 = u - &(z * &C2);
    let (p1, p2) = z.mul_exact(&C2);
    let (t1, t2) = u.sum_exact(&p1.neg());
    let v2 = ((t1 - &v1) + &t2) - &p2;
    // x <= M => z < 2ᴾ⁻²
    let e = z.exp - BigFloat::FRACTION_BITS as i32;
    let q = (&z.signif >> e.unsigned_abs()).lo as u32 & 0x3;
    (q, v1, v2)
}

// Max input value for fast_reduce
// M = ◯₂₅₅((2²⁵³-1)⋅C) =
// 22735723555735395267514683923608116415837086083024526872423259444871180904135
const M: BigFloat = BigFloat::new(
    0x6487ed5110b4611a62633145c06e0e68,
    0x948127044533e63a0105df531d89cd8e,
    253,
);

pub(super) fn rem_frac_pi_2(x: &BigFloat) -> (u32, BigFloat, BigFloat) {
    let x_abs = x.abs();
    if &x_abs <= &BigFloat::FRAC_PI_4 {
        (0, *x, BigFloat::ZERO)
    } else if &x_abs <= &M {
        fast_reduce(x)
    } else {
        // M < x <= f256::MAX
        const D: u256 = BigFloat::TAU.signif;
        // x >= TAU => exp(x) >= 2 => following expression can't be < 0
        let sh = x.exp as u32 - 2;
        let mut t = x.signif;
        t = t.lshift_rem(&D, sh);
        if t.is_zero() {
            return (0, BigFloat::ZERO, BigFloat::ZERO);
        }
        let shl = t.leading_zeros() - 1;
        t <<= shl;
        let u = BigFloat::new(
            t.hi as i128,
            t.lo,
            -((BigFloat::FRACTION_BITS + shl - 2) as i32),
        );
        debug_assert!(u.abs() < BigFloat::TAU);
        debug_assert!(u.abs() <= x.abs());
        rem_frac_pi_2(&u)
    }
}
