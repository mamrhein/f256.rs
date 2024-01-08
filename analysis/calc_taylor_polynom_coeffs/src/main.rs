// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use clap::Parser;
use f256::f256;

const N: usize = 37;

fn print_as_array_elem_code(n: u32, coeff: &f256) {
    let (hi, lo) = coeff.to_bits();
    println!(
        "// {}1 / {}! ≈\n// {:e}\nf256::from_bits((0x{:032x}, 0x{:032x})),",
        ['+', '-'][coeff.is_sign_negative() as usize],
        n,
        coeff,
        hi,
        lo,
    );
}

fn print_as_const_array(name: &str, coeffs: &[(u32, f256); N]) {
    println!("const N: usize = {N}\n");
    println!("const {}: [f256; N] = [", name);
    for elem in coeffs.iter().rev() {
        let (n, coeff) = *elem;
        print_as_array_elem_code(n, &coeff);
    }
    println!("];");
}

fn calc_coeffs_for_sin() {
    let mut coeff = f256::ONE;
    let mut coeffs = [(1, coeff); N];
    for i in 1..N {
        let j = 2 * i as u32;
        let k = j + 1;
        coeff = &(-coeff) / &f256::from(j * k);
        coeffs[i] = (k, coeff);
    }
    print_as_const_array("COEFFS", &coeffs);
}

fn calc_coeffs_for_cos() {
    let mut coeff = f256::ONE;
    let mut coeffs = [(0, coeff); N];
    for i in 1..N {
        let j = 2 * i as u32;
        coeff = &(-coeff) / &f256::from((j - 1) * j);
        coeffs[i] = (j, coeff);
    }
    print_as_const_array("COEFFS", &coeffs);
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// circular function: sin cos tan cot
    #[arg(short, long)]
    func: String,
}

fn main() {
    let args = Args::parse();
    match args.func.as_str() {
        "sin" => calc_coeffs_for_sin(),
        "cos" => calc_coeffs_for_cos(),
        _ => panic!("not implemented yet"),
    }
}
