// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use std::ops::Neg;

use super::{
    two_over_pi::{get_511_bits, TWO_OVER_PI},
    u256, BigFloat, FP509,
};
use crate::{
    big_uint::u512, consts::TAU, f256, FRACTION_BITS, SIGNIFICAND_BITS, TWO,
};

const FP_FRAC_PI_4: FP509 = FP509::new(
    0x1921fb54442d18469898cc51701b839a,
    0x252049c1114cf98e804177d4c7627364,
    0x4a29410f31c6809bbdf2a33679a74863,
    0x6605614dbe4be286e9fc26adadaa3849,
);
const FP_FRAC_PI_2: FP509 = FP509::new(
    0x3243f6a8885a308d313198a2e0370734,
    0x4a4093822299f31d0082efa98ec4e6c8,
    0x9452821e638d01377be5466cf34e90c6,
    0xcc0ac29b7c97c50dd3f84d5b5b547091,
);
const FP_FRAC_3_PI_4: FP509 = FP509::new(
    0x4b65f1fccc8748d3c9ca64f450528ace,
    0x6f60dd4333e6ecab80c4677e56275a2c,
    0xde7bc32d955381d339d7e9a36cf5d92a,
    0x321023e93ae3a794bdf4740908fea8da,
);
const FP_PI: FP509 = FP509::new(
    0x6487ed5110b4611a62633145c06e0e68,
    0x948127044533e63a0105df531d89cd91,
    0x28a5043cc71a026ef7ca8cd9e69d218d,
    0x98158536f92f8a1ba7f09ab6b6a8e123,
);
const FP_FRAC_5_PI_4: FP509 = FP509::new(
    0x7da9e8a554e17960fafbfd9730899202,
    0xb9a170c55680dfc881475727e4ec40f5,
    0x72ce454bf8e0830ab5bd3010604469f0,
    0xfe1ae684b77b6ca291ecc1646453196c,
);
const FP_FRAC_7_PI_4_M4: FP509 = FP509::new(
    0x2feddf4ddd3ba9ee2c2d963a10c09937,
    0x03e20447791ad2e581ca46d173b127be,
    0x0720c76a5c6d844231a2767d5392fab7,
    0xca25a920341331b065e50ebfbfa789fd,
);
const FP_FRAC_9_PI_4_M4: FP509 = FP509::new(
    0x6231d5f66595da7b5d5f2edcf0f7a06b,
    0x4e2297c99bb4c602824d367b02760e86,
    0x9b734988bffa8579ad87bcea46e18b7e,
    0x96306bbbb0aaf6be39dd5c1b1afbfa8f,
);

// For the input value x, calculate ⌈x/½π⌋ % 4 and x % ½π
fn fp_reduce(exp: i32, x: &f256) -> (u32, FP509) {
    debug_assert!(exp >= -1);
    debug_assert!(x.is_sign_positive());
    match exp {
        -1 => {
            // ½ <= |x| < 1
            let mut fx = FP509::from(x);
            if fx <= FP_FRAC_PI_4 {
                return (0, fx);
            }
            fx -= &FP_FRAC_PI_2;
            return (1, fx);
        }
        0 => {
            // 1 <= |x| < 2
            let mut fx = FP509::from(x);
            fx -= &FP_FRAC_PI_2;
            return (1, fx);
        }
        1 => {
            // 2 <= |x| < 4
            let mut fx = FP509::from(x);
            if fx <= FP_FRAC_3_PI_4 {
                fx -= &FP_FRAC_PI_2;
                return (1, fx);
            } else if fx <= FP_FRAC_5_PI_4 {
                fx -= &FP_PI;
                return (2, fx);
            } else {
                fx -= &FP_PI;
                fx -= &FP_FRAC_PI_2;
                return (3, fx);
            }
        }
        2 => {
            // 4 <= |x| < 8
            let mut fx = FP509::from(&(x - f256::from_u64(4)));
            if fx <= FP_FRAC_7_PI_4_M4 {
                fx += &FP509::TWO;
                fx -= &FP_PI;
                fx += &FP509::TWO;
                fx -= &FP_FRAC_PI_2;
                return (3, fx);
            } else if fx <= FP_FRAC_9_PI_4_M4 {
                fx -= &FP_PI;
                fx += &FP509::TWO;
                fx -= &FP_PI;
                fx += &FP509::TWO;
                return (0, fx);
            } else {
                fx -= &FP_PI;
                fx += &FP509::TWO;
                fx -= &FP_PI;
                fx += &FP509::TWO;
                fx -= &FP_FRAC_PI_2;
                return (1, fx);
            }
        }
        _ => {}
    }
    fma_reduce(exp, x)
}

