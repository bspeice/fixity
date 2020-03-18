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
use crate::SOH;
use nom::bytes::complete::{is_a, take, take_till1};
use nom::character::complete::digit1;
use nom::combinator::{map_res, opt, verify};
use nom::error::ErrorKind;
use nom::sequence::tuple;
use nom::IResult;
use num_traits::{FromPrimitive, PrimInt, Signed, ToPrimitive};

const ASCII_DIGITS: [u8; 10] = [
    b'0',
    b'1',
    b'2',
    b'3',
    b'4',
    b'5',
    b'6',
    b'7',
    b'8',
    b'9',
];

fn swap_outer_inner(slice: &mut [u8]) {
    let mut x = 0;
    let mut y = slice.len() - 1;

    while x < y {
        slice.swap(x, y);
        x += 1;
        y -= 1;
    }
}

pub(crate) fn byte(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], u8> {
    move |i: &[u8]| verify(take(1_u8), |i: &[u8]| i[0] == b)(i).map(|(i, v)| (i, v[0]))
}

pub(crate) fn atoi<T>(i: &[u8]) -> IResult<&[u8], T>
where
    T: PrimInt + Signed + FromPrimitive,
{
    let signed_atoi = |(sign, digits): (Option<_>, &[u8])| -> Result<T, ErrorKind> {
        let mut value = T::zero();
        for d in digits {
            value = value
                .checked_mul(&T::from_u8(10).unwrap())
                .ok_or(ErrorKind::TooLarge)?;
            value = value
                .checked_add(&sign.map_or_else(
                    || T::from_u8(*d - b'0').unwrap(),
                    |_| T::from_u8(*d - b'0').unwrap().neg(),
                ))
                .ok_or(ErrorKind::TooLarge)?;
        }
        Ok(value)
    };
    map_res(tuple((opt(byte(b'-')), digit1)), signed_atoi)(i)
}

pub(crate) fn itos<T>(value: T, buf: &mut [u8]) -> Option<usize>
where
    T: PrimInt + Signed + FromPrimitive + ToPrimitive
{
    if buf.len() == 0 {
        return None;
    }
    let mut index = 0;

    if value == T::zero() {
        buf[index] = b'0';
        return Some(1);
    }

    // This works by writing digits in reverse order, integer divide by 10, and then swap all bytes
    // from outer to inward at the end. It's not side-effect free, we don't reset the buffer if
    // we run out of space.
    let ten = T::from_u8(10).unwrap();
    let sign = value < T::zero();
    let mut value = if sign {
        value * T::from_i8(-1).unwrap()
    } else {
        value
    };


    while value != T::zero() {
        let c = (value % ten).to_usize().unwrap();
        buf[index] = ASCII_DIGITS[c];
        index += 1;

        if buf.len() < index {
            return None
        }

        value = value / ten;
    }

    // Write the sign byte if necessary
    if sign {
        buf[index] = b'-';
        index += 1;
        // We don't need to check for length here, because we're guaranteed to be done
    }

    swap_outer_inner(&mut buf[..index]);
    Some(index)
}

pub(crate) fn u_atoi<T>(i: &[u8]) -> IResult<&[u8], T>
where
    T: PrimInt + FromPrimitive,
{
    let unsigned_atoi = |digits: &[u8]| -> Result<T, ErrorKind> {
        let mut value = T::zero();
        for d in digits {
            value = value
                .checked_mul(&T::from_u8(10).unwrap())
                .ok_or(ErrorKind::TooLarge)?;
            value = value
                .checked_add(&T::from_u8(*d - b'0').unwrap())
                .ok_or(ErrorKind::TooLarge)?;
        }
        Ok(value)
    };
    map_res(digit1, unsigned_atoi)(i)
}

pub(crate) fn u_itos<T>(value: T, buf: &mut [u8]) -> Option<usize>
    where
        T: PrimInt + FromPrimitive + ToPrimitive
{
    if buf.len() == 0 {
        return None;
    }
    let mut index = 0;

    if value == T::zero() {
        buf[index] = b'0';
        return Some(1);
    }

    // This works by writing digits in reverse order, integer divide by 10, and then swap all bytes
    // from outer to inward at the end. It's not side-effect free, we don't reset the buffer if
    // we run out of space.
    let mut value = value;
    let ten = T::from_u8(10).unwrap();

    while value != T::zero() {
        let c = (value % ten).to_usize().unwrap();
        buf[index] = ASCII_DIGITS[c];
        index += 1;

        if buf.len() < index {
            return None
        }

        value = value / ten;
    }

    swap_outer_inner(&mut buf[..index]);
    Some(index)
}

