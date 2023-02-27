// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{fmt, slice::Iter};

/// Parts of a formatted decimal number.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Part<'a> {
    /// Either '-', '+', '.', 'e' or 'E'
    Char(char),
    /// A sequence of decimal digits.
    Digits(&'a str),
    /// Given number of zero digits.
    Zeroes(usize),
    /// Given number of given char.
    Padding(usize, char),
}

impl<'a> Part<'a> {
    /// Returns the byte length of given part.
    pub fn len(&self) -> usize {
        match *self {
            Self::Char(_) => 1,
            Self::Digits(s) => s.len(),
            Self::Zeroes(n) => n,
            Self::Padding(n, _) => n,
        }
    }

    /// Writes a part into the supplied buffer.
    pub fn write<T: fmt::Write>(&self, mut buf: T) -> fmt::Result {
        match *self {
            Self::Char(c) => {
                buf.write_char(c)?;
            }
            Self::Digits(digits) => {
                buf.write_str(digits)?;
            }
            Self::Zeroes(n) => {
                for i in 0..n {
                    buf.write_char('0')?;
                }
            }
            Self::Padding(n, filler) => {
                for i in 0..n {
                    buf.write_char(filler)?;
                }
            }
        }
        Ok(())
    }
}

/// Representation of a formatted decimal number
#[derive(Clone)]
pub(crate) struct Formatted<'a> {
    pub(crate) parts: &'a [Part<'a>],
}

impl<'a> Formatted<'a> {
    /// Return the byte length of the formatted result.
    pub fn len(&self) -> usize {
        self.parts.iter().map(|p| p.len()).sum()
    }

    /// Writes all parts into the supplied buffer.
    pub fn write<T: fmt::Write>(&self, mut buf: T) -> fmt::Result {
        for part in self.parts {
            part.write(&mut buf)?;
        }
        Ok(())
    }

    /// Writes the formatted parts to the formatter after applying the padding.
    pub fn pad_parts(
        &self,
        is_sign_negative: bool,
        form: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut width = form.width().unwrap_or(1);
        let mut sign = if is_sign_negative {
            "-"
        } else if form.sign_plus() {
            "+"
        } else {
            ""
        };
        let len = self.len() + sign.len();
        let mut pre_padding: Option<Part<'_>> = None;
        let mut post_padding: Option<Part<'_>> = None;
        if len < width {
            if form.sign_aware_zero_pad() {
                // Render sign first, then zero padding.
                form.write_str(sign)?;
                sign = "";
                pre_padding = Some(Part::Padding(width - len, '0'));
            } else {
                let filler = form.fill();
                let n_to_fill = width - len;
                (pre_padding, post_padding) = match form.align() {
                    Some(fmt::Alignment::Center) => {
                        let n_to_fill_pre = n_to_fill / 2;
                        (
                            Some(Part::Padding(n_to_fill_pre, filler)),
                            Some(Part::Padding(
                                n_to_fill - n_to_fill_pre,
                                filler,
                            )),
                        )
                    }
                    Some(fmt::Alignment::Left) => {
                        (None, Some(Part::Padding(n_to_fill, filler)))
                    }
                    _ => {
                        // Default to right align
                        (Some(Part::Padding(n_to_fill, filler)), None)
                    }
                };
            }
            if let Some(part) = pre_padding {
                part.write(&mut *form)?;
            }
            form.write_str(sign)?;
            self.write(&mut *form)?;
            if let Some(part) = post_padding {
                part.write(&mut *form)?;
            }
        } else {
            // No alignment, no padding
            form.write_str(sign)?;
            self.write(&mut *form)?;
        }
        Ok(())
    }
}