// Accurate range reduction algorithm, adapted from
// S. Boldo, M. Daumas, R.-C. Li,
// Formally verified argument reduction with a fused multiply-add
// IEEE Trans. Comput. 58(8), 1139–1145 (2009)
// For the input value x, calculate ⌈x/½π⌋ % 4 and x % ½π
fn fma_reduce(exp: i32, x: &f256) -> (u32, FP509) {
    // R = ◯₂₅₅(1/½π) =
    // 0.6366197723675813430755350534900574481378385829618257949906693762355871905369
    const R: BigFloat = BigFloat::new(
        1,
        -1,
        (
            0x517cc1b727220a94fe13abe8fa9a6ee0,
            0x6db14acc9e21c820ff28b1d5ef5de2b1,
        ),
    );
    // C = ◯₂₅₅(1/R) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    // C1 = ◯₂₅₃(C) =
    // 1.57079632679489661923132169163975144209858469968755291048747229615390820314306
    const C1: BigFloat = BigFloat::new(
        1,
        0,
        (
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd90,
        ),
    );
    // C2 = ⌈(C - C1) / 8⋅ulp(ulp(C1))⌋ ⋅ 8⋅ulp(ulp(C1)) =
    // 4.0029261425274538885256060583180088389717792640288565295989842465635080655216e-77
    const C2: BigFloat = BigFloat::new(
        1,
        -254,
        (
            0x4a29410f31c6809bbdf2a33679a74863,
            0x6605614dbe4be286e9fc26adadaa3848,
        ),
    );
    // C3 = ◯₂₅₅(C - C1 - C2) =
    // 8.7899010274302568753719915932600646668568561314064066646591366424167272794258e-154
    const C3: BigFloat = BigFloat::new(
        1,
        -509,
        (
            0x5e485b576625e7ec6f44c42e9a637ed6,
            0xb0bff5cb6f406b7edee386bfb5a899fa,
        ),
    );
    // D = 3⋅2²⁵³ =
    // 43422033463993573283839119378257965444976244249615211514796594002967423614976
    const D: BigFloat = BigFloat::new(
        1,
        254,
        (
            0x60000000000000000000000000000000,
            0x00000000000000000000000000000000,
        ),
    );
    // Max exponent for fast_reduce
    const M: i32 = 253;
    // Number of significant bits in C1 + C2
    const C1_C2_PREC: u32 = 2 * BigFloat::FRACTION_BITS - 2;

    if exp <= M {
        let x = BigFloat::from(x);
        let z = x.mul_add(&R, &D) - &D;
        let u1 = z.neg().mul_add(&C1, &x);
        let needed_bits = 1 - u1.exp + BigFloat::FRACTION_BITS as i32 + x.exp;
        let (v1, v2) = if needed_bits <= C1_C2_PREC as i32 {
            let v1 = z.neg().mul_add(&C2, &u1);
            let (p1, p2) = z.mul_exact(&C2);
            let (t1, t2) = u1.sum_exact(&p1.neg());
            (v1, ((t1 - &v1) + &t2) - &p2)
        } else {
            let u2 = z.neg().mul_add(&C2, &u1);
            let v1 = z.neg().mul_add(&C3, &u2);
            let (p1, p2) = z.mul_exact(&C3);
            let (t1, t2) = u2.sum_exact(&p1.neg());
            (v1, ((t1 - &v1) + &t2) - &p2)
        };
        // x <= M => z < 2ᴾ⁻²
        let e = z.exp - BigFloat::FRACTION_BITS as i32;
        let q = (&z.signif >> e.unsigned_abs()).lo as u32 & 0x3;
        debug_assert!(v1.abs() <= BigFloat::FRAC_PI_4);
        // Convert (v1 + v2) into a fixed-point number with 509-bit-fraction
        // |v1| <= ½π => v1.exp <= 0
        let mut fx = FP509::from(&v1);
        fx += &FP509::from(&v2);
        return (q, fx);
    }
    large_val_reduce(exp, x)
}

