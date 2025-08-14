// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

mod common;
mod dec_repr;
mod formatted;
mod powers_of_five;
mod to_fixed_prec;

use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
};
use core::{
    cmp::min,
    fmt::{self, Display, Write},
};

use dec_repr::DecNumRepr;
use to_fixed_prec::{bin_2_dec_fixed_point, bin_2_dec_scientific};

use crate::{f256, split_f256_enc, EXP_MAX, U256};

const MAX_PREC: usize = 75;

fn format_nan(form: &mut fmt::Formatter<'_>) -> fmt::Result {
    let nan = "NaN".to_owned();
    let s = if let Some(width) = form.width() {
        match form.align() {
            Some(fmt::Alignment::Center) => format!("{:^width$}", nan),
            Some(fmt::Alignment::Left) => format!("{:<width$}", nan),
            _ => format!("{:>width$}", nan),
        }
    } else {
        nan
    };
    form.write_str(s.as_str())
}

fn format_special(f: &f256, form: &mut fmt::Formatter<'_>) -> fmt::Result {
    if f.eq_zero() {
        let prec = min(form.precision().unwrap_or(0), MAX_PREC);
        let s = format!("{:.*}", prec, 0.);
        form.pad_integral(f.is_sign_positive(), "", s.as_str())
    } else if f.is_nan() {
        format_nan(form)
    } else {
        form.pad_integral(f.is_sign_positive(), "", "inf")
    }
}

#[inline]
fn format_exact(
    f: &f256,
    prec: usize,
    form: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    assert!(
        prec <= MAX_PREC,
        "Maximum precision exceeded: {prec} > {MAX_PREC}."
    );
    let s = bin_2_dec_fixed_point(f.abs(), prec);
    let start = s.starts_with('0') as usize;
    form.pad_integral(f.is_sign_positive(), "", &s[start..])
}

#[inline]
fn format_shortest(f: &f256, form: &mut fmt::Formatter<'_>) -> fmt::Result {
    debug_assert!(f.is_finite() && !f.eq_zero());
    let d = DecNumRepr::shortest_from_f256(f);
    d.fmt(form)
}

impl fmt::Display for f256 {
    /// Formats the value using the given formatter.
    ///
    /// Panics:
    /// -------
    /// * The given precision exceeds 75.
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_special() {
            format_special(self, form)
        } else {
            match form.precision() {
                Some(prec) => format_exact(self, prec, form),
                None => format_shortest(self, form),
            }
        }
    }
}

impl fmt::Debug for f256 {
    #[allow(clippy::cast_possible_wrap)]
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (sign, exp, signif) = split_f256_enc(self);
        if exp == EXP_MAX as i32 {
            fmt::Debug::fmt(&self.to_string(), form)
        } else {
            fmt::Debug::fmt(&(sign, exp, signif), form)
        }
    }
}

fn format_scientific_special(
    f: &f256,
    exp_mark: char,
    form: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    if f.eq_zero() {
        let prec = min(form.precision().unwrap_or(0), MAX_PREC);
        let s = match exp_mark {
            'e' => format!("{:.*e}", prec, 0.),
            'E' => format!("{:.*E}", prec, 0.),
            _ => unreachable!(),
        };
        form.pad_integral(f.is_sign_positive(), "", s.as_str())
    } else if f.is_nan() {
        format_nan(form)
    } else {
        form.pad_integral(f.is_sign_positive(), "", "inf")
    }
}

#[inline]
fn format_scientific_common(
    f: &f256,
    exp_mark: char,
    form: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    if f.is_special() {
        format_scientific_special(f, exp_mark, form)
    } else {
        match form.precision() {
            Some(prec) => format_scientific_exact(f, exp_mark, prec, form),
            None => format_scientific_shortest(f, exp_mark, form),
        }
    }
}

#[inline]
fn format_scientific_exact(
    f: &f256,
    exp_mark: char,
    prec: usize,
    form: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    assert!(
        prec <= MAX_PREC,
        "Maximum precision for scientific format exceeded: {prec} > \
         {MAX_PREC}."
    );
    let s = bin_2_dec_scientific(f.abs(), exp_mark, prec);
    form.pad_integral(f.is_sign_positive(), "", &s)
}

