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

/// Base FIX protocol tag/value pair
#[derive(Debug, PartialEq)]
pub struct RawTag<'a> {
    /// Tag number for a FIX message
    pub tag: u16,
    /// Unparsed data associated with a specific tag
    pub value: &'a [u8],
}
