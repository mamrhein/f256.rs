// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ptr;

/// Check whether an u64 is holding 8 decimal digits.
#[inline]
pub fn chunk_contains_8_digits(chunk: u64) -> bool {
    // Subtract b'0' from each byte.
    let x = chunk.wrapping_sub(0x3030303030303030);
    // Add 0x46 (= 0x7f - b'9') to each byte.
    let y = chunk.wrapping_add(0x4646464646464646);
    // In x now all original bytes < b'0' have the highest bit set, and
    // in y now all original bytes > b'9' are > 0x7f.
    // Then, in x|y all original bytes besides b'0' .. b'9' are > 0x7f.
    // Thus, bitwise-and with 0x80 gives 0 for all original bytes b'0' .. b'9'
    // and 0x7f for all others.
    (x | y) & 0x8080808080808080 == 0
}

/// Check whether an u64 is holding 7 decimal digits and a b'.'.
/// If so, return the index of the b'.', counted from lowest byte, i.e. the
/// left-most byte (digits in little endian order!).
#[inline]
pub fn chunk_contains_7_digits_and_a_dot_at(mut chunk: u64) -> Option<u32> {
    let x = chunk ^ 0x2e2e2e2e2e2e2e2e;
    let y = x.wrapping_sub(0x0101010101010101);
    let z = y & (!x & 0x8080808080808080);
    // In x all original bytes = b'.' are = 0x00, while all others are not.
    // In y only original bytes = b'.' or > 0x80 have the high bit set.
    // Bitwise negating x and clearing all but the high bit gives a value
    // where only original bytes = b'.' or < 0x80 have the high bit set.
    // Finally, by ANDing this with y, in z only original bytes = b'.' are
    // = 0x80 while all others are zero.
    if z.count_ones() == 1 {
        // We have found just one b'.'.
        // Check whether the other bytes are digits.
        let n = z.leading_zeros();
        // Turn the b'.' into b'0'.
        chunk += ((b'0' - b'.') as u64) << ((u64::BITS - 8 - n) as u64);
        if chunk_contains_8_digits(chunk) {
            Some(n / 8)
        } else {
            None
        }
    } else {
        None
    }
}

/// Convert an u64 holding a sequence of 8 decimal digits into an u64.
#[inline]
pub fn chunk_to_u64(mut chunk: u64) -> u64 {
    // The following is adopted from Johnny Lee: Fast numeric string to int
    // [https://johnnylee-sde.github.io/Fast-numeric-string-to-int].
    chunk &= 0x0f0f0f0f0f0f0f0f;
    chunk = (chunk & 0x000f000f000f000f)
        .wrapping_mul(10)
        .wrapping_add((chunk >> 8) & 0x000f000f000f000f);
    chunk = (chunk & 0x0000007f0000007f)
        .wrapping_mul(100)
        .wrapping_add((chunk >> 16) & 0x0000007f0000007f);
    (chunk & 0x3fff)
        .wrapping_mul(10000)
        .wrapping_add((chunk >> 32) & 0x3fff)
}

// Internal parsing state.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ParsingState {
    pub(super) start_pos_signif: usize,
    pub(super) end_pos_signif: usize,
    pub(super) signif_truncated: bool,
    pub(super) pos_radix_point: Option<usize>,
    pub(super) invalid: bool,
}

// Bytes wrapper specialized for parsing number literals
pub(super) struct AsciiNumLit<'a> {
    bytes: &'a [u8],
    pub(super) state: ParsingState,
}

