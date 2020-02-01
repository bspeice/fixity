//! FIX protocol wire format handling
use nom::bytes::complete::{take_till1, take};
use nom::character::is_digit;
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::{AsBytes, Err, IResult, Slice};
use core::ops::RangeFrom;

const SOH: u8 = 0x01;

/// Base FIX protocol tag/value pair
#[derive(Debug, PartialEq)]
pub struct Tag<'a> {
    /// Tag number for a FIX message
    pub tag: u16,
    /// Unparsed data associated with a specific tag
    pub value: &'a [u8],
}

fn atoi(slice: &[u8]) -> u16 {
    // Because FIX tag values are unsigned integers less than u16::max(), we can improve performance
    // over Rust's `str::parse<>()`.
    let mut value = 0;

    for b in slice {
        value *= 10;
        value += *b as u16 - '0' as u16;
    }

    value
}

fn byte<I, Error: ParseError<I>>(b: u8) -> impl Fn(I) -> IResult<I, u8, Error>
where
    I: AsBytes + Slice<RangeFrom<usize>>,
{
    move |i: I| match i.as_bytes().iter().next().map(|cmp| {
        let is_match = *cmp == b;
        (b, is_match)
    }) {
        Some((b, true)) => Ok((i.slice(1..), b)),
        _ => Err(Err::Error(Error::from_char(i, b as char))),
    }
}

/// Read a simple FIX tag using a custom delimiter. Returns a struct containing the tag number
/// and value bytes.
///
/// ```rust
/// # use fixity_core::wire_parser::{Tag, delimited};
/// let message = "8=FIX.4.4|".as_bytes();
/// let (_, tag) = delimited(b'|')(message).unwrap();
/// assert_eq!(tag, Tag { tag: 8, value: "FIX.4.4".as_bytes() });
/// ```
pub fn delimited(delimiter: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Tag> {
    move |i: &[u8]| {
        let (rem, (tag_bytes, _, value, _)) = tuple((
            take_till1(|b| !is_digit(b)),
            byte(b'='),
            take_till1(|b| b == delimiter),
            byte(delimiter),
        ))(i)?;

        Ok((
            rem,
            Tag {
                tag: atoi(tag_bytes),
                value,
            },
        ))
    }
}

/// Read a simple FIX tag using the default (ASCII SOH) delimiter. Returns a struct containing
/// the tag number and value bytes.
///
/// ```rust
/// # use fixity_core::wire_parser::{Tag, tag};
/// let message = &[56, 61, 70, 73, 88, 46, 52, 46, 52, 0x01];
/// let (_, tag) = tag()(message).unwrap();
/// assert_eq!(tag, Tag { tag: 8, value: "FIX.4.4".as_bytes() });
/// ```
pub fn tag() -> impl Fn(&[u8]) -> IResult<&[u8], Tag> {
    delimited(SOH)
}

/// Read a FIX fixed-length data tag.
///
/// In addition to the more-common `<tag>=<value>SOH` format, FIX tags can include arbitrary bytes
/// in data tags. These tags are guaranteed to be preceded by a tag declaring the payload length,
/// because data tags are allowed to contain the delimiter field within them.
///
/// ```rust
/// # use fixity_core::wire_parser::{Tag, data_delimited, delimited};
/// let message = "24=3|25=||||".as_bytes();
/// let (rem, len_tag) = delimited(b'|')(message).unwrap();
///
/// assert_eq!(len_tag, Tag { tag: 24, value: b"3" });
/// let len_str = std::str::from_utf8(len_tag.value).unwrap();
/// let len: usize = len_str.parse().unwrap();
///
/// let (rem, data_tag) = data_delimited(len, b'|')(rem).unwrap();
/// assert_eq!(data_tag, Tag { tag: 25, value: b"|||" });
/// ```
pub fn data_delimited(count: usize, delimiter: u8) -> impl Fn(&[u8]) -> IResult<&[u8], Tag> {
    move |i: &[u8]| {
        let (rem, (tag_bytes, _, value, _)) = tuple((
            take_till1(|b| !is_digit(b)),
            byte(b'='),
            take(count),
            byte(delimiter)
        ))(i)?;

        Ok((
            rem,
            Tag {
                tag: atoi(tag_bytes),
                value
            }
        ))
    }
}

/// Read a FIX fixed-length data tag delimited by the default (ASCII SOH) delimiter.
///
/// ```rust
/// # use fixity_core::wire_parser::{Tag, data_tag, tag};
/// let message = &[50, 52, 61, 51, 01, 50, 53, 61, 01, 01, 01, 01];
/// let (rem, len_tag) = tag()(message).unwrap();
///
/// assert_eq!(len_tag, Tag { tag: 24, value: b"3" });
/// let len_str = std::str::from_utf8(len_tag.value).unwrap();
/// let len: usize = len_str.parse().unwrap();
///
/// let (rem, data_tag) = data_tag(len)(rem).unwrap();
/// assert_eq!(data_tag, Tag { tag: 25, value: &[01, 01, 01] });
/// ```
pub fn data_tag(length: usize) -> impl Fn(&[u8]) -> IResult<&[u8], Tag> {
    data_delimited(length, SOH)
}

#[cfg(test)]
mod tests {
    use super::atoi;

    #[test]
    fn basic_atoi() {
        let bytes = &b"1234"[..];
        assert_eq!(atoi(bytes), 1234);
    }
}