#[inline]
fn format_scientific_shortest(
    f: &f256,
    exp_mark: char,
    form: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    debug_assert!(f.is_finite() && !f.eq_zero());
    let d = DecNumRepr::shortest_from_f256(f);
    d.fmt_scientific(exp_mark, form)
}

impl fmt::LowerExp for f256 {
    /// Formats the value using the given formatter in scientific notation
    /// with a lower-case `e`.
    ///
    /// Panics:
    /// -------
    /// * The given precision exceeds 75.
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_scientific_common(self, 'e', form)
    }
}

impl fmt::UpperExp for f256 {
    /// Formats the value using the given formatter in scientific notation
    /// with a lower-case `E`.
    ///
    /// Panics:
    /// -------
    /// * The given precision exceeds 75.
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_scientific_common(self, 'E', form)
    }
}

#[cfg(test)]
mod display_tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    #[should_panic]
    fn test_max_prec_exceeded() {
        let f = f256::ONE;
        let _ = format!("{f:.76}");
    }

    #[test]
    fn test_zero() {
        let f = f256::ZERO;
        assert_eq!(format!("{f}"), "0");
        assert_eq!(format!("{f:+10.3}"), "    +0.000");
        assert_eq!(format!("{f:07}"), "0000000");
        assert_eq!(format!("{f:>7}"), "      0");
        assert_eq!(format!("{f:<7}"), "0      ");
        assert_eq!(format!("{f:^7}"), "   0   ");
    }

    #[test]
    fn test_neg_zero() {
        let f = f256::NEG_ZERO;
        assert_eq!(format!("{f}"), "-0");
        assert_eq!(format!("{f:+10.3}"), "    -0.000");
        assert_eq!(format!("{f:07}"), "-000000");
        assert_eq!(format!("{f:>9.1}"), "     -0.0");
        assert_eq!(format!("{f:<9.1}"), "-0.0     ");
        assert_eq!(format!("{f:^9.1}"), "  -0.0   ");
    }

    #[test]
    fn test_nan() {
        let f = f256::NAN;
        assert_eq!(format!("{f}"), "NaN");
        assert_eq!(format!("{f:+10.3}"), "       NaN");
        assert_eq!(format!("{f:07}"), "    NaN");
        assert_eq!(format!("{f:>7}"), "    NaN");
        assert_eq!(format!("{f:<7}"), "NaN    ");
        assert_eq!(format!("{f:^7}"), "  NaN  ");
    }

    #[test]
    fn test_inf() {
        let f = f256::INFINITY;
        assert_eq!(format!("{f}"), "inf");
        assert_eq!(format!("{f:+5.2}"), " +inf");
        assert_eq!(format!("{f:^7}"), "  inf  ");
    }

    #[test]
    fn test_neg_inf() {
        let f = f256::NEG_INFINITY;
        assert_eq!(format!("{f}"), "-inf");
        assert_eq!(format!("{f:>12.7}"), "        -inf");
        assert_eq!(format!("{f:<+.0}"), "-inf");
    }

    #[test]
    fn test_shortest_one() {
        let f = f256::ONE;
        assert_eq!(format!("{f}"), "1");
        assert_eq!(format!("{f:>10}"), "         1");
        assert_eq!(format!("{f:<+}"), "+1");
        assert_eq!(format!("{f:^+7.}"), "  +1   ");
    }

    #[test]
    fn test_shortest_one_tenth() {
        let f = f256::from_str("-0.1").unwrap();
        assert_eq!(format!("{f}"), "-0.1");
        assert_eq!(format!("{f}"), "-0.1");
        assert_eq!(format!("{f:>10}"), "      -0.1");
        assert_eq!(format!("{f:<+}"), "-0.1");
        assert_eq!(format!("{f:^+7.}"), " -0.1  ");
    }

    #[test]
    fn test_shortest_one_half() {
        let f = f256::encode(0, -1, U256::new(0, 1));
        assert_eq!(format!("{f}"), "0.5");
        assert_eq!(format!("{f:3}"), "0.5");
        assert_eq!(format!("{f:_>4.}"), "_0.5");
        assert_eq!(format!("{f:~^8}"), "~~0.5~~~");
    }

    #[test]
    fn test_shortest_one_third() {
        let f = f256::ONE / f256::from_u64(3);
        assert_eq!(
            format!("{f}"),
            "0.333333333333333333333333333333333333333333333333333333333\
                   333333333333333"
        );
        assert_eq!(
            format!("{f:70}"),
            "0.333333333333333333333333333333333333333333333333333333333\
                   333333333333333"
        );
        assert_eq!(
            format!("{f:_>75.}"),
            "_0.33333333333333333333333333333333333333333333333333333333\
                   3333333333333333"
        );
        assert_eq!(
            format!("{f:~^80}"),
            "~~~0.333333333333333333333333333333333333333333333333333333\
                   333333333333333333~~~"
        );
    }

    #[test]
    fn test_shortest_normal_gt1() {
        let f = f256::from_str("320.1000009").unwrap();
        assert_eq!(format!("{f}"), "320.1000009");
    }

    #[test]
    fn test_shortest_normal_near_zero() {
        let f = f256::from_str("1.000009e-82").unwrap();
        assert_eq!(
            format!("{f}"),
            "0.000000000000000000000000000000000000000000000000000000000\
                   0000000000000000000000001000009"
        );
    }

    #[test]
    fn test_shortest_min_positive() {
        let f = f256::MIN_POSITIVE;
        let s = format!("{f}");
        let tail =
            "000248242795146434978829932822291387172367768770607964686927095329\
            791378756";
        assert_eq!(s.len(), 78986);
        assert!(s.ends_with(tail));
        let g = f256::from_str(&s).unwrap();
        assert_eq!(f, g);
    }

    #[test]
    fn test_shortest_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        let s = format!("{f}");
        let tail =
            "00000000000000000000000000000000000000000000000000000000000000002";
        assert_eq!(s.len(), 78986);
        assert!(s.ends_with(tail));
        let g = f256::from_str(&s).unwrap();
        assert_eq!(f, g);
    }

    #[test]
    fn test_shortest_normal_near_ten_pow_70() {
        let f = f256::from_str("-1.004809e70").unwrap();
        let s = format!("{f}");
        assert_eq!(
            s,
            "-1004809000000000000000000000000000000000000000000000000000\
                   0000000000000"
        );
        let g = f256::from_str(&s).unwrap();
        assert_eq!(f, g);
    }

    #[test]
    fn test_shortest_normal_near_3_times_ten_pow_72() {
        let f = f256::from_sign_exp_signif(
            0,
            5,
            (
                324518553658426726783156020576278_u128,
                181827047980905777921517454682111627119_u128,
            ),
        );
        let s = format!("{f}");
        assert_eq!(
            s,
            "35336941295567686591665950014860824089068155234144096205253\
                   94194978172380"
        );
        let g = f256::from_str(&s).unwrap();
        assert_eq!(
            f,
            g,
            "\nf: {}\n   {:?}\ng: {}\n   {:?}\n",
            &s,
            f.as_sign_exp_signif(),
            &g.to_string(),
            g.as_sign_exp_signif()
        );
        let h = f256::from_str(
            "35336941295567686591665950014860824089068155234144096205253\
                   94194978172384",
        )
        .unwrap();
        assert_eq!(
            f,
            h,
            "\nf: {}\n   {:?}\nh: {}\n   {:?}\n",
            &s,
            f.as_sign_exp_signif(),
            &h.to_string(),
            h.as_sign_exp_signif()
        );
    }

    #[test]
    fn test_shortest_normal_near_ten_pow_73() {
        let f = f256::from_sign_exp_signif(
            0,
            4,
            (
                162259276829213363391578010288177_u128,
                287198669047081428097154997580432370835_u128,
            ),
        );
        let s = format!("{f}");
        assert_eq!(
            s,
            "88342353238919216479164875037173063446811271749601431923299\
                   0893842024750"
        );
        let g = f256::from_str(&s).unwrap();
        assert_eq!(
            f,
            g,
            "\nf: {}\n   {:?}\ng: {}\n   {:?}\n",
            &s,
            f.as_sign_exp_signif(),
            &g.to_string(),
            g.as_sign_exp_signif()
        );
    }

    #[test]
    fn test_shortest_f256_max() {
        let f = f256::MAX;
        let s = format!("{f}");
        let head =
            "16113257174857604736195721184520050106440238745496695174763\
            7125049607183000";
        assert_eq!(s.len(), 78914);
        assert_eq!(&s[..head.len()], head);
        let g = f256::from_str(&s).unwrap();
        assert_eq!(f, g);
    }

    #[test]
    fn test_fixed_prec_one() {
        let f = f256::ONE;
        assert_eq!(format!("{f:.2}"), "1.00");
        assert_eq!(format!("{f:>10.0}"), "         1");
        assert_eq!(format!("{f:<+.3}"), "+1.000");
        assert_eq!(
            format!("{f:^+78.72}"),
            " +1.0000000000000000000000000000000000000000000000000000000\
                   00000000000000000  "
        );
    }

    #[test]
    fn test_fixed_prec_one_tenth() {
        let f = f256::from_str("-0.1").unwrap();
        assert_eq!(format!("{f:.2}"), "-0.10");
        assert_eq!(format!("{f:.0}"), "-0");
        assert_eq!(
            format!("{f:>35.24}"),
            "        -0.100000000000000000000000"
        );
        assert_eq!(format!("{f:<+7.2}"), "-0.10  ");
        assert_eq!(format!("{f:^+7.4}"), "-0.1000");
    }

    #[test]
    fn test_fixed_prec_one_half() {
        let f = f256::encode(0, -1, U256::new(0, 1));
        assert_eq!(format!("{f:>5.0}"), "    0");
        assert_eq!(format!("{f:.3}"), "0.500");
        assert_eq!(format!("{f:_>4.1}"), "_0.5");
        assert_eq!(format!("{f:~^8.2}"), "~~0.50~~");
    }

    #[test]
    fn test_fixed_prec_one_third() {
        let f = f256::ONE / f256::from_u64(3);
        assert_eq!(format!("{f:.3}"), "0.333");
        assert_eq!(
            format!("{f:70.75}"),
            "0.333333333333333333333333333333333333333333333333333333333\
                   333333333333332579"
        );
        assert_eq!(
            format!("{f:_>75.70}"),
            "___0.333333333333333333333333333333333333333333333333333333\
                   3333333333333333"
        );
        assert_eq!(
            format!("{f:~^80.57}"),
            "~~~~~~~~~~0.33333333333333333333333333333333333333333333333\
                   3333333333~~~~~~~~~~~"
        );
    }

    #[test]
    fn test_fixed_prec_normal_gt1() {
        let f = f256::from_str("320.100000907").unwrap();
        assert_eq!(format!("{f:.7}"), "320.1000009");
    }

    #[test]
    fn test_fixed_prec_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        let s = format!("{f}");
        let tail =
            "00000000000000000000000000000000000000000000000000000000000000002";
        assert_eq!(s.len(), 78986);
        assert!(s.ends_with(tail));
        let g = f256::from_str(&s).unwrap();
        assert_eq!(f, g);
    }

    #[test]
    fn test_fixed_prec_normal_near_ten_pow_70() {
        let f = f256::from_str("-1.004809e70").unwrap();
        assert_eq!(
            format!("{f:.21}"),
            "-1004809000000000000000000000000000000000000000000000000000\
                   0000000000000.000000000000000000000"
        );
    }

    #[test]
    fn test_near_1e53() {
        let f = f256::from_sign_exp_signif(
            0,
            -60,
            (
                444818555402263217458454931195100,
                215969272825691358654988435264388805316,
            ),
        );
        let s = format!("{f:.5}");
        assert_eq!(
            s,
            "131287264811883885796002000348253624778443062443856017.28632"
                .to_owned()
        );
    }
}

