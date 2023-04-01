// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod add_tests {
    use core::cmp::Ordering;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!((f256::NAN + f256::ONE).is_nan());
        assert!((f256::ONE + f256::NAN).is_nan());
        assert!((f256::NAN + f256::NAN).is_nan());
        assert!((f256::NAN + f256::INFINITY).is_nan());
        assert!((f256::INFINITY + f256::NAN).is_nan());
        assert!((f256::NAN + f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY + f256::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY + f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::INFINITY + f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE + f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY + f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_INFINITY + f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE + f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert!((f256::INFINITY + f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY + f256::INFINITY).is_nan());
    }

    #[test]
    fn test_zero() {
        // Because the normal cmp treats 0 == -0, we have to use total_cmp.
        assert_eq!(
            (f256::ZERO + f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::ZERO + f256::NEG_ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::NEG_ZERO + f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::NEG_ZERO + f256::NEG_ZERO).total_cmp(&f256::NEG_ZERO),
            Ordering::Equal
        );
        assert_eq!(f256::ONE + f256::ZERO, f256::ONE);
        assert_eq!(f256::ZERO + f256::ONE, f256::ONE);
        assert_eq!(f256::ONE + f256::NEG_ZERO, f256::ONE);
        assert_eq!(f256::NEG_ZERO + f256::ONE, f256::ONE);
    }

    #[test]
    fn test_normal() {
        assert_eq!(f256::ONE + f256::ONE, f256::TWO);
        assert_eq!(f256::ONE + f256::NEG_ONE, f256::ZERO);
        assert_eq!(f256::TWO + f256::TWO, f256::from(4.0));
        assert_eq!(f256::from(3.5) + f256::from(3.5), f256::from(7.0));
        assert_eq!(f256::from(3.5) + f256::from(-3.5), f256::ZERO);
        assert_eq!(f256::from(-3.5) + f256::from(-3.5), f256::from(-7.0));
        assert_eq!(f256::MAX + f256::MIN, f256::ZERO);
        assert_eq!(f256::MIN + f256::MAX, f256::ZERO);
        assert_eq!(f256::MAX + f256::EPSILON, f256::MAX);
        assert_eq!(f256::MIN + f256::EPSILON, f256::MIN);
        assert_eq!(f256::MAX + f256::MIN_GT_ZERO, f256::MAX);
        assert_eq!(f256::MIN + f256::MIN_GT_ZERO, f256::MIN);
    }

    #[test]
    fn test_subnormal_add_subnormal_giving_subnormal() {
        let x = f256::from_sign_exp_signif(0, -262378, (37538580480, 352));
        assert!(x.is_subnormal());
        let y = f256::from_sign_exp_signif(0, -262378, (17, 65003));
        assert!(y.is_subnormal());
        let z = f256::from_sign_exp_signif(0, -262378, (37538580497, 65355));
        assert!(z.is_subnormal());
        assert_eq!(x + y, z);
        assert_eq!(y + x, z);
    }

    #[test]
    fn test_subnormal_add_subnormal_giving_normal() {
        let x = f256::from_sign_exp_signif(
            0,
            -262378,
            (
                324518553658426726783156020576255,
                340282366920938463463374607431768199447,
            ),
        );
        assert!(x.is_subnormal());
        let y = f256::from_sign_exp_signif(
            0,
            -262378,
            (
                2535301200456458802993406410751,
                340282366920938463463374607431768211363,
            ),
        );
        assert!(y.is_subnormal());
        let z = f256::from_sign_exp_signif(
            0,
            -262377,
            (
                163526927429441592793074713493503,
                340282366920938463463374607431768205405,
            ),
        );
        assert!(z.is_normal());
        assert_eq!(x + y, z);
        assert_eq!(y + x, z);
    }

    #[test]
    fn test_normal_add_subnormal() {
        let x = f256::from_sign_exp_signif(
            0,
            -262376,
            (
                649037107316853453566312041152511,
                340282366920938463463374607431768187439,
            ),
        );
        assert!(x.is_normal());
        let y = f256::from_sign_exp_signif(
            0,
            -262378,
            (
                2535301200456458802993406410751,
                340282366920938463463374607431768211363,
            ),
        );
        assert!(y.is_subnormal());
        let z = f256::from_sign_exp_signif(
            0,
            -262374,
            (
                162338504991727627729171554238463,
                340282366920938463463374607431768205449,
            ),
        );
        assert!(z.is_normal());
        assert_eq!(x + y, z);
        assert_eq!(y + x, z);
    }

    #[test]
    fn test_min_gt_zero() {
        assert_eq!(f256::MAX + f256::MIN_GT_ZERO, f256::MAX);
        assert_eq!(f256::MIN + f256::MIN_GT_ZERO, f256::MIN);
        assert_eq!(f256::MIN_GT_ZERO + f256::MAX, f256::MAX);
        assert_eq!(f256::MIN_GT_ZERO + f256::MIN, f256::MIN);
        assert_eq!(f256::ONE + f256::MIN_GT_ZERO, f256::ONE);
        assert_eq!(f256::MIN_GT_ZERO + f256::ONE, f256::ONE);
    }

    #[test]
    fn test_overflow() {
        assert_eq!(f256::MAX + f256::MAX, f256::INFINITY);
        assert_eq!(f256::MAX + f256::ONE, f256::MAX);
    }

    #[test]
    fn test_signif_sum_overflow() {
        let x = f256::from_sign_exp_signif(
            0,
            -149851,
            (
                604778990250044258239859383006861,
                133457104047625401538330868155383882371,
            ),
        );
        let y = f256::from_sign_exp_signif(
            0,
            -149848,
            (
                57435753979957922695426646058213,
                329141037492710407041656814437889265703,
            ),
        );
        let z = f256::from_sign_exp_signif(
            0,
            -149849,
            (
                266066255522426909950818137868143,
                96152208885684853406990782909030131951,
            ),
        );
        assert_eq!(z, x + y);
    }

    #[test]
    fn test_signif_sum_overflow_2() {
        let x = f256::from_sign_exp_signif(
            0,
            -26672,
            (
                285795927001334733763948317049649,
                287350118037595727199178516752227046729,
            ),
        );
        let y = f256::from_sign_exp_signif(
            0,
            -26673,
            (
                512918941017936088634451700607106,
                37404434668644251012814945231532696663,
            ),
        );
        let z = f256::from_sign_exp_signif(
            0,
            -26670,
            (
                135563849377575694520293541838300,
                246654267303448694908083801057882454493,
            ),
        );
        assert_eq!(z, x + y);
    }

    #[test]
    fn test_subnormal_add_subnormal_giving_subnormal_2() {
        let x = f256::from_sign_exp_signif(
            1,
            -262378,
            (7509505897047126, 339876022494367207146865378540378746392),
        );
        let y = f256::from_sign_exp_signif(1, -262378, (0, 21747048302197486));
        let z = f256::from_sign_exp_signif(
            1,
            -262378,
            (7509505897047126, 339876022494367207146887125588680943878),
        );
        assert_eq!(z, x + y);
    }

    #[test]
    fn test_subnormal_add_subnormal_giving_subnormal_3() {
        let x = f256::from_sign_exp_signif(
            0,
            -262310,
            (
                152208704844451993242366249628516,
                324487274017150707511562659760330794299,
            ),
        );
        let y =
            f256::from_sign_exp_signif(1, -262378, (0, 37915000772644918358));
        let z = f256::from_sign_exp_signif(
            0,
            -262312,
            (
                608834819377807972969464998514067,
                277101995305787439656126816746018542827,
            ),
        );
        println!("{:?}", x.as_sign_exp_signif());
        println!("{:?}", y.as_sign_exp_signif());
        assert_eq!(z, x + y);
    }

    #[test]
    fn test_add_large_ints() {
        let x = f256::from_sign_exp_signif(
            0,
            105337,
            (
                602576129675788222002820055251844,
                82931759505431347843784813614315806051,
            ),
        );
        let y = f256::from_sign_exp_signif(
            0,
            105334,
            (
                601563861155012453246819040811600,
                201036493261246703061047998087190660657,
            ),
        );
        let z = f256::from_sign_exp_signif(
            0,
            105338,
            (
                338885806160082389329336217676647,
                54030660581543592863207906687607319317,
            ),
        );
        assert_eq!(z, x + y);
    }

    #[test]
    fn test_add_large_ints_2() {
        let x = f256::from_sign_exp_signif(
            0,
            15313,
            (
                460536143629997484487281743407796,
                158958087867975078094318880337509942897,
            ),
        );
        let y = f256::from_sign_exp_signif(
            0,
            15312,
            (
                128508114755699524999699245962620,
                42700807356084781598260591564231439767,
            ),
        );
        let z = f256::from_sign_exp_signif(
            0,
            15313,
            (
                524790201007847246987131366389106,
                180308491546017468893449176119625662780,
            ),
        );
        assert_eq!(z, x + y);
    }
}