pub(crate) fn tagnum(i: &[u8]) -> IResult<&[u8], u16> {
    // Tagnum is an unsigned `atoi` that doesn't accept leading zeros
    tuple((
        verify(opt(is_a(&b"0"[..])), |i: &Option<_>| i.is_none()),
        u_atoi::<u16>,
    ))(i)
    .map(|(i, v)| (i, v.1))
}

/// Base FIX protocol tag/value pair
#[derive(Debug, PartialEq)]
pub struct RawTag<'a> {
    /// Tag number for a FIX message
    pub tag: u16,
    /// Unparsed data associated with a specific tag
    pub value: &'a [u8],
}

/// Parse a simple FIX tag using a custom delimiter.
pub fn tag_delimited(delimiter: u8) -> impl Fn(&[u8]) -> IResult<&[u8], RawTag> {
    move |i: &[u8]| {
        tuple((
            tagnum,
            byte(b'='),
            take_till1(|c| c == delimiter),
            byte(delimiter),
        ))(i)
        .map(|(i, (tag, _, value, _))| (i, RawTag { tag, value }))
    }
}

/// Parse a simple FIX tag using the standard ASCII `SOH` delimiter.
pub fn tag(payload: &[u8]) -> IResult<&[u8], RawTag> {
    tag_delimited(SOH)(payload)
}

/// Parse a data FIX tag using a custom delimiter.
pub fn data_tag_delimited(delimiter: u8, len: usize) -> impl Fn(&[u8]) -> IResult<&[u8], RawTag> {
    move |i: &[u8]| {
        tuple((tagnum, byte(b'='), take(len), byte(delimiter)))(i)
            .map(|(i, (tag, _, value, _))| (i, RawTag { tag, value }))
    }
}

/// Parse a data FIX tag using the standard ASCII `SOH` delimiter.
pub fn data_tag(payload: &[u8], len: usize) -> IResult<&[u8], RawTag> {
    data_tag_delimited(SOH, len)(payload)
}

#[cfg(test)]
mod tests {
    use super::{atoi, byte, tagnum, u_atoi};
    use crate::wire_format::{data_tag_delimited, tag_delimited, RawTag, itos};

    #[test]
    fn byte_simple() {
        assert_eq!(byte(0x01)(&[0x01][..]), Ok((&[][..], 0x01)));
        assert_eq!(byte(b'|')(&b"|"[..]), Ok((&b""[..], b'|')));
        assert_eq!(byte(b'|')(&b"|1234"[..]), Ok((&b"1234"[..], b'|')));
        assert!(byte(b'|')(b"1|").is_err());
    }

    #[test]
    fn byte_empty() {
        assert!(byte(b'|')(b"").is_err());
    }

    #[test]
    fn atoi_simple() {
        assert_eq!(atoi(b"1234"), Ok((&b""[..], 1234)));
        assert_eq!(atoi(b"-1234"), Ok((&b""[..], -1234)));
        assert_eq!(atoi(b"00123400"), Ok((&b""[..], 123400)));
        assert_eq!(atoi(b"-00123400"), Ok((&b""[..], -123400)));
        assert_eq!(atoi(b"0"), Ok((&b""[..], 0)));
    }

    #[test]
    fn atoi_range() {
        assert_eq!(atoi::<i8>(b"127"), Ok((&b""[..], 127)));
        assert_eq!(atoi::<i8>(b"-128"), Ok((&b""[..], -128)));

        assert!(atoi::<i8>(b"128").is_err());
        assert!(atoi::<i8>(b"-129").is_err());
    }

