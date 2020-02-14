//! Extra utility parsers for use with `nom`
use nom::IResult;
use num_traits::{FromPrimitive, PrimInt, Signed};

#[allow(dead_code)]
pub(crate) fn byte(_b: u8) -> impl Fn(&[u8]) -> IResult<&[u8], u8> {
    |_i: &[u8]| unimplemented!()
}

#[allow(dead_code)]
pub(crate) fn atoi<T>(_i: &[u8]) -> IResult<&[u8], T>
where
    T: PrimInt + Signed + FromPrimitive,
{
    unimplemented!()
}

#[allow(dead_code)]
pub(crate) fn u_atoi<T>(_i: &[u8]) -> IResult<&[u8], T>
where
    T: PrimInt + FromPrimitive,
{
    unimplemented!()
}

#[allow(dead_code)]
pub(crate) fn tagnum(_i: &[u8]) -> IResult<&[u8], u16>
{
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use crate::utils::{atoi, u_atoi, byte, tagnum};

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
}
