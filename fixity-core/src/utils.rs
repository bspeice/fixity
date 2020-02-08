//! Extra utility parsers for use with `nom`

use nom::character::complete::digit1;
use nom::combinator::{map, opt};
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
pub(crate) fn atoi<'a, T, E: ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], T, E>
where
    T: PrimInt + Signed + FromPrimitive,
{
    map(tuple((opt(byte(b'-')), u_atoi)), |(sign, value)| {
        sign.map_or_else(|| value, |_| value * T::from_i8(-1).unwrap())
    })(i)
}

#[inline]
pub(crate) fn u_atoi<'a, T, E: ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], T, E>
where
    T: PrimInt + FromPrimitive,
{
    map(digit1, |digits: &[u8]| {
        digits.iter().fold(T::zero(), |val, d| {
            val * T::from_u8(10).unwrap() + T::from_u8(d - b'0').unwrap()
        })
    })(i)
}

#[cfg(test)]
mod tests {
    use crate::utils::{atoi, u_atoi, byte};
    use nom::{Err};
    use nom::error::ErrorKind;

    #[test]
    fn atoi_simple() {
        assert_eq!(atoi::<_, ()>(b"1234"), Ok((&b""[..], 1234)));
        assert_eq!(u_atoi::<_, ()>(b"1234"), Ok((&b""[..], 1234)));

        assert_eq!(atoi::<_, ()>(b"-1234"), Ok((&b""[..], -1234)));
    }

    #[test]
    fn atoi_leading_zero() {
        assert_eq!(atoi::<_, ()>(b"01234"), Ok((&b""[..], 1234)));
        assert_eq!(u_atoi::<_, ()>(b"01234"), Ok((&b""[..], 1234)));
        assert_eq!(atoi::<_, ()>(b"00123400"), Ok((&b""[..], 123400)));
        assert_eq!(u_atoi::<_, ()>(b"00123400"), Ok((&b""[..], 123400)));

        assert_eq!(atoi::<_, ()>(b"-01234"), Ok((&b""[..], -1234)));
        assert_eq!(atoi::<_, ()>(b"-00123400"), Ok((&b""[..], -123400)));
    }

    #[test]
    fn atoi_zero() {
        assert_eq!(atoi::<_, ()>(b"0"), Ok((&b""[..], 0)));
        assert_eq!(u_atoi::<_, ()>(b"0"), Ok((&b""[..], 0)));
    }

    #[test]
    fn byte_simple() {
        assert_eq!(byte::<(&[u8], ErrorKind)>(b'a')(b"abc"), Ok((&b"bc"[..], b'a')));
        assert_eq!(byte::<(&[u8], ErrorKind)>(b'a')(b"bc"), Err(Err::Error((&b"bc"[..], ErrorKind::Char))));
    }
}
