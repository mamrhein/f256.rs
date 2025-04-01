// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod random_powi_tests {
    use std::path::PathBuf;

    use csv::ReaderBuilder;
    use f256::f256;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Record {
        x: (u32, i32, u128, u128),
        y: i32,
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

    fn run_tests(op: fn(&f256, i32) -> f256, file_name: &str) {
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
                    let y = rec.y;
                    let z = f256::from_sign_exp_signif(
                        rec.z.0,
                        rec.z.1,
                        (rec.z.2, rec.z.3),
                    );
                    assert_eq!(op(&x, y), z, "\nFailed: {rec:?}");
                }
                Err(e) => panic!("{}", e),
            }
        }
    }

    fn powi<'a>(x: &'a f256, y: i32) -> f256 {
        x.powi(y)
    }

    #[test]
    fn test_powi() {
        run_tests(powi, "test_powi.txt");
    }
}