impl<'a> AsciiNumLit<'a> {
    #[inline]
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            state: ParsingState::default(),
        }
    }

    #[inline]
    pub(super) fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.bytes.len()
    }

    /// self <- self[n..]
    #[inline]
    #[allow(unsafe_code)]
    pub(super) unsafe fn skip_n(&mut self, n: usize) {
        debug_assert!(self.bytes.len() >= n);
        self.bytes = self.bytes.get_unchecked(n..);
    }

    /// self <- self[1..]
    #[inline]
    #[allow(unsafe_code)]
    pub(super) unsafe fn skip_1(&mut self) {
        self.skip_n(1)
    }

    #[inline]
    pub(super) fn first(&self) -> Option<&u8> {
        self.bytes.first()
    }

    #[inline]
    pub(super) fn first_eq(&self, b: u8) -> bool {
        Some(&b) == self.first()
    }

    #[inline]
    pub(super) fn first_is_digit(&self) -> bool {
        match self.first() {
            Some(c) if c.wrapping_sub(b'0') < 10 => true,
            _ => false,
        }
    }

    #[inline]
    pub(super) fn eq_ignore_ascii_case(&self, other: &[u8]) -> bool {
        self.bytes.eq_ignore_ascii_case(other)
    }

    #[inline]
    #[allow(unsafe_code)]
    pub(super) fn get_sign(&mut self) -> u32 {
        match self.first() {
            Some(&c) if c == b'-' => {
                // SAFETY: safe because of match
                unsafe { self.skip_1() };
                1
            }
            Some(&c) if c == b'+' => {
                // SAFETY: safe because of match
                unsafe { self.skip_1() };
                0
            }
            _ => 0,
        }
    }

    // Read 8 bytes as u64 (little-endian).
    #[inline]
    #[allow(unsafe_code)]
    unsafe fn read_u64_unchecked(&self) -> u64 {
        debug_assert!(self.bytes.len() >= 8);
        let src = self.bytes.as_ptr() as *const u64;
        u64::from_le(ptr::read_unaligned(src))
    }

    // Try to read the next 8 bytes from self.
    #[inline]
    #[allow(unsafe_code)]
    pub(super) fn read_u64(&self) -> Option<u64> {
        if self.len() >= 8 {
            // SAFETY: safe because of condition above!
            Some(unsafe { self.read_u64_unchecked() })
        } else {
            None
        }
    }

    // self <- self[x..] where x is the position of the first non-zero digit
    // or the first non-digit character.
    // If skip_radix_point is true, in case a radix point is detected its
    // position is recorded in the internal state and it is skipped also.
    #[allow(unsafe_code)]
    pub(super) fn skip_leading_zeroes(&mut self, skip_radix_point: bool) {
        // First, try chunks of 8 digits
        while let Some(mut k) = self.read_u64() {
            if chunk_contains_8_digits(k) {
                if chunk_to_u64(k) == 0 {
                    // SAFETY: safe because of call to self.read_u64 above
                    unsafe {
                        self.skip_n(8);
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        // Handle remaining zeroes
        while self.first_eq(b'0') {
            // SAFETY: safe because of condition above!
            unsafe {
                self.skip_1();
            };
        }
        if self.first_eq(b'.') {
            if let Some(_) = self.state.pos_radix_point {
                // Double radix point
                self.state.invalid = true;
                return;
            }
            if skip_radix_point {
                self.state.pos_radix_point = Some(self.len());
                // SAFETY: safe because of condition above!
                unsafe {
                    self.skip_1();
                };
                self.skip_leading_zeroes(false);
            }
        }
    }

    #[allow(unsafe_code)]
    pub(super) fn parse_exponent(&mut self) -> Option<i32> {
        let mut exponent = 0_i32;
        if let Some(c) = self.first() {
            if *c == b'e' || *c == b'E' {
                // SAFETY: safe because of condition above
                unsafe { self.skip_1() };
                let exp_is_negative = match self.first() {
                    None => {
                        return None;
                    }
                    Some(&c) if c == b'-' => {
                        // SAFETY: safe because of match
                        unsafe { self.skip_1() };
                        true
                    }
                    Some(&c) if c == b'+' => {
                        // SAFETY: safe because of match
                        unsafe { self.skip_1() };
                        false
                    }
                    _ => false,
                };
                // Need atleast one digit.
                if let Some(c) = self.first() {
                    let d = c.wrapping_sub(b'0');
                    if d < 10 {
                        exponent = d as i32;
                        // SAFETY: safe because of call to self.first above
                        unsafe {
                            self.skip_1();
                        }
                    } else {
                        return None;
                    }
                }
                while let Some(c) = self.first() {
                    let d = c.wrapping_sub(b'0');
                    if d < 10 {
                        exponent = exponent
                            .saturating_mul(10)
                            .saturating_add(d as i32);
                        // SAFETY: safe because of call to self.first above
                        unsafe {
                            self.skip_1();
                        }
                    } else {
                        break;
                    }
                }
                if exp_is_negative {
                    exponent = -exponent;
                }
            }
        }
        Some(exponent)
    }
}
