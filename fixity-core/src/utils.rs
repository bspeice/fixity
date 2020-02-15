//! Extra utility parsers for use with `nom`
use nom::bytes::complete::{is_a, take};
use nom::character::complete::digit1;
use nom::combinator::{map_res, opt, verify};
use nom::error::ErrorKind;
use nom::sequence::tuple;
use nom::IResult;
use num_traits::{FromPrimitive, PrimInt, Signed};

#[allow(dead_code)]
pub(crate) fn byte(b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], u8> {
    move |i: &[u8]| verify(take(1_u8), |i: &[u8]| i[0] == b)(i).map(|(i, v)| (i, v[0]))
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
pub(crate) fn tagnum(i: &[u8]) -> IResult<&[u8], u16> {
    // Tagnum is an unsigned `atoi` that doesn't accept leading zeros
    tuple((
        verify(opt(is_a(&b"0"[..])), |i: &Option<_>| i.is_none()),
        u_atoi::<u16>,
    ))(i)
    .map(|(i, v)| (i, v.1))
}

#[cfg(test)]
mod tests {
    use crate::utils::{atoi, byte, tagnum, u_atoi};

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
}