// Range reduction algorithm, adapted from
// M.Payne, R.Hanek,
// Radian reduction for trigonometric functions
// SIGNUM Newsletter 18, p. 19–24
// For the input value x, calculate ⌈x/½π⌋ % 4 and x % ½π
fn large_val_reduce(e: i32, x: &f256) -> (u32, FP509) {
    debug_assert!(e >= SIGNIFICAND_BITS as i32 + 1);

    let m = x.integral_significand();
    // Now we have |x| = m⋅2ᵉ⁻ᴾ⁺¹,
    // where P = 237, e >= 236 and 2ᴾ⁻¹ <= m <= 2ᴾ-1.
    //
    // Let C = ½π and R = 2/π.
    // We want to calculate
    // k = ⌈x/C⌋ = ⌈x⋅R⌋ and y = x - k⋅C = C⋅(x⋅R - k).
    // For further processing of the circular functions we only need the last
    // two bits of k, i.e. k mod 4.
    // Let i = e - P.
    // Then x⋅R = m⋅2ᵉ⁻ᴾ⁺¹⋅R = m⋅2ⁱ⁺¹⋅R.
    // Let 0.d₋₁d₋₂d₋₃... be the infinite binary expansion of R.
    // Split R so that
    // R = R₀⋅2¹⁻ⁱ + (R₁ + R₂)⋅2⁻ⁱ⁻⁵¹⁰, where
    // R₀ = 0 if e <= P + 1, else
    // R₀ = 0d₋₁...d₁₋ᵢ
    // R₁ = d₋ᵢd₋ᵢ₋₁...d₋ᵢ₋₅₁₀
    // R₂ = 0.d₋ᵢ₋₅₁₁...
    // Then x⋅R = m⋅4⋅R₀ + m⋅2⁻⁵⁰⁹⋅R₁ + m⋅2⁻⁵⁰⁹⋅R₂.
    // The first part will - when multiplied by C - become a multiple of 2π
    // and thus can be ignored for trigonometric functions.
    // The last part is too small to have a relevant influence in the result.
    let i = e as u32 - SIGNIFICAND_BITS;
    let r1 = get_511_bits(i - 1);
    let (tl, th1) = m.widening_mul(&r1.lo);
    let (th2, _) = m.widening_mul(&r1.hi);
    let (mut th, _) = th1.overflowing_add(&th2);
    let mut f = u512::new(th, tl);
    let mut k = (th.hi >> u128::BITS - 3) as u32;
    th.hi &= (1 << (u128::BITS - 3)) - 1;
    let mut y = FP509::from(u512::new(th, tl));
    if y > FP509::ONE_HALF || (y == FP509::ONE_HALF && (k & 1) == 1) {
        k += 1;
        y -= &FP509::ONE;
    };
    y.imul_round(&FP_FRAC_PI_2);
    (k % 4, y)
}

/// Calculate ⌈x/½π⌋ % 4 and x % ½π.
#[inline]
pub(super) fn reduce(x: &f256) -> (u32, FP509) {
    debug_assert!(x.is_finite() && x.is_sign_positive());
    let x_exp = x.exponent();
    if x_exp <= -2 {
        // |x| < ½ => no need for reduction
        return (0, FP509::from(x));
    }
    fp_reduce(x_exp, &x)
}

