// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod from_random_str_tests {
    use core::str::FromStr;
    use std::path::PathBuf;

    use csv::ReaderBuilder;
    use f256::f256;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Record {
        lit: String,
        s: u8,
        e: i32,
        h: u128,
        l: u128,
    }

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

    fn run_tests(file_name: &str) {
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_path(get_path(file_name))
            .unwrap();
        for rec in rdr.deserialize::<Record>() {
            match rec {
                Ok(rec) => {
                    let res = f256::from_str(&rec.lit);
                    match res {
                        Ok(f) => {
                            let a = f.as_sign_exp_signif();
                            let b = (rec.s as u32, rec.e, (rec.h, rec.l));
                            assert_eq!(a, b, "\nparsed: {}\n", rec.lit);
                        }
                        Err(e) => panic!("{}: {}", e, rec.lit),
                    }
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    #[test]
    fn test_fast_exact() {
        run_tests("test_from_str_fast_exact.txt");
    }

    #[test]
    fn test_fast_approx() {
        run_tests("test_from_str_fast_approx.txt");
    }

    #[test]
    fn slowtest_normal() {
        run_tests("test_from_str_normal.txt");
    }

    #[test]
    fn slowtest_subnormal() {
        run_tests("test_from_str_subnormal.txt");
    }

    #[test]
    fn slowtest_extreme() {
        run_tests("test_from_str_extreme.txt");
    }
}
