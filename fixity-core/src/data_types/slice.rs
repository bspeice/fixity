//! FIX value types

use crate::data_types::{FixValue, ParseResult, ParseErrorKind};
use core::str::from_utf8;

// Note: According to the FIX data type definition, string types must not contain the delimiter
// character. However, we don't check for the delimiter here because there's no way for the message
// parser to give us a invalid value; an invalid delimiter in the message corrupts the entire
// message, not just this value.
impl<'a> FixValue<'a, ParseErrorKind> for &'a str {
    fn from_bytes(input: &'a [u8]) -> ParseResult<Self, ParseErrorKind> {
        // While strings in FIX are required to be ASCII by the protocol definition, I'm not sure
        // if this is respected in practice.
        // If users need alternate encodings, they should implement `FixValue` on a wrapper type.
        from_utf8(input).map_err(|_| ParseErrorKind::String)
    }

    fn to_bytes(&self, buf: &mut [u8]) -> Option<usize> {
        // While strings in FIX are required to be ASCII by the protocol definition, I'm not sure
        // if this is respected in practice.
        // If users need alternate encodings, they should implement `FixValue` on a wrapper type.
        let self_bytes = self.as_bytes();
        if buf.len() >= self_bytes.len() {
            buf.copy_from_slice(self_bytes);
            Some(self_bytes.len())
        } else {
            None
        }
    }
}

impl<'a> FixValue<'a, ()> for &'a [u8] {
    fn from_bytes(input: &'a [u8]) -> ParseResult<Self, ()> {
        Ok(input)
    }

    fn to_bytes(&self, buf: &mut [u8]) -> Option<usize> {
        if buf.len() >= self.len() {
            buf.copy_from_slice(self);
            Some(self.len())
        } else {
            None
        }
    }
}
