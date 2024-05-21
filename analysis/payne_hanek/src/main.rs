// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use rug::{float::Constant::Pi, integer::Order, Float, Integer};

const P: u32 = 236;
const EXP_MAX: u32 = 2_u32.pow(18) - 1;
const LZ_MAX: u32 = 253;
const N: u32 = EXP_MAX + LZ_MAX + 2 * P + 4;
const L: u32 = N / u8::BITS;

fn main() {
    assert_eq!(N % u8::BITS, 0);
    let r = Integer::from(2) / Float::with_val(N, Pi);
    let (m, e) = r.to_integer_exp().unwrap();
    assert_eq!(e.unsigned_abs(), N);
    assert_eq!(m.significant_digits::<u8>(), L as usize);
    let bytes: Vec<u8> = m.to_digits(Order::MsfBe);

    for b in &bytes {
        print!("0x{b:02x},");
    }
}