#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert_eq!(
            format!("{:?}", f256::ZERO),
            "(0, 0, (0x00000000000000000000000000000000, \
             0x00000000000000000000000000000000))"
        );
        assert_eq!(
            format!("{:?}", f256::NEG_ZERO),
            "(1, 0, (0x00000000000000000000000000000000, \
             0x00000000000000000000000000000000))"
        );
    }

    #[test]
    fn test_nan() {
        let f = f256::NAN;
        assert_eq!(format!("{f:?}"), "\"NaN\"");
    }

    #[test]
    fn test_inf() {
        let f = f256::INFINITY;
        assert_eq!(format!("{f:?}"), "\"inf\"");
    }

    #[test]
    fn test_neg_inf() {
        let f = f256::NEG_INFINITY;
        assert_eq!(format!("{f:?}"), "\"-inf\"");
    }

    #[test]
    fn test_normal() {
        assert_eq!(
            format!("{:?}", f256::EPSILON),
            "(0, -472, (0x00001000000000000000000000000000, \
             0x00000000000000000000000000000000))"
        );
    }

    #[test]
    fn test_subnormal() {
        assert_eq!(
            format!("{:?}", f256::MIN_GT_ZERO),
            "(0, -262378, (0x00000000000000000000000000000000, \
             0x00000000000000000000000000000001))"
        );
        let f = f256::MIN_POSITIVE - f256::MIN_GT_ZERO;
        assert_eq!(
            format!("{:?}", f),
            "(0, -262378, (0x00000fffffffffffffffffffffffffff, \
             0xffffffffffffffffffffffffffffffff))"
        );
    }
}