    #[test]
    fn atoi_format() {
        assert!(atoi::<i16>(b"|1234").is_err());
        assert!(atoi::<i16>(b"-|1234").is_err());
        assert_eq!(atoi(b"1234|"), Ok((&b"|"[..], 1234)));
        assert_eq!(atoi(b"1|234"), Ok((&b"|234"[..], 1)));
        assert_eq!(atoi(b"-1|234"), Ok((&b"|234"[..], -1)));
    }

    #[test]
    fn atoi_empty() {
        assert!(atoi::<i8>(b"").is_err());
        assert!(u_atoi::<u8>(b"").is_err());
    }

    #[test]
    fn u_atoi_simple() {
        assert_eq!(u_atoi(b"1234"), Ok((&b""[..], 1234)));
        assert_eq!(u_atoi(b"00123400"), Ok((&b""[..], 123400)));
        assert_eq!(u_atoi(b"0"), Ok((&b""[..], 0)));
    }

    #[test]
    fn u_atoi_range() {
        assert_eq!(u_atoi::<u8>(b"255"), Ok((&b""[..], 255)));
        assert!(u_atoi::<u8>(b"256").is_err());
    }

    #[test]
    fn u_atoi_format() {
        assert!(u_atoi::<u16>(b"|1234").is_err());
        assert_eq!(u_atoi(b"1|234"), Ok((&b"|234"[..], 1)));
    }

    #[test]
    fn tagnum_simple() {
        assert_eq!(tagnum(b"1234"), Ok((&b""[..], 1234)));
        assert_eq!(tagnum(b"12340"), Ok((&b""[..], 12340)));
        assert_eq!(tagnum(b"1234|"), Ok((&b"|"[..], 1234)));
    }

    #[test]
    fn tagnum_leading_zero() {
        assert!(tagnum(b"012").is_err());
        assert!(tagnum(b"0").is_err());
    }

    #[test]
    fn atoi_overflow() {
        assert!(atoi::<i8>(b"128").is_err());
        assert!(atoi::<i8>(b"-129").is_err());
    }

    #[test]
    fn u_atoi_overflow() {
        assert!(u_atoi::<u8>(b"257").is_err());
    }

    #[test]
    fn tagnum_overflow() {
        assert!(tagnum(b"65536").is_err());
    }

    #[test]
    fn tag_delimited_simple() {
        assert_eq!(
            tag_delimited(b'|')(b"8=FIX.4.4|"),
            Ok((
                &b""[..],
                RawTag {
                    tag: 8,
                    value: b"FIX.4.4"
                }
            ))
        )
    }

    #[test]
    fn tag_delimited_missing_delimiter() {
        assert!(tag_delimited(b'|')(b"8=FIX.4.4").is_err())
    }

    #[test]
    fn data_delimited_simple() {
        assert_eq!(
            data_tag_delimited(b'|', 7)(b"8=FIX.4.4|"),
            Ok((
                &b""[..],
                RawTag {
                    tag: 8,
                    value: b"FIX.4.4"
                }
            ))
        )
    }

    #[test]
    fn data_delimited_improper_size() {
        assert!(data_tag_delimited(b'|', 6)(b"8=FIX.4.4|").is_err());
        assert!(data_tag_delimited(b'|', 8)(b"8=FIX.4.4|").is_err());
    }

    #[test]
    fn data_delimited_missing_delimiter() {
        assert!(data_tag_delimited(b'|', 7)(b"8=FIX.4.4").is_err())
    }

    #[test]
    fn itos_simple() {
        let value = 8;
        let buffer = &mut [0u8][..];
        assert_eq!(itos(value, buffer), Some(1));
        assert_eq!(buffer, b"8");

        let value = 128;
        let buffer = &mut [0u8; 3][..];
        assert_eq!(itos(value, buffer), Some(3));
        assert_eq!(buffer, b"128");

        let value = -128;
        let buffer = &mut [0u8; 4][..];
        assert_eq!(itos(value, buffer), Some(4));
        assert_eq!(buffer, b"-128");
    }

    #[test]
    fn u_itos_simple() {
        let value = 8;
        let buffer = &mut [0u8][..];
        assert_eq!(itos(value, buffer), Some(1));
        assert_eq!(buffer, b"8");

        let value = 128;
        let buffer = &mut [0u8; 3][..];
        assert_eq!(itos(value, buffer), Some(3));
        assert_eq!(buffer, b"128");
    }
}
