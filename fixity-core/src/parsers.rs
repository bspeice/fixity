//! Extra utility parsers for use with `nom`

use nom::bytes::streaming::take_till1;
use nom::character::is_digit;
use nom::combinator::opt;
use nom::error::ParseError;
use nom::Err;
use nom::IResult;
use num_traits::{FromPrimitive, PrimInt, Signed};

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

pub(crate) fn atoi<'a, T, E: ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], T, E>
where
    T: PrimInt + Signed + FromPrimitive,
{
    let (i, is_neg) = opt(byte(b'-'))(i)?;
    let (i, digits) = take_till1(|b| !is_digit(b))(i)?;

    let mut value = T::zero();
    for d in digits {
        value = value * T::from_u8(10).unwrap();
        value = value + T::from_u8(d - b'0').unwrap();
    }

    Ok((i, is_neg.map_or(value, |_| -value)))
}

pub(crate) fn u_atoi<'a, T, E: ParseError<&'a [u8]>>(i: &'a [u8]) -> IResult<&'a [u8], T, E>
where
    T: PrimInt + FromPrimitive,
{
    let (i, digits) = take_till1(|b| !is_digit(b))(i)?;

    let mut value = T::zero();
    for d in digits {
        value = value * T::from_u8(10).unwrap();
        value = value + T::from_u8(d - b'0').unwrap();
    }

    Ok((i, value))
}

#[cfg(test)]
mod tests {
    use crate::parsers::{atoi, byte};
    use nom::sequence::tuple;
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

        let (rem, (val, _)) = tuple((atoi::<i16, ()>, byte(b'|')))(bytes).unwrap();
        assert!(rem.is_empty());
        assert_eq!(val, 1234_i16);
    }
}
