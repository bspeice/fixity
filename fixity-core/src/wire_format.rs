//! FIX protocol wire format handling. The wire format of a FIX message is incredibly simple:
//! Integer keys with ASCII values, delimited by the SOH (ASCII 0x01) byte. Value payloads are not
//! allowed to include the delimiter character. There are two exceptions to this format:
//!
//! - Because the SOH character is unprintable, the delimiter is often replaced with
//!   a separate value when humans need to inspect messages
//! - The "data" value type is allowed to include the delimiter character, because its length
//!   is always transmitted in a tag immediately preceding it.
//!
//! Because we have to be robust to properly-formed messages that contain illegal values,
//! parsing a FIX message happens in multiple phases; the first phase simply splits up each tag
//! (key-value pair), message content validation occurs later.
use nom::bytes::complete::{take, take_till1};
use nom::sequence::tuple;
use nom::IResult;

use crate::utils::{byte, u_atoi};

/// The default FIX delimiter value; used to separate individual FIX tags
pub const SOH: u8 = 0x01;

/// Base FIX protocol tag/value pair
#[derive(Debug, PartialEq)]
pub struct RawTag<'a> {
    /// Tag number for a FIX message
    pub tag: u16,
    /// Unparsed data associated with a specific tag
    pub value: &'a [u8],
}

/// Read a simple FIX tag using a custom delimiter. Returns a struct containing the tag number
/// and value bytes.
///
/// ```rust
/// # use fixity_core::wire_format::{RawTag, delimited};
/// let message = "8=FIX.4.4|".as_bytes();
/// let (_, tag) = delimited(b'|')(message).unwrap();
/// assert_eq!(tag, RawTag { tag: 8, value: "FIX.4.4".as_bytes() });
/// ```
pub fn delimited(delimiter: u8) -> impl Fn(&[u8]) -> IResult<&[u8], RawTag> {
    move |i: &[u8]| {
        let (rem, (tag, _, value, _)) = tuple((
            u_atoi,
            byte(b'='),
            take_till1(|b| b == delimiter),
            byte(delimiter),
        ))(i)?;

        Ok((rem, RawTag { tag, value }))
    }
}

/// Read a simple FIX tag using the default (ASCII SOH) delimiter. Returns a struct containing
/// the tag number and value bytes.
///
/// ```rust
/// # use fixity_core::wire_format::{RawTag, tag};
/// let message = &[56, 61, 70, 73, 88, 46, 52, 46, 52, 0x01];
/// let (_, tag) = tag()(message).unwrap();
/// assert_eq!(tag, RawTag { tag: 8, value: "FIX.4.4".as_bytes() });
/// ```
pub fn tag() -> impl Fn(&[u8]) -> IResult<&[u8], RawTag> {
    delimited(SOH)
}

/// Read a FIX fixed-length data tag.
///
/// In addition to the more-common `<tag>=<value>SOH` format, FIX tags can include arbitrary bytes
/// in data tags. These tags are guaranteed to be preceded by a tag declaring the payload length,
/// because data tags are allowed to contain the delimiter field within them.
///
/// ```rust
/// # use fixity_core::wire_format::{RawTag, data_delimited, delimited};
/// let message = "24=3|25=||||".as_bytes();
/// let (rem, len_tag) = delimited(b'|')(message).unwrap();
///
/// assert_eq!(len_tag, RawTag { tag: 24, value: b"3" });
/// let len_str = std::str::from_utf8(len_tag.value).unwrap();
/// let len: usize = len_str.parse().unwrap();
///
/// let (rem, data_tag) = data_delimited(len, b'|')(rem).unwrap();
/// assert_eq!(data_tag, RawTag { tag: 25, value: b"|||" });
/// ```
pub fn data_delimited(count: usize, delimiter: u8) -> impl Fn(&[u8]) -> IResult<&[u8], RawTag> {
    move |i: &[u8]| {
        let (rem, (tag, _, value, _)) =
            tuple((u_atoi, byte(b'='), take(count), byte(delimiter)))(i)?;

        Ok((rem, RawTag { tag, value }))
    }
}

/// Read a FIX fixed-length data tag delimited by the default (ASCII SOH) delimiter.
///
/// ```rust
/// # use fixity_core::wire_format::{RawTag, data_tag, tag};
/// let message = &[50, 52, 61, 51, 01, 50, 53, 61, 01, 01, 01, 01];
/// let (rem, len_tag) = tag()(message).unwrap();
///
/// assert_eq!(len_tag, RawTag { tag: 24, value: b"3" });
/// let len_str = std::str::from_utf8(len_tag.value).unwrap();
/// let len: usize = len_str.parse().unwrap();
///
/// let (rem, data_tag) = data_tag(len)(rem).unwrap();
/// assert_eq!(data_tag, RawTag { tag: 25, value: &[01, 01, 01] });
/// ```
pub fn data_tag(length: usize) -> impl Fn(&[u8]) -> IResult<&[u8], RawTag> {
    data_delimited(length, SOH)
}
