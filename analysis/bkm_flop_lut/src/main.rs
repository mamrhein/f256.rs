use std::ops::{Neg, Shl};

use rug::{ops::CompleteRound, Complete, Float, Integer};

const P: u32 = 511;

fn print_repr(f: &Float, t: &str) {
    let (m, mut e) = f.to_integer_exp().unwrap();
    e += P as i32 - 1;
    debug_assert_eq!(m.significant_bits(), P);
    let b: Integer = Integer::from(1).shl(128);
    let mut q = m.clone();
    let mut s = "".to_string();
    for _ in 0..4 {
        let r = (&q % &b).complete();
        s = format!("0x{:032x},\n", r) + &s;
        q /= &b;
    }
    println!("Float512::new(1, {e}, (");
    println!("{}),){}", s, t);
}

fn bkm_lut(n: u32) {
    let one = Float::with_val(P, 1);
    for i in 0..n {
        let k = (i as i32).neg();
        let f = one.clone() + Float::with_val(P, k).exp2();
        let l = f.ln();
        println!("// n = {i}");
        println!("// l = {l}");
        print_repr(&l, ",");
    }
}

fn log_consts() {
    let ln_2 = Float::with_val(P, 2).ln();
    println!("// LN_2 = ◯₅₁₀(logₑ(2)) =");
    println!("// {ln_2}");
    print!("pub(crate) const LN_2: Float512 = ");
    print_repr(&ln_2, ";");
    let ln_10 = Float::with_val(P, 10).ln();
    println!("// LN_10 = ◯₅₁₀(logₑ(10)) =");
    println!("// {ln_10}");
    print!("pub(crate) const LN_10: Float512 = ");
    print_repr(&ln_10, ";");
    #[allow(non_snake_case)]
    let E = Float::with_val(2 * P, 1).exp();
    let log2_e = E.log2_ref().complete(P);
    println!("// LOG2_E = ◯₅₁₀(log₂(E)) =");
    println!("// {log2_e}");
    print!("pub(crate) const LOG2_E: Float512 = ");
    print_repr(&log2_e, ";");
    let log10_e = E.log10_ref().complete(P);
    println!("// LOG10_E = ◯₅₁₀(log₁₀(E)) =");
    println!("// {log10_e}");
    print!("pub(crate) const LOG10_E: Float512 = ");
    print_repr(&log10_e, ";");
}

fn main() {
    bkm_lut(512 + 1);
    println!();
    log_consts();
}
