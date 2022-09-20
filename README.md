This crate provides an implementation of octuple-precision binary
floating-point arithmetics.

### From Wikipedia:

"In its 2008 revision, the IEEE 754 standard specifies a `binary256` format
among the interchange formats (it is not a basic format), as having:

    Sign bit: 1 bit
    Exponent width: 19 bits
    Significand precision: 237 bits (236 explicitly stored)

The format is written with an implicit lead bit with value 1 unless the
exponent is all zeros. Thus only 236 bits of the significand appear in the
memory format, but the total precision is 237 bits (approximately 71 decimal
digits: log₁₀(2²³⁷) ≈ 71.344). The bits are laid out as follows:

![Layout of octuple-precision floating-point format](images/Octuple_precision_layout.png)

##### Exponent encoding

The octuple-precision binary floating-point exponent is encoded using an
offset binary representation, with the zero offset being 262143; also known as
exponent bias in the IEEE 754 standard.

    Eₘᵢₙ = −262142
    Eₘₐₓ = 262143
    Exponent bias = 3FFFF₁₆ = 262143

Thus, as defined by the offset binary representation, in order to get the true
exponent the offset of 262143 has to be subtracted from the stored exponent.

The stored exponents 00000₁₆ and 7FFFF₁₆ are interpreted specially.

| Exponent          | Significand zero | Significand non-zero    | Equation                                                                 |
|-------------------|------------------|-------------------------|--------------------------------------------------------------------------|
| 00000₁₆           | 0, −0            | subnormal numbers       | (-1)<sup>signbit</sup> × 2<sup>−262142</sup> × 0.significandbits₂        |
| 00001₁₆ … 7FFFE₁₆ | normalized value | normalized value        | (-1)<sup>signbit</sup> × 2<sup>exponent bits₂</sup> × 1.significandbits₂ |
| 7FFFF₁₆           | ±∞               | NaN (quiet, signalling) |

The minimum strictly positive (subnormal) value is 2<sup>−262378</sup> ≈ 10<sup>−78984</sup> and has a precision of only
one bit. The minimum positive normal value is 2<sup>−262142</sup> ≈ 2.4824 × 10<sup>−78913</sup>. The maximum
representable value is 2<sup>262144</sup> − 2<sup>261907</sup> ≈ 1.6113 × 10<sup>78913</sup>.

The type `f256` will provide the same stable API as the built-in `f64`
(besides differences caused by the increased precision).

### Getting started

Add `f256` to your `Cargo.toml`:

```toml
[dependencies]
f256 = "0.1"
```
