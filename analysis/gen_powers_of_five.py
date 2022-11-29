MAX_ABS_EXP = 1024
P = 256
B = 1 << (P >> 1)


def gen_table():
    for i in range(-MAX_ABS_EXP, 0):
        p = 5 ** -i
        msb = p.bit_length()
        shift = P - 1 + msb
        p = (1 << shift) // p
        hi, lo = divmod(p, B)
        print(f"    (0x{hi:032x}, 0x{lo:032x}, {-msb}), // 5^{i}")
    for i in range(MAX_ABS_EXP + 1):
        p = 5 ** i
        msb = p.bit_length()
        shift = P - msb
        if shift > 0:
            p <<= shift
        elif shift < 0:
            p >>= -shift
        hi, lo = divmod(p, B)
        print(f"    (0x{hi:032x}, 0x{lo:032x}, {msb - 1}), // 5^{i}")


if __name__ == '__main__':
    gen_table()
