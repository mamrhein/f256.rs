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
    use f256::f256;
    use serde::Deserialize;

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
                    assert!(
                        res.diff_within_n_bits(&z, err),
                        "\nFailed:\nx: {x:?}\nz: {z:?}\nr: {res:?}\nx: \
                             {x:e}\nz: {z:e}\nr: {res:e}"
                    );
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
    fn test_sin_fma_range() {
        run_tests(f256::sin, 0, "test_sin_fma_range.txt");
    }

    #[test]
    fn test_sin_large_values() {
        run_tests(f256::sin, 0, "test_sin_large_values.txt");
    }

    #[test]
    fn test_cos_lt_2pi() {
        run_tests(f256::cos, 0, "test_cos_lt_2pi.txt");
    }

    #[test]
    fn test_cos_fma_range() {
        run_tests(f256::cos, 0, "test_cos_fma_range.txt");
    }

    #[test]
    fn test_cos_large_values() {
        run_tests(f256::cos, 0, "test_cos_large_values.txt");
    }

    #[test]
    fn test_tan_lt_2pi() {
        run_tests(f256::tan, 1, "test_tan_lt_2pi.txt");
    }

    #[test]
    fn test_tan_fma_range() {
        run_tests(f256::tan, 2, "test_tan_fma_range.txt");
    }

    #[test]
    fn test_tan_large_values() {
        run_tests(f256::tan, 2, "test_tan_large_values.txt");
    }

    #[test]
    fn test_atan_lt_2pi() {
        run_tests(f256::atan, 0, "test_atan_lt_2pi.txt");
    }

    #[test]
    fn test_tan_mid_range() {
        run_tests(f256::atan, 0, "test_atan_mid_range.txt");
    }

    #[test]
    fn test_atan_large_values() {
        run_tests(f256::atan, 0, "test_atan_large_values.txt");
    }

    #[test]
    fn test_asin() {
        run_tests(f256::asin, 0, "test_asin.txt");
    }

    #[test]
    fn test_acos() {
        run_tests(f256::acos, 0, "test_acos.txt");
    }

    #[test]
    fn test_ln_normal() {
        run_tests(f256::ln, 0, "test_ln_normal.txt");
    }

    #[test]
    fn test_ln_subnormal() {
        run_tests(f256::ln, 0, "test_ln_subnormal.txt");
    }

    #[test]
    fn test_log2_normal() {
        run_tests(f256::log2, 0, "test_log2_normal.txt");
    }

    #[test]
    fn test_log2_subnormal() {
        run_tests(f256::log2, 0, "test_log2_subnormal.txt");
    }

    #[test]
    fn test_log10_normal() {
        run_tests(f256::log10, 0, "test_log10_normal.txt");
    }

    #[test]
    fn test_log10_subnormal() {
        run_tests(f256::log10, 0, "test_log10_subnormal.txt");
    }

    #[test]
    fn test_ln_1p() {
        run_tests(f256::ln_1p, 0, "test_ln_1p.txt");
    }

    #[test]
    fn test_exp() {
        run_tests(f256::exp, 0, "test_exp.txt");
    }

    #[test]
    fn test_exp_m1() {
        run_tests(f256::exp_m1, 0, "test_exp_m1.txt");
    }

    #[test]
    fn test_exp2() {
        run_tests(f256::exp2, 0, "test_exp2.txt");
    }
}
