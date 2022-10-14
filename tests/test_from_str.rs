// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod from_str_tests {
    use core::str::FromStr;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::from_str("nan").unwrap().is_nan());
        assert!(f256::from_str("-nan").unwrap().is_nan());
        assert!(f256::from_str("+nan").unwrap().is_nan());
        assert!(f256::from_str("Nan").unwrap().is_nan());
        assert!(f256::from_str("-Nan").unwrap().is_nan());
        assert!(f256::from_str("+Nan").unwrap().is_nan());
        assert!(f256::from_str("NaN").unwrap().is_nan());
        assert!(f256::from_str("-NaN").unwrap().is_nan());
        assert!(f256::from_str("+NaN").unwrap().is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::from_str("inf").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("+inf").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("-inf").unwrap(), f256::NEG_INFINITY);
        assert_eq!(f256::from_str("infinity").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("+infinity").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("-infinity").unwrap(), f256::NEG_INFINITY);
        assert_eq!(f256::from_str("Inf").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("+Inf").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("-Inf").unwrap(), f256::NEG_INFINITY);
        assert_eq!(f256::from_str("Infinity").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("+Infinity").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("-Infinity").unwrap(), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::from_str("0").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("+0").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("-0").unwrap(), f256::NEG_ZERO);
        assert_eq!(f256::from_str("0000000000").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("+0000000000").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("-0000000000").unwrap(), f256::NEG_ZERO);
        assert_eq!(f256::from_str("00.00000000").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("+0000.000000").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("-000000000.0").unwrap(), f256::NEG_ZERO);
    }

    #[test]
    fn test_exp_overflow() {
        let f = f256::from_str("12.5E78915").unwrap();
        assert_eq!(f, f256::INFINITY);
        let f = f256::from_str("-12.5E78915").unwrap();
        assert_eq!(f, f256::NEG_INFINITY);
    }

    #[test]
    fn test_exp_underflow() {
        let f = f256::from_str("12.5E-78983").unwrap();
        assert_eq!(f, f256::ZERO);
        let f = f256::from_str("-12.5E-78983").unwrap();
        assert_eq!(f, f256::ZERO);
    }

    #[test]
    fn test_normal_fast_path() {
        let f = f256::from_str("17.625").unwrap();
        assert_eq!(f.as_sign_exp_signif(), (0, -3, (0, 141)));
        let s =
            "-7.629394531250000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000e-06";
        let f = f256::from_str(s).unwrap();
        assert_eq!(f.as_sign_exp_signif(), (1, -17, (0, 1)));
        let s = "1234567890.1234567890123456789012345678901234567890";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                -204,
                (
                    93281312402337715824574088725497,
                    139132118994828259455133236546529242971
                )
            )
        );
        let s = "0.03978e-97";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                -563,
                (
                    352949761382803248364893811984677,
                    334535897686359893047635719463331157031
                )
            )
        );
    }

    #[test]
    fn test_fast_path_max_digits_exceeded() {
        let mut s = "1.".to_string();
        s.push_str(&*"0".repeat(93));
        s.push('9');
        let s = s.as_str();
        let f = f256::from_str(s).unwrap();
        assert_eq!(f.as_sign_exp_signif(), (0, 0, (0, 1)));
    }

    #[test]
    fn test_subnormal() {
        // TODO
    }

    #[test]
    fn test_err_empty_str() {
        let res = f256::from_str("");
        assert!(res.is_err());
    }

    #[test]
    fn test_err_invalid_lit() {
        let lits = [" ", "+", "-4.33.2", "2.87 e3", "+e3", ".4e3 "];
        for lit in lits {
            let res = f256::from_str(lit);
            assert!(res.is_err());
        }
    }
}
