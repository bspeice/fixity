//! FIX types representing integral values
use crate::data_types::{FixValue, ParseResult, ParseErrorKind};
use crate::wire_format::{atoi, itos, u_atoi, u_itos};
use nom::combinator::all_consuming;

macro_rules! integral_value {
    ($parser:expr, $serializer:expr, $($t:ty)*, $err:expr) => {
        $(
            impl<'a> FixValue<'a, ParseErrorKind> for $t {
                fn from_bytes(input: &[u8]) -> ParseResult<Self> {
                    all_consuming($parser)(input)
                        .map(|(_, v)| v)
                        .map_err(|_| $err)
                }

                fn to_bytes(&self, buf: &mut [u8]) -> Option<usize> {
                    $serializer(*self, buf)
                }
            }
        )*
    };
}

integral_value!(u_atoi, u_itos, u8 u16 u32 u64, ParseErrorKind::UnsignedInteger);
integral_value!(atoi, itos, i8 i16 i32 i64, ParseErrorKind::SignedInteger);
