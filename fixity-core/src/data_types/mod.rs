//! Value types associated with parsing FIX fields
pub mod int;
pub mod string;

use nom::error::{ParseError as NParseError};
use nom::InputLength;
use nom::{IResult as NResult};

/// FIX data types for the key-value pairs. These data types assume that the provided input
/// is exactly the expected value; unconsumed input is considered an error.
// The lifetime declared here is needed to prove that fields within the struct implementation
// can't outlive the original payload.
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
pub enum ParseError<'a> {
    /// Error encountered while parsing an integer field
    IntField,
    /// Error encountered while parsing an unsigned integer field
    UnsignedIntField,
    /// Error encountered while parsing a day of month field
    DayOfMonthField,
    /// Error encountered while parsing a data field
    DataField,
    /// Error encountered while parsing a string field
    StringField,
    /// Input payload was not fully consumed
    UnusedInput(&'a [u8]),
}