#[cfg(test)]
mod format_exp_tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    #[should_panic]
    fn test_max_prec_exceeded() {
        let f = f256::ONE;
        let _ = format!("{f:.83e}");
    }

    #[test]
    fn test_zero() {
        let f = f256::ZERO;
        assert_eq!(format!("{f:e}"), "0e0");
        assert_eq!(format!("{f:+10.3e}"), "  +0.000e0");
        assert_eq!(format!("{f:07E}"), "00000E0");
        assert_eq!(format!("{f:>7E}"), "    0E0");
        assert_eq!(format!("{f:<7e}"), "0e0    ");
        assert_eq!(format!("{f:^7e}"), "  0e0  ");
    }

    #[test]
    fn test_neg_zero() {
        let f = f256::NEG_ZERO;
        assert_eq!(format!("{f:E}"), "-0E0");
        assert_eq!(format!("{f:+12.3e}"), "    -0.000e0");
        assert_eq!(format!("{f:07e}"), "-0000e0");
        assert_eq!(format!("{f:>9.1e}"), "   -0.0e0");
        assert_eq!(format!("{f:<9.1e}"), "-0.0e0   ");
        assert_eq!(format!("{f:^9.1e}"), " -0.0e0  ");
    }

    #[test]
    fn test_nan() {
        let f = f256::NAN;
        assert_eq!(format!("{f:e}"), "NaN");
        assert_eq!(format!("{f:+10.3e}"), "       NaN");
        assert_eq!(format!("{f:07e}"), "    NaN");
        assert_eq!(format!("{f:>7e}"), "    NaN");
        assert_eq!(format!("{f:<7e}"), "NaN    ");
        assert_eq!(format!("{f:^7e}"), "  NaN  ");
    }

    #[test]
    fn test_inf() {
        let f = f256::INFINITY;
        assert_eq!(format!("{f:e}"), "inf");
        assert_eq!(format!("{f:+5.2E}"), " +inf");
        assert_eq!(format!("{f:^7e}"), "  inf  ");
    }

    #[test]
    fn test_neg_inf() {
        let f = f256::NEG_INFINITY;
        assert_eq!(format!("{f:E}"), "-inf");
        assert_eq!(format!("{f:>12.7e}"), "        -inf");
        assert_eq!(format!("{f:<+.0e}"), "-inf");
    }

    #[test]
    fn test_shortest_one() {
        let f = f256::ONE;
        assert_eq!(format!("{f:e}"), "1e0");
        assert_eq!(format!("{f:>10e}"), "       1e0");
        assert_eq!(format!("{f:<+e}"), "+1e0");
        assert_eq!(format!("{f:^+7.e}"), " +1e0  ");
    }

    #[test]
    fn test_shortest_one_tenth() {
        let f = f256::from_str("-0.1e0").unwrap();
        assert_eq!(format!("{f:e}"), "-1e-1");
        assert_eq!(format!("{f:>10e}"), "     -1e-1");
        assert_eq!(format!("{f:<+e}"), "-1e-1");
        assert_eq!(format!("{f:^+6.e}"), "-1e-1 ");
    }

    #[test]
    fn test_shortest_one_half() {
        let f = f256::encode(0, -1, U256::new(0, 1));
        assert_eq!(format!("{f:e}"), "5e-1");
        assert_eq!(format!("{f:3e}"), "5e-1");
        assert_eq!(format!("{f:_>7.e}"), "___5e-1");
        assert_eq!(format!("{f:~^8e}"), "~~5e-1~~");
    }

    #[test]
    fn test_shortest_normal_gt1() {
        let f = f256::from_str("320.1000009").unwrap();
        assert_eq!(format!("{f:e}"), "3.201000009e2");
    }

    #[test]
    fn test_shortest_normal_near_zero() {
        let f = f256::from_str("1.000009e-82").unwrap();
        assert_eq!(format!("{f:e}"), "1.000009e-82");
    }

    #[test]
    fn test_shortest_min_positive() {
        let f = f256::MIN_POSITIVE;
        let s = "2.48242795146434978829932822291387172367768770607964686927095329791378756e-78913";
        assert_eq!(format!("{f:e}"), s);
        let g = f256::from_str(s).unwrap();
        assert_eq!(f, g);
    }

    #[test]
    fn test_shortest_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        let s = "2e-78984";
        assert_eq!(format!("{f:e}"), s);
        let g = f256::from_str(s).unwrap();
        assert_eq!(f, g);
    }

    #[test]
    fn test_shortest_normal_near_ten_pow_70() {
        let f = f256::from_str("-1.00480900e70").unwrap();
        assert_eq!(format!("{f:e}"), "-1.004809e70");
    }

    #[test]
    fn test_shortest_normal_five_times_ten_pow_75() {
        let f = f256::from_str("5e75").unwrap();
        assert_eq!(format!("{f:e}"), "5e75");
    }

    #[test]
    fn test_shortest_f256_max() {
        let f = f256::MAX;
        assert_eq!(
            format!("{f:e}"),
            "1.6113257174857604736195721184520050106440238745496695174763712504\
            9607183e78913");
    }

    #[test]
    fn test_fixed_prec_one_half() {
        let f = f256::from_str("0.5").unwrap();
        assert_eq!(format!("{f:.3e}"), "5.000e-1");
    }

    #[test]
    fn test_fixed_prec_one_tenth() {
        let f = f256::from_str("-0.1e0").unwrap();
        assert_eq!(format!("{f:.2e}"), "-1.00e-1");
    }

    #[test]
    fn test_fixed_prec_epsilon() {
        let f = -f256::EPSILON;
        assert_eq!(format!("{f:^20.6e}"), "   -9.055679e-72    ");
    }

    #[test]
    fn test_fixed_prec_2_pow_250() {
        let f = f256::from_str("5.527147875260445e-76").unwrap();
        assert_eq!(format!("{f:.8e}"), "5.52714788e-76");
    }

    #[test]
    fn test_fixed_prec_f256_min_positive() {
        let f = f256::MIN_POSITIVE;
        assert_eq!(format!("{f:.4e}"), "2.4824e-78913");
    }

    #[test]
    fn test_fixed_prec_f256_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(format!("{f:.5e}"), "2.24801e-78984");
    }

    #[test]
    fn test_fixed_prec_ninetynine_and_a_half() {
        let f = f256::from_str("99.5").unwrap();
        assert_eq!(format!("{f:.0e}"), "1e2");
    }

    #[test]
    fn test_fixed_prec_10_pow_305() {
        let f = f256::from_str("7.8392e305").unwrap();
        assert_eq!(format!("{f:.2e}"), "7.84e305");
    }

    #[test]
    fn test_fixed_prec_near_1e71() {
        let f =
            f256::from_str("-1.10427941548649020598956093796446e71").unwrap();
        assert_eq!(format!("{f:.8e}"), "-1.10427942e71");
    }

    #[test]
    fn test_fixed_prec_10_pow_22093() {
        let f = f256::from_str("5.900065e22093").unwrap();
        assert_eq!(format!("{f:.5e}"), "5.90007e22093");
    }

    #[test]
    fn test_1e153() {
        let f = f256::from_str("-1e153").unwrap();
        let s = format!("{f:.0e}");
        assert_eq!(s, "-1e153");
    }

    #[test]
    fn test_fixed_prec_f256_max() {
        let f = f256::MAX;
        assert_eq!(format!("{f:.4e}"), "1.6113e78913");
    }

    #[test]
    fn test_2e_minus_78888() {
        let f = f256::from_str("-2e-78888").unwrap();
        let s = format!("{f:.0e}");
        assert_eq!(s, "-2e-78888");
    }
}