#[cfg(test)]
mod reduce_tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_frac_pi_4() {
        let f = f256::from(&FP_FRAC_PI_4);
        let fx = FP509::from(&f);
        assert!(fx < FP_FRAC_PI_4);
        let (q, r) = reduce(&f);
        assert_eq!(q, 0);
        assert_eq!(r, fx);
    }

    #[test]
    fn test_frac_pi_2() {
        let f = f256::from(&FP_FRAC_PI_2);
        let mut fx = FP509::from(&f);
        assert!(fx < FP_FRAC_PI_2);
        let (q, r) = reduce(&f);
        assert_eq!(q, 1);
        fx -= &FP_FRAC_PI_2;
        assert_eq!(r, fx);
    }

    #[test]
    fn test_pi() {
        let f = f256::from(&FP_PI);
        let mut fx = FP509::from(&f);
        assert!(fx < FP_PI);
        let (q, r) = reduce(&f);
        assert_eq!(q, 2);
        fx -= &FP_PI;
        assert_eq!(r, fx);
    }

    #[test]
    fn test_frac_5_pi_4() {
        let f = f256::from(&FP_FRAC_5_PI_4);
        let mut fx = FP509::from(&f);
        assert!(fx < FP_FRAC_5_PI_4);
        let (q, r) = reduce(&f);
        assert_eq!(q, 2);
        fx -= &FP_PI;
        assert_eq!(r, fx);
    }

    #[test]
    fn test_near_pi_over_2() {
        // 1.570796326794896619231321691639751442098584699687552910487472296153908199
        let f = f256::from_sign_exp_signif(
            0,
            -236,
            (
                0x00001921fb54442d18469898cc51701b,
                0x839a252049c1114cf98e804177d4c762,
            ),
        );
        // -4.081838735141263582281490600494564033380656130039804322382899197982948856e-72
        let r = f256::from_sign_exp_signif(
            1,
            -474,
            (
                0x00001cd9128a5043cc71a026ef7ca8cd,
                0x9e69d218d98158536f92f8a1ba7f09ab,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 1);
        assert_eq!(f256::from(&fx), r);
    }

    #[test]
    fn test_near_5_pi_over_2() {
        // 7.853981633974483096156608458198757210492923498437764552437361480769541013
        let f = f256::from_sign_exp_signif(
            0,
            -234,
            (
                0x00001f6a7a2955385e583ebeff65cc22,
                0x6480ae685c3155a037f22051d5c9f93b,
            ),
        );
        // -2.297835518052893176389214420697236605538218253920745232749266210043741685e-72
        let r = f256::from_sign_exp_signif(
            1,
            -474,
            (
                0x0000103d5cb39152fe3820c2ad6f4c04,
                0x18111a7c3f86b9a12ddedb28a47b3059,
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
        // -2.6902318393958948138169491581374663396316393610919104502987401460736171854647e-72
        let r = BigFloat::new(
            -1,
            -238,
            (
                0x4c0d390b2059bc41923ce8f474398c88,
                0xf76daed66808d34acd2c658eaa8f6acc,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 2);
        assert_eq!(BigFloat::from(&fx), r);
    }

    #[test]
    fn test_near_2_pow_240() {
        // 1.647600973514298144155219627423120419897466257678199323002374021565490368e72
        let f = f256::from_sign_exp_signif(
            0,
            3,
            (
                0x00001dd71d552efac6c6246fb9a1e568,
                0x0e751b9e3f84e2a77bbc0298a0f4b498,
            ),
        );
        // 4.766543275114000144936176176450192810718081377608351404051118669257945354e-74
        let r = BigFloat::new(
            1,
            -244,
            (
                0x563d1ec38077f1810a728c57851aba2b,
                0x71bc78f95015c84078d6591b6b74a571,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 3);
        assert_eq!(BigFloat::from(&fx), r);
    }

    #[test]
    fn test_2_pow_300() {
        let f = f256::from_sign_exp_signif(
            0,
            64,
            (
                0x00001000000000000000000000000000,
                0x00000000000000000000000000000000,
            ),
        );
        let r = BigFloat::new(
            -1,
            -3,
            (
                0x6d6426ef8f5dd348fe6e5056a1746784,
                0x2274a7eb9c97d04b267a41e0d1898945,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 1);
        assert_eq!(BigFloat::from(&fx), r);
    }

    #[test]
    fn test_2_pow_15236() {
        let f = f256::from_sign_exp_signif(
            0,
            15000,
            (
                0x00001000000000000000000000000000,
                0x00000000000000000000000000000000,
            ),
        );
        // 0.3742030203459253563266600098797338765224469432914356135160726857543409513358
        let r = BigFloat::new(
            1,
            -2,
            (
                0x5fcbc4e6733e79165158c5a8465bd4af,
                0x02ee033f64e42764159bba795550da8e,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 1);
        assert_eq!(BigFloat::from(&fx), r);
    }

    #[test]
    fn test_near_2_pow_175000() {
        // 1.880206549610325586327114279571172497400670406622609569302813638907329340e52680
        let f = f256::from_sign_exp_signif(
            0,
            174764,
            (
                0x000010f258e66993bcca1cccec6818d0,
                0x53e47fc6c26e2ed5626a9c63a353cf87,
            ),
        );
        // -3.593598548722686367734065727968384533432676656819918617957851704757758941e-72
        let r = f256::from_sign_exp_signif(
            1,
            -474,
            (
                0x00001965b922a61c5aef27c2f900f9ef,
                0x860efa16a7027421c5949f98ac80caea,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 3);
        assert_eq!(f256::from(&fx), r);
    }

    #[test]
    fn test_nearest_to_multible_of_pi_over_2() {
        // 2.970070222171535186122403750332127410373506874041943887689128979638136212e55900
        let f = f256::from_sign_exp_signif(
            0,
            185461,
            (
                0x000014683988582470d7cad38fb584b4,
                0xf1c2d085d2933836d205e79e8f61e4ee,
            ),
        );
        // -5.769198204535869190785720230896528973489817286545990660946235357113661705e-77
        let r = f256::from_sign_exp_signif(
            1,
            -490,
            (
                0x00001ab89a0b66c5b89c0aa9c372047b,
                0xfa9e02e32987ffbf9bd44278301166fd,
            ),
        );
        let (q, fx) = reduce(&f);
        assert_eq!(q, 0);
        assert_eq!(f256::from(&fx), r);
    }
}
