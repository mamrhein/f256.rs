// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod random_math_fn_tests {
    use std::path::PathBuf;

    use csv::ReaderBuilder;
    use serde::Deserialize;

    use f256::f256;

    #[derive(Debug, Deserialize)]
    struct Record {
        x: (u32, i32, u128, u128),
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

    fn run_tests(op: fn(&f256) -> f256, err: u32, file_name: &str) {
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
                    let z = f256::from_sign_exp_signif(
                        rec.z.0,
                        rec.z.1,
                        (rec.z.2, rec.z.3),
                    );
                    let res = op(&x);
                    if err == 0 {
                        assert_eq!(
                            res, z,
                            "\nFailed:\nx: {x:?}\nz: {z:?}\nr: {res:?}"
                        );
                    } else {
                        let d = (z - res).abs();
                        let m = f256::from(err) * z.ulp();
                        assert!(
                            d <= m,
                            "\nFailed:\nx: {x:e}\nz: {z:?}\nr: {res:?}\nd: \
                             {d:?}\nm: {m:?}"
                        );
                    }
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    fn sqrt(x: &f256) -> f256 {
        x.sqrt()
    }

    #[test]
    fn test_sqrt() {
        run_tests(sqrt, 0, "test_sqrt.txt");
    }

    #[test]
    fn test_sin_lt_2pi() {
        run_tests(f256::sin, 0, "test_sin_lt_2pi.txt");
    }

    #[test]
    fn test_cos_lt_2pi() {
        run_tests(f256::cos, 0, "test_cos_lt_2pi.txt");
    }

    #[test]
    fn test_sin_ge_2pi() {
        run_tests(f256::sin, 512, "test_sin_ge_2pi.txt");
    }

    #[test]
    fn test_cos_ge_2pi() {
        run_tests(f256::cos, 512, "test_cos_ge_2pi.txt");
    }
}
