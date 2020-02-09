//! Extra utility parsers for use with `nom`

use nom::character::complete::digit1;
use nom::combinator::{map, opt, verify};
use nom::error::ParseError;
use nom::sequence::tuple;
use nom::Err;
use nom::IResult;
use num_traits::{FromPrimitive, PrimInt, Signed};

#[inline]
pub(crate) fn byte<'a, E: ParseError<&'a [u8]>>(
    b: u8,
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], u8, E> {
    move |i: &[u8]| match i.iter().next().map(|cmp| {
        let is_match = *cmp == b;
        (b, is_match)
    }) {
        Some((b, true)) => Ok((&i[1..], b)),
        _ => Err(Err::Error(E::from_char(i, b as char))),
    }
}

#[inline]
pub(crate) fn atoi<T>(i: &[u8]) -> IResult<&[u8], T>
where
    T: PrimInt + Signed + FromPrimitive,
{
    map(tuple((opt(byte(b'-')), u_atoi)), |(sign, value)| {
        sign.map_or_else(|| value, |_| value * T::from_i8(-1).unwrap())
    })(i)
}

#[inline]
pub(crate) fn u_atoi<T>(i: &[u8]) -> IResult<&[u8], T>
where
    T: PrimInt + FromPrimitive,
{
    map(digit1, |digits: &[u8]| {
        digits.iter().fold(T::zero(), |val, d| {
            val * T::from_u8(10).unwrap() + T::from_u8(d - b'0').unwrap()
        })
    })(i)
}

#[inline]
pub(crate) fn u_atoi_range<T>(min: T, max: T) -> impl Fn(&[u8]) -> IResult<&[u8], T>
where
    T: PrimInt + FromPrimitive,
{
    move |i: &[u8]| verify(u_atoi, |v| min <= *v && *v <= max)(i)
}

#[cfg(test)]
mod tests {
    use crate::utils::{atoi, byte, u_atoi, u_atoi_range};
    use nom::error::ErrorKind;
    use nom::Err;

    #[test]
    fn atoi_simple() {
        let msg = &b"1234"[..];
        assert_eq!(atoi(msg), Ok((&b""[..], 1234)));
        assert_eq!(u_atoi(msg), Ok((&b""[..], 1234)));
    }

    #[test]
    fn atoi_signed() {
        let msg = &b"-1234"[..];
        assert_eq!(atoi(msg), Ok((&b""[..], -1234)));
    }

    #[test]
    fn atoi_leading_zero() {
        assert_eq!(atoi(b"01234"), Ok((&b""[..], 1234)));
        assert_eq!(u_atoi(b"01234"), Ok((&b""[..], 1234)));
        assert_eq!(atoi(b"00123400"), Ok((&b""[..], 123400)));
        assert_eq!(u_atoi(b"00123400"), Ok((&b""[..], 123400)));

        assert_eq!(atoi(b"-01234"), Ok((&b""[..], -1234)));
        assert_eq!(atoi(b"-00123400"), Ok((&b""[..], -123400)));
    }

    #[test]
    fn atoi_zero() {
        assert_eq!(atoi(b"0"), Ok((&b""[..], 0)));
        assert_eq!(u_atoi(b"0"), Ok((&b""[..], 0)));
    }

    #[test]
    fn atoi_format() {
        let msg = &b"|1234"[..];
        assert_eq!(atoi::<i16>(msg), Err(Err::Error((msg, ErrorKind::Digit))));
        assert_eq!(u_atoi::<u16>(msg), Err(Err::Error((msg, ErrorKind::Digit))));

        let msg = &b"1234|"[..];
        assert_eq!(atoi(msg), Ok((&b"|"[..], 1234)));
        assert_eq!(u_atoi(msg), Ok((&b"|"[..], 1234)));
    }

    #[test]
    fn byte_simple() {
        assert_eq!(
            byte::<(&[u8], ErrorKind)>(b'a')(b"abc"),
            Ok((&b"bc"[..], b'a'))
        );
        assert_eq!(
            byte::<(&[u8], ErrorKind)>(b'a')(b"bc"),
            Err(Err::Error((&b"bc"[..], ErrorKind::Char)))
        );
    }

    #[test]
    fn u_atoi_range_simple() {
        let msg = &b"1234"[..];
        assert_eq!(u_atoi_range(0, 1234)(msg), Ok((&b""[..], 1234)));
        assert_eq!(u_atoi_range(1234, 2048)(msg), Ok((&b""[..], 1234)));
        assert_eq!(
            u_atoi_range(1234, 0)(msg),
            Err(Err::Error((msg, ErrorKind::Verify)))
        );
    }
}
