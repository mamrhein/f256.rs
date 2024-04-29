// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use std::ops::Neg;

use super::{u256, BigFloat, FP509};
use crate::{
    consts::{FRAC_PI_4, TAU},
    f256, FRACTION_BITS,
};

// Accurate range reduction algorithm, adapted from
// S. Boldo, M. Daumas, R.-C. Li,
// Formally verified argument reduction with a fused multiply-add
// IEEE Trans. Comput. 58(8), 1139–1145 (2009)
// For the input value f, calculate ⌈f/½π⌋ % 4 and f % ½π
fn fast_reduce(f: &f256) -> (u32, FP509) {
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

    let x = BigFloat::from(f);
    let z = x.mul_add(&R, &D) - &D;
    let u = z.neg().mul_add(&C1, &x);
    let v1 = z.neg().mul_add(&C2, &u);
    let (p1, p2) = z.mul_exact(&C2);
    let (t1, t2) = u.sum_exact(&p1.neg());
    let v2 = ((t1 - &v1) + &t2) - &p2;
    // x <= M => z < 2ᴾ⁻²
    let e = z.exp - BigFloat::FRACTION_BITS as i32;
    let q = (&z.signif >> e.unsigned_abs()).lo as u32 & 0x3;
    debug_assert!(v1.abs() <= BigFloat::FRAC_PI_4);
    // Convert (v1 + v2) into a fixed-point number with 509-bit-fraction
    // |v1| < ½π => v1.exp <= 0
    let mut fx = FP509::from(&v1);
    fx += &FP509::from(&v2);
    (q, fx)
}

// Max exponent for fast_reduce
const M: i32 = 253;

/// Calculate ⌈x/½π⌋ % 4 and x % ½π.
pub(super) fn reduce(x: &f256) -> (u32, FP509) {
    let x_exp = x.quantum_exponent();
    if &x.abs() <= &FRAC_PI_4 {
        (0, FP509::from(x))
    } else if x_exp <= M {
        fast_reduce(x)
    } else {
        // The following algorithm is not accurate!
        // TODO: replace by impl of Payne-Hanek-Algorithm.
        // M < x <= f256::MAX
        const D: u256 = TAU.integral_significand();
        // x >= TAU => exp(x) >= 2 => following expression can't be < 0
        let sh = x.quantum_exponent() as u32 - 2;
        let mut t = x.integral_significand();
        t = t.lshift_rem(&D, sh);
        if t.is_zero() {
            return (0, FP509::ZERO);
        }
        let shl = t.leading_zeros() - 1;
        t <<= shl;
        let u = f256::from_sign_exp_signif(
            0,
            -((FRACTION_BITS + shl - 2) as i32),
            (t.hi, t.lo),
        );
        debug_assert!(u.abs() < TAU);
        debug_assert!(u.abs() <= x.abs());
        reduce(&u)
    }
}

#[cfg(test)]
mod reduce_tests {
    use super::*;

    #[test]
    fn test_near_pi_over_2() {
        // 1.570796326794896619231321691639751442098584699687552910487472296153908199
        let f = f256::from_sign_exp_signif(
            1,
            -235,
            (
                254876276031724631276054471292941,
                257605800546129575968570270509613867953,
            ),
        );
        // -4.081838735141263582281490600494564033380656130039804322382899197982948856e-72
        let r = f256::from_sign_exp_signif(
            0,
            -474,
            (
                585105718108822006372840578525389,
                210567475778970147682262528188485667243,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 1);
        assert_eq!(f256::from(&fx), r);
    }

    #[test]
    fn test_near_287162_pi_over_2() {
        // 451072.762503992264821001752482581001682512026226517387166060002390623476
        let f = f256::from_sign_exp_signif(
            0,
            -218,
            (
                558401033334757190931105923669791,
                18759989041509371038882973050122181661,
            ),
        );
        // -0.252291083838150703047132073301933401753305159681715343517117525111538028
        let r = f256::from_sign_exp_signif(
            1,
            -235,
            (
                40936568814036772438458176887655,
                243968839912839697982605558556793863273,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 2);
        assert_eq!(f256::from(&fx), r);
    }

    #[test]
    fn test_near_2_pow_93() {
        // 9.73479040006733330540476643817952771288809799461055977136144949973928394e27
        let f = f256::from_sign_exp_signif(
            0,
            -144,
            (
                637979223658812755503006773292533,
                179734367594601023673460503048988831334,
            ),
        );
        // 2.69023183939589481381694915813746633963163936109191045029874014607361719e-72
        let r = f256::from_sign_exp_signif(
            1,
            -474,
            (
                385627687521681818499319280246030,
                131771364287184707842005051688567941796,
            ),
        );
        println!("{f:e}");
        println!("{r:e}");
        let (q, fx) = reduce(&f);
        println!("{:?}", -fx);
        println!("{:e}", f256::from(&fx));
        assert_eq!(q, 2);
        assert_eq!(f256::from(&fx), r);
    }
}
