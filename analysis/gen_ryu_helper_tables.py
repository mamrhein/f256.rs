# -*- coding: utf-8 -*-
# ----------------------------------------------------------------------------
# Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
# License:     This program is part of a larger application. For license
#              details please read the file LICENSE.TXT provided together
#              with the application.
# ----------------------------------------------------------------------------
# $Source$
# $Revision$

import sys
from math import ceil, floor, log10, log2

P = 237
EMAX = 262143
EMIN = 1 - EMAX
K = 501

B0 = 2 ** 128
B1 = 2 ** 256
B2 = 2 ** 384


def print_entry(i: int, t: int) -> None:
    """Print table entry."""
    n3, t = divmod(t, B2)
    n2, t = divmod(t, B1)
    n1, n0 = divmod(t, B0)
    print(f"    (0x{n3:032x}, 0x{n2:032x}, 0x{n1:032x}, 0x{n0:032x}), "
          f"// {i:>5}")


def gen_ge_table() -> None:
    """Generate table of values ⌊2ʰ / 5ᵍ⌋ + 1.

    The table is indexed by g ∈ [0..n), where P = 237,
    n = ⌊(Eₘₐₓ - P - 1) × log₁₀(2)⌋,, K = 501,
    Eₘₐₓ = 262143 and h = ⌊g × log₂(5)⌋ + K.
    """
    emax = EMAX - P - 1
    gmax = floor(emax * log10(2)) - 1
    for g in range(gmax + 1):
        t = 5 ** g
        h = t.bit_length() - 1 + K
        t = (1 << h) // t + 1
        assert t.bit_length() <= K + 1
        print_entry(g, t)


def gen_lt_table() -> None:
    """Generate table of values ⌊5⁻ᵉ⁻ᵍ / 2ʰ⌋.

    The table is indexed by (-e - g), where P = 237, e ∈ [Eₘᵢₙ-P-1..0),
    g = max(0, ⌊e × log₁₀(5)⌋ - 1) and h = ⌈g × log₂(5)⌉ - 501.
    """
    emin = EMIN - P - 1
    gmax = floor(-emin * log10(5)) - 1
    for i in range(-emin - gmax + 1):
        t = 5 ** i
        h = t.bit_length() - K
        if h > 0:
            t >>= h
        elif h < 0:
            t <<= -h
        print_entry(i, t)


if __name__ == '__main__':
    arg = sys.argv[1]
    if arg == 'GE':
        gen_ge_table()
    elif arg == 'LT':
        gen_lt_table()
    else:
        print("Wrong arg!")
