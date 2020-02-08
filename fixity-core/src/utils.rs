//! Extra utility parsers for use with `nom`

use nom::character::streaming::digit1;
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
    use crate::utils::atoi;
    use nom::{Err, Needed};

    #[test]
    fn atoi_incomplete() {
        let bytes = b"1234";

        let parsed = atoi::<i16, ()>(bytes);
        assert_eq!(parsed, Err(Err::Incomplete(Needed::Size(1))));
    }

    #[test]
    fn atoi_complete() {
        let bytes = b"1234|";

        let (rem, val) = atoi::<i16, ()>(bytes).unwrap();
        assert_eq!(rem, b"|");
        assert_eq!(val, 1234_i16);
    }

    #[test]
    fn atoi_negative() {
        let bytes = b"-1234|";

        let (rem, val) = atoi::<i16, ()>(bytes).unwrap();
        assert_eq!(rem, b"|");
        assert_eq!(val, -1234_i16);
    }
}
