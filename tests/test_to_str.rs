// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use std::{path::PathBuf, str::FromStr};

use csv::ReaderBuilder;
use f256::f256;
use serde::Deserialize;

fn get_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("tests");
    dir
}

fn get_path(file_name: &str) -> PathBuf {
    let mut p = get_dir();
    p.push(file_name);
    p
}

fn run_tests<Record: for<'a> Deserialize<'a>>(
    file_name: &str,
    do_test: &dyn Fn(&Record),
) {
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_path(get_path(file_name))
        .unwrap();
    for rec in rdr.deserialize::<Record>() {
        match rec {
            Ok(rec) => {
                do_test(&rec);
            }
            Err(e) => panic!("{}", e),
        }
    }
}

#[cfg(test)]
mod random_f256_to_shortest_str_tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct Record {
        s: u32,
        e: i32,
        h: u128,
        l: u128,
        lit: String,
    }

    fn do_test(rec: &Record) {
        let f = f256::from_sign_exp_signif(rec.s, rec.e, (rec.h, rec.l));
        let s = f.to_string();
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
        let n = s.len();
        if n < rec.lit.len() {
            assert!(
                rec.lit.starts_with(&s[..n - 1]),
                "\nlit: {}\nstr: {}\n",
                &*rec.lit,
                &s
            );
        } else {
            assert_eq!(
                rec.lit[..60],
                s[..60],
                "\nlit: {}\nstr: {}\n",
                &*rec.lit,
                &s
            );
        }
    }

    #[test]
    fn test_small_float() {
        run_tests::<Record>("test_to_str_small_float_shortest.txt", &do_test);
    }

    #[test]
    fn test_small_int() {
        run_tests::<Record>("test_to_str_small_int_shortest.txt", &do_test);
    }

    #[test]
    fn slowtest_large_int() {
        run_tests::<Record>("test_to_str_large_int_shortest.txt", &do_test);
    }
}

#[cfg(test)]
mod random_f256_to_shortest_exp_tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct Record {
        s: u32,
        e: i32,
        h: u128,
        l: u128,
        lit: String,
    }

    fn do_test(rec: &Record) {
        let f = f256::from_sign_exp_signif(rec.s, rec.e, (rec.h, rec.l));
        let s = format!("{f:e}");
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
        let mut n = s.len();
        if n < rec.lit.len() {
            let v: Vec<&str> = s.split('e').collect();
            assert!(v.len() == 2);
            n = *&v[0].len();
            assert!(
                rec.lit.starts_with(&v[0][..n - 1]),
                "\nlit: {}\nstr: {}\n",
                &*rec.lit,
                &s
            );
            assert!(
                rec.lit.ends_with(&v[1]),
                "\nlit: {}\nstr: {}\n",
                &*rec.lit,
                &s
            );
        } else {
            assert_eq!(rec.lit, s, "\nlit: {}\nstr: {}\n", &*rec.lit, &s);
        }
    }

    #[test]
    fn slowtest_fract() {
        run_tests::<Record>("test_to_str_fract_shortest_exp.txt", &do_test);
    }

    #[test]
    fn slowtest_subnormal() {
        run_tests::<Record>("test_to_str_subnormal_shortest_exp.txt", &do_test);
    }
}

#[cfg(test)]
mod random_f256_to_fixed_prec_exp_tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    #[repr(C)]
    struct Record {
        s: u32,
        e: i32,
        h: u128,
        l: u128,
        p: usize,
        lit: String,
    }

    fn do_test(rec: &Record) {
        let f = f256::from_sign_exp_signif(rec.s, rec.e, (rec.h, rec.l));
        let p = rec.p;
        let s = format!("{f:.*e}", p);
        assert_eq!(rec.lit, s, "\nlit: {}\nstr: {}\n", &*rec.lit, &s);
    }

    #[test]
    fn test_small_float() {
        run_tests::<Record>(
            "test_to_str_small_float_fixed_prec_exp.txt",
            &do_test,
        );
    }

    #[test]
    fn test_small_int() {
        run_tests::<Record>(
            "test_to_str_small_int_fixed_prec_exp.txt",
            &do_test,
        );
    }

    #[test]
    fn test_fract() {
        run_tests::<Record>("test_to_str_fract_fixed_prec_exp.txt", &do_test);
    }

    #[test]
    fn test_large_int() {
        run_tests::<Record>(
            "test_to_str_large_int_fixed_prec_exp.txt",
            &do_test,
        );
    }

    #[test]
    fn test_subnormal() {
        run_tests::<Record>(
            "test_to_str_subnormal_fixed_prec_exp.txt",
            &do_test,
        );
    }
}
