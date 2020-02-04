//! Value types associated with parsing FIX fields
pub mod int;
pub mod string;

/// FIX value trait. Encapsulates behavior related to the "value" part of key-value pairs in FIX.
pub trait Field<'a>
where
    Self: Sized,
{
    /// Native type that this field will parse values into
    type Type;

    /// Parse and create a new field from an ASCII byte stream
    fn new(payload: &'a [u8]) -> Result<Self, ParseError>;

    /// Get the native value of this field after serialization
    fn value(&self) -> Self::Type;
}

/// Encountered an error or ill-formed FIX message during parsing
pub enum ParseError {
    /// Other errors encountered during parsing
    Unknown,
}
