// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::cmp::max;
use core::ops::{Mul, Neg, Rem, Shl, Sub};
use rug::ops::Pow;
use rug::{Complete, Integer};

const PREC_LEVEL: u32 = 8;
const TOTAL_BITS: u32 = 1_u32 << PREC_LEVEL;
const EXP_BITS: u32 = 4 * PREC_LEVEL - 13;
const SIGNIFICAND_BITS: u32 = TOTAL_BITS - EXP_BITS;
// const FRACTION_BITS: u32 = SIGNIFICAND_BITS - 1;
const EMAX: i32 = (1 << (EXP_BITS - 1)) - 1;
const EMIN: i32 = 1 - EMAX;

/// Computes the minimum and maximum of (x * e) % y for e in (0..lim].
fn minmax(x: &Integer, y: &Integer, limit: &Integer) -> (Integer, Integer) {
    let c = x.clone().gcd(y);
    let mut b = y.clone().div_exact(&c);
    if limit >= &b {
        return (Integer::ZERO, y.sub(&c).complete());
    }
    let mut a = x.clone().div_exact(&c).rem(&b);
    let mut s = Integer::from(1);
    let mut t = Integer::ZERO;
    let mut u = Integer::ZERO;
    let mut v = Integer::from(1);
    loop {
        while &b >= &a {
            u -= &s;
            if (&u).neg().complete() >= *limit {
                return (a, y.sub(&b).complete());
            }
            b -= &a;
            v -= &t;
        }
        if b == Integer::ZERO {
            return (Integer::from(1), Integer::from(1));
        }
        while &a >= &b {
            s -= &u;
            if &s >= limit {
                return (a, y.sub(&b).complete());
            }
            a -= &b;
            t -= &v;
        }
        if a == Integer::ZERO {
            return (Integer::from(1), Integer::from(1));
        }
    }
}

fn limits_for_ryu(limit: &Integer) {
    let log10_2 = 2_f64.log10();
    let log10_5 = 5_f64.log10();

    let mut b0 = 0_u32;
    let maxe = EMAX - SIGNIFICAND_BITS as i32 - 1;
    for e in 0..maxe {
        let q = (log10_2.mul(f64::from(e)).floor() as i32).sub(1).max(0);
        let p5 = Integer::from(5).pow(q as u32);
        let p2 = Integer::from(1).shl((e - q) as usize);
        let (_, maxv) = minmax(&p2, &p5, &limit);
        let num = (&limit).mul(&p2).complete().mul(&p5);
        let den = &p5 - maxv;
        let quot = num / den;
        let bits = quot.significant_bits() - &p5.significant_bits();
        b0 = max(b0, bits);
    }
    println!("   B0: {b0}");

    let mut b1 = 0_u32;
    let maxe = -EMIN + SIGNIFICAND_BITS as i32 + 1;
    for e in 0..maxe {
        let q = (log10_5.mul(f64::from(e)).floor() as i32).sub(1).max(0);
        let p5 = Integer::from(5).pow((e - q) as u32);
        let p2 = Integer::from(1).shl(q as usize);
        let (minv, _) = minmax(&p5, &p2, &limit);
        let quot = minv / limit;
        let bits = &p5.significant_bits() - quot.significant_bits();
        b1 = max(b1, bits);
    }
    println!("   B1: {b1}");
}

fn limits_for_ryu_print(limit: &Integer) {
    let log10_2 = 2_f64.log10();

    let mut c1 = 0_u32;
    let maxe2 = (-EMIN + SIGNIFICAND_BITS as i32 + 1) as u32;
    let maxe10 = 100_u32;
    for e2 in 1..maxe2 {
        let e10 = log10_2.mul(e2 as f64).floor() as u32 + 1;
        let v10 = Integer::from(10).pow(e10);
        let v2 = Integer::from(1).shl(e2);
        let (minv, _) = minmax(&v10, &v2, &limit);
        let mut quot = if minv == Integer::ZERO {
            limit.clone()
        } else {
            limit / minv
        };
        // quot += e2;
        let bits = quot.significant_bits().saturating_sub(e2);
        c1 = max(c1, bits);
        // println!("{e2}, {c1}")
    }
    println!("   C1: {c1}");
}

fn main() {
    println!("Float{TOTAL_BITS}:");
    println!("   PREC = {SIGNIFICAND_BITS}");
    println!("   Emax = {EMAX}");
    println!("   Emin = {EMIN}");
    let limit = (Integer::from(1) << (SIGNIFICAND_BITS + 2)) - 3;
    println!("   Wmax = {limit}");
    limits_for_ryu(&limit);
    limits_for_ryu_print(&limit);
}
