// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod random_bin_op_tests {
    use std::path::PathBuf;

    use csv::ReaderBuilder;
    use f256::f256;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Record {
        x: (u32, i32, u128, u128),
        y: (u32, i32, u128, u128),
        z: (u32, i32, u128, u128),
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

    fn run_tests(op: fn(&f256, &f256) -> f256, file_name: &str) {
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_path(get_path(file_name))
            .unwrap();
        for rec in rdr.deserialize::<Record>() {
            match rec {
                Ok(rec) => {
                    let x = f256::from_sign_exp_signif(
                        rec.x.0,
                        rec.x.1,
                        (rec.x.2, rec.x.3),
                    );
                    assert!(x.is_finite(), "\nx not finite: {rec:?}");
                    let y = f256::from_sign_exp_signif(
                        rec.y.0,
                        rec.y.1,
                        (rec.y.2, rec.y.3),
                    );
                    assert!(y.is_finite(), "\ny not finite: {rec:?}");
                    let z = f256::from_sign_exp_signif(
                        rec.z.0,
                        rec.z.1,
                        (rec.z.2, rec.z.3),
                    );
                    assert_eq!(op(&x, &y), z, "\nFailed: {rec:?}");
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    fn add<'a>(x: &'a f256, y: &'a f256) -> f256 {
        x + y
    }

    #[test]
    fn test_add_sub() {
        run_tests(add, "test_add_sub.txt");
    }

    fn mul<'a>(x: &'a f256, y: &'a f256) -> f256 {
        x * y
    }

    #[test]
    fn test_mul() {
        run_tests(mul, "test_mul.txt");
    }

    fn div<'a>(x: &'a f256, y: &'a f256) -> f256 {
        x / y
    }

    #[test]
    fn test_div() {
        run_tests(div, "test_div.txt");
    }

    fn rem<'a>(x: &'a f256, y: &'a f256) -> f256 {
        x % y
    }

    #[test]
    fn test_rem() {
        run_tests(rem, "test_rem.txt");
    }

    fn sos<'a>(x: &'a f256, y: &'a f256) -> f256 {
        x.sum_of_squares(*y)
    }

    #[test]
    fn test_sos() {
        run_tests(sos, "test_sos.txt");
    }
}
