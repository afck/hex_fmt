//! # Formatting and shortening byte slices as hexadecimal strings
//!
//! This crate provides wrappers for byte slices and lists of byte slices that implement the
//! standard formatting traits and print the bytes as a hexadecimal string, eliding from the middle
//! if the length would exceed the `precision` format parameter.
//!
//! ```
//! use hex_fmt::{HexFmt, HexList};
//!
//! assert_eq!("090a0b", &format!("{}", HexFmt(&[9u8, 10, 11])));
//! let nine_to_f = [9u8, 10, 11, 12, 13, 14, 15];
//! assert_eq!("090..0f", &format!("{:.7}", HexFmt(&nine_to_f)));
//! assert_eq!("090..E0F", &format!("{:.8X}", HexFmt(&nine_to_f)));
//! assert_eq!("090a..0e0f", &format!("{}", HexFmt(&nine_to_f)));
//! assert_eq!("[4142, 4241]", &format!("{}", HexList(&[b"AB", b"BA"])));
//! assert_eq!("[4A4B, 4B4A]", &format!("{:X}", HexList(&[b"JK", b"KJ"])));
//! ```

use std::fmt::{Debug, Display, Formatter, LowerHex, Result, UpperHex};

const DEFAULT_PRECISION: usize = 10;
const ELLIPSIS: &str = "..";

/// Wrapper for a byte array, whose `Debug`, `Display` and `LowerHex` implementations output
/// shortened hexadecimal strings.
pub struct HexFmt<T>(pub T);

impl<T: AsRef<[u8]>> Debug for HexFmt<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        LowerHex::fmt(self, f)
    }
}

impl<T: AsRef<[u8]>> Display for HexFmt<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        LowerHex::fmt(self, f)
    }
}

impl<T: AsRef<[u8]>> LowerHex for HexFmt<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        fmt(self.0.as_ref(), f, Case::Lower)
    }
}

impl<T: AsRef<[u8]>> UpperHex for HexFmt<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        fmt(self.0.as_ref(), f, Case::Upper)
    }
}

/// Wrapper for a list of byte arrays, whose `Debug`, `Display` and `LowerHex` implementations
/// output shortened hexadecimal strings.
pub struct HexList<T>(pub T);

impl<T: Clone + IntoIterator> Debug for HexList<T>
where
    T::Item: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        LowerHex::fmt(self, f)
    }
}

impl<T: Clone + IntoIterator> Display for HexList<T>
where
    T::Item: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        LowerHex::fmt(self, f)
    }
}

impl<T: Clone + IntoIterator> LowerHex for HexList<T>
where
    T::Item: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let entries = self.0.clone().into_iter().map(HexFmt);
        f.debug_list().entries(entries).finish()
    }
}

impl<T: Clone + IntoIterator> UpperHex for HexList<T>
where
    T::Item: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut iter = self.0.clone().into_iter();
        write!(f, "[")?;
        if let Some(item) = iter.next() {
            UpperHex::fmt(&HexFmt(item), f)?;
        }
        for item in iter {
            write!(f, ", ")?;
            UpperHex::fmt(&HexFmt(item), f)?;
        }
        write!(f, "]")
    }
}

fn fmt(bytes: &[u8], f: &mut Formatter, case: Case) -> Result {
    // TODO: Respect `f.width()`, `f.align()` and `f.fill()`.
    let precision = f.precision().unwrap_or(DEFAULT_PRECISION);

    // If the array is short enough, don't shorten it.
    if 2 * bytes.len() <= precision {
        for byte in bytes {
            fmt_byte(f, *byte, case)?;
        }
        return Ok(());
    }

    // If the bytes don't fit and the ellipsis fills the maximum width, print only that.
    if precision <= ELLIPSIS.len() {
        return write!(f, "{:.*}", precision, ELLIPSIS);
    }

    // Compute the number of hex digits to display left and right of the ellipsis.
    let num_hex_digits = precision.saturating_sub(ELLIPSIS.len());
    let right = num_hex_digits / 2;
    let left = num_hex_digits - right;

    // Print the bytes on the left.
    for byte in &bytes[..(left / 2)] {
        fmt_byte(f, *byte, case)?;
    }
    // If odd, print only the first hex digit of the next byte.
    if left & 1 == 1 {
        fmt_digit(f, bytes[left / 2] >> 4, case)?;
    }

    // Print the ellipsis.
    f.write_str(ELLIPSIS)?;

    // If `right` is odd, print the second hex digit of a byte.
    if right & 1 == 1 {
        fmt_digit(f, bytes[(bytes.len() - right / 2 - 1)] & 0x0f, case)?;
    }
    // Print the remaining bytes on the right.
    for byte in &bytes[(bytes.len() - right / 2)..] {
        fmt_byte(f, *byte, case)?;
    }
    Ok(())
}

fn fmt_byte(f: &mut Formatter, byte: u8, case: Case) -> Result {
    match case {
        Case::Upper => write!(f, "{:02X}", byte),
        Case::Lower => write!(f, "{:02x}", byte),
    }
}

fn fmt_digit(f: &mut Formatter, digit: u8, case: Case) -> Result {
    match case {
        Case::Upper => write!(f, "{:1X}", digit),
        Case::Lower => write!(f, "{:1x}", digit),
    }
}

#[derive(Copy, Clone)]
enum Case {
    Upper,
    Lower,
}

#[cfg(test)]
mod tests {
    use super::HexFmt;

    #[test]
    fn test_fmt() {
        assert_eq!("", &format!("{:.0}", HexFmt(&[0x01])));
        assert_eq!(".", &format!("{:.1}", HexFmt(&[0x01])));
        assert_eq!("01", &format!("{:.2}", HexFmt(&[0x01])));
        assert_eq!("..", &format!("{:.2}", HexFmt(&[0x01, 0x23])));
    }
}
