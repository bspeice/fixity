//! De/Serialization between FIX data types and Rust's native types.
pub mod int;
pub mod slice;

/// Base trait for de/serializing the "value" part of FIX's "tag-value" format.
///
/// Each FIX message is an ASCII byte stream made up of many key-value pairs with a delimiter
/// in between (we use `|` below instead of the default ASCII SOH):
///
/// ```text
/// 8=FIX.4.4|9=12|...
/// ```
///
/// The type associated with each key is governed by either the FIX standard (except when
/// brokers don't implement it correctly) or bilateral agreement, but is known ahead of time.
/// It is the responsibility of the `FixValue` trait to encode to and from native types when
/// interacting with FIX messages.
///
/// This allows users to define custom types, which may be particularly helpful when defining
/// custom enumerations:
///
/// ```rust
/// # use fixity_core::data_types::{FixValue, ParseErrorKind, ParseResult};
/// enum AlgoType {
///     VWAP,
///     TWAP,
/// }
/// impl<'a> FixValue<'a, ()> for AlgoType {
///     fn from_bytes(input: &[u8]) -> ParseResult<Self, ()> {
///         if input.len() != 1 {
///             return Err(());
///         }
///
///         match input {
///             b"V" => Ok(AlgoType::VWAP),
///             b"T" => Ok(AlgoType::TWAP),
///             _ => Err(())
///         }
///     }
///
///     fn to_bytes(&self, buf: &mut [u8]) -> Option<usize> {
///         if buf.len() == 0 {
///             return None;
///         }
///
///         buf[0] = match self {
///             AlgoType::VWAP => b'V',
///             AlgoType::TWAP => b'T',
///         };
///         Some(1)
///     }
/// }
/// ```
///
/// During deserialization (`FixValue::from_bytes()`), the byte slice provided will represent the
/// bytes in between the key-value separator (`=`) and tag delimiter (`SOH` by default). It is
/// expected that all bytes are used during deserialization; an error should be returned if this is
/// not the case.
///
/// During serialization (`FixValue::to_bytes()`), values are written into the buffer and before
/// returning the number of bytes written; it is also safe to modify the buffer and then return
/// `None` to indicate that the value was not successfully written even though the buffer may
/// have been modified. If there is not enough space to encode the value, implementations should
/// also return `None`.
pub trait FixValue<'a, E>
where
    Self: Sized,
{
    /// Deserialize a FIX value type from a byte buffer. The buffer will contain all bytes in
    /// between the key-value separator (`=`) and tag delimiter (`SOH` by default).
    fn from_bytes(input: &'a [u8]) -> ParseResult<Self, E>;

    /// Encode a FIX value type into a byte buffer. Intended to function similar to Rust's
    /// [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) trait. If there are
    /// not enough bytes in the provided buffer, serialization should return a `None` value, at
    /// which point serialization may re-attempted with a larger message buffer.
    ///
    /// It is assumed that serialization is infallible with a sufficiently large message buffer.
    fn to_bytes(&self, buf: &mut [u8]) -> Option<usize>;
}

/// Error types that can be encountered when parsing FIX value strings into the native types.
#[derive(Debug, Eq, PartialEq)]
pub enum ParseErrorKind {
    /// Error while deserializing an unsigned integer
    UnsignedInteger,
    /// Error while deserializing a (potentially) signed integer
    SignedInteger,
    /// Error while deserializing a byte slice into UTF-8 string
    String,
}

/// Result type for deserializing FIX values into the corresponding native types.
pub type ParseResult<O, E = ParseErrorKind> = Result<O, E>;

#[cfg(test)]
mod tests {
    use crate::data_types::FixValue;

    #[test]
    fn u8_roundtrip() {
        let buffer = b"12";

        let value: u8 = FixValue::from_bytes(buffer).unwrap();
        assert_eq!(12, value);

        let buffer = &mut [0u8; 2][..];
        let bytes_written = FixValue::to_bytes(&value, buffer);
        assert_eq!(bytes_written, Some(2));
        assert_eq!(buffer, b"12");
    }
}
