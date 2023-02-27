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
from math import ceil, log2, log10, floor

P = 237
EMAX = 262143
EMIN = 1 - EMAX

T = 384
TECHB = 1 << 128

CHUNK_SIZE = 19
COMPRESSION_RATE = 32
ADDITIONAL_BITS = T - 16 - ceil(log2(10 ** CHUNK_SIZE)) - COMPRESSION_RATE
BITS = ADDITIONAL_BITS + COMPRESSION_RATE
MOD = (10 ** CHUNK_SIZE) << BITS
assert MOD.bit_length() < T

MAX_PREC = 75
CHUNK_CUTOFF = ceil(MAX_PREC / CHUNK_SIZE) + 2

LOG10_2 = log10(2)
LOG10_5 = log10(5)


def print_entry(idx: int, segment_idx: int, chunk_idx: int, t: int) -> None:
    """Print table entry."""
    assert t.bit_length() <= T
    chunks = []
    q = t
    for i in range(3):
        q, r = divmod(q, TECHB)
        chunks.append(r)
    print(
        f"    (0x{chunks[2]:032x}, 0x{chunks[1]:032x}, 0x{chunks[0]:032x}), "
        f"// {idx:>5}: [{segment_idx:>3}, {chunk_idx:>4}]"
    )


# Number of segments to be omitted from table for positive exponents.
POS_TAB_SEGM_OFFSET = (512 - P) // COMPRESSION_RATE


def gen_pos_tables() -> None:
    """Generate helper tables for exp > 0."""
    n_segments = (EMAX + 1 - P) // COMPRESSION_RATE + 1 - POS_TAB_SEGM_OFFSET
    table_size = n_segments * CHUNK_CUTOFF
    params = []
    table = []

    for segment_idx in range(n_segments):
        # limits of binary exponents mapped to segment
        upper_exp = (segment_idx + POS_TAB_SEGM_OFFSET + 1) * COMPRESSION_RATE - 1
        lower_exp = upper_exp - COMPRESSION_RATE + 1
        shift = lower_exp + BITS
        first_chunk = (ceil(log10(2) * (upper_exp + P - 1)) + 1) // CHUNK_SIZE
        params.append((lower_exp, upper_exp, first_chunk + 1, shift))
        for chunk_idx in range(first_chunk, first_chunk - CHUNK_CUTOFF, -1):
            t = ((1 << shift) // 10 ** (chunk_idx * CHUNK_SIZE) + 1) % MOD
            assert (t.bit_length() < T)
            table.append((segment_idx, chunk_idx, t))

    print(f"const N_SEGMENTS: usize = {n_segments};")
    print(f"const SEGMENT_OFFSET: u32 = {POS_TAB_SEGM_OFFSET};\n")

    print("#[rustfmt::skip]")
    print("static POW2_DIV_POW10_PARAMS: [(u32, u32); N_SEGMENTS] = [")
    for idx, (lower_exp, upper_exp, n_chunks, shift) in enumerate(params):
        print(
            f"    ({n_chunks:>6}, {shift:>8}), // {idx:>5}: "
            f"{lower_exp:>6} .. {upper_exp:>6}")
    print("];\n")

    print(f"const TABLE_SIZE: usize = {table_size};\n")

    print("#[rustfmt::skip]")
    print("static POW2_DIV_POW10_TABLE: [(u128, u128, u128); TABLE_SIZE] = [")
    for idx, entry in enumerate(table):
        print_entry(idx, *entry)
    print("];")


# Number of segments to be omitted from table for negative exponents.
NEG_TAB_SEGM_OFFSET = (P - 1) // COMPRESSION_RATE


def calc_n_zero_chunks(exp: int) -> int:
    """Calculate the minimal number of zero chunks from exponent."""
    return max(0, -ceil(LOG10_2 * (exp + P)) // CHUNK_SIZE)


def gen_neg_tables() -> None:
    """Generate helper tables for exp < -236."""
    n_segments = \
        (-EMIN + 2 * P - 1) // COMPRESSION_RATE + 1 - NEG_TAB_SEGM_OFFSET
    params = []
    table = []
    offset = 0
    for segment_idx in range(n_segments):
        # limits of binary exponents mapped to segment
        upper_exp = -(segment_idx + NEG_TAB_SEGM_OFFSET) * COMPRESSION_RATE
        lower_exp = upper_exp - COMPRESSION_RATE + 1
        shift = BITS + lower_exp
        shift_left = max(0, shift)
        shift_right = max(0, -shift)
        chunk_count = 0
        chunk_idx = calc_n_zero_chunks(upper_exp)
        params.append((lower_exp, upper_exp, chunk_idx, shift))
        while chunk_count < CHUNK_CUTOFF:
            chunk_idx += 1
            t = (((10 ** (chunk_idx * CHUNK_SIZE)) << shift_left) >>
                 shift_right) % MOD
            assert t.bit_length() < T
            chunk_count += 1
            table.append((segment_idx, -chunk_idx, t))
        offset += chunk_count

    table_size = len(table)
    assert table_size == offset

    print(f"const N_SEGMENTS: usize = {n_segments};")
    print(f"const SEGMENT_OFFSET: u32 = {NEG_TAB_SEGM_OFFSET};\n")

    print("#[rustfmt::skip]")
    print("static POW10_DIV_POW2_PARAMS: [(u32, i32); N_SEGMENTS] = [")
    for idx, (lower_exp, upper_exp, n_zero_chunks, shift) in enumerate(params):
        print(
            f"    ({n_zero_chunks:>6}, {shift:>8}), // {idx:>5}: "
            f"{upper_exp:>6} .. {lower_exp:>6}")
    print("];\n")

    print(f"const TABLE_SIZE: usize = {table_size};\n")

    print("#[rustfmt::skip]")
    print("static POW10_DIV_POW2_TABLE: [(u128, u128, u128); TABLE_SIZE] = [")
    for idx, entry in enumerate(table):
        print_entry(idx, *entry)
    print("];\n")


if __name__ == '__main__':
    print(f"P = {P}\n"
          f"EMAX = {EMAX}\n"
          f"EMIN = {EMIN}\n"
          f"CHUNK_SIZE = {CHUNK_SIZE}\n"
          f"COMPRESSION_RATE = {COMPRESSION_RATE}\n"
          f"ADDITIONAL_BITS = {ADDITIONAL_BITS}\n"
          f"CHUNK_CUTOFF = {CHUNK_CUTOFF}\n"
          )
    arg = sys.argv[1]
    if arg == 'P':
        gen_pos_tables()
    elif arg == 'N':
        gen_neg_tables()
    else:
        print("Wrong arg!")
