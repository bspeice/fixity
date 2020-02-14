//! Value types associated with parsing FIX fields
pub mod int;
pub mod string;

/// FIX data types for the key-value pairs. These data types assume that the provided input
/// is exactly the expected value; unconsumed input is considered an error.
pub trait Field<'a>
where
    Self: Sized,
{
    /// Native type that this field will parse values into
    type Output;

    /// Parse and create a new field from an ASCII byte stream
    fn parse(payload: &'a [u8]) -> Result<Self::Output, ParseError>;
}

/// Encountered an error or ill-formed FIX message during parsing
#[derive(Debug, PartialEq)]
pub enum ParseError {}
