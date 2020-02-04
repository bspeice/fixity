//! FIX types representing integral values
use crate::data_types::{Field, ParseError};
use crate::parsers::{atoi, u_atoi};

/// Integer FIX value; allowed to be preceded by an arbitrary number of 0's
pub struct IntField {
    value: i64,
}

impl<'a> Field<'a> for IntField {
    type Type = i64;

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        let (rem, value) = atoi::<i64, ()>(payload).or(Err(ParseError::Unknown))?;
        if rem.is_empty() {
            Ok(IntField { value })
        } else {
            Err(ParseError::Unknown)
        }
    }

    fn value(&self) -> Self::Type {
        self.value
    }
}

/// Unsigned integer FIX value; used as a basis for parsing other related value types
pub struct UnsignedIntField {
    value: u64,
}

impl<'a> Field<'a> for UnsignedIntField {
    type Type = u64;

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        let (rem, value) = u_atoi::<u64, ()>(payload).or(Err(ParseError::Unknown))?;
        if rem.is_empty() {
            Ok(UnsignedIntField { value })
        } else {
            Err(ParseError::Unknown)
        }
    }

    fn value(&self) -> Self::Type {
        self.value
    }
}

/// Integer field representing some data length in bytes
pub type LengthField = UnsignedIntField;

/// Integer field representing the number of times a FIX group repeats
pub type NumInGroupField = UnsignedIntField;

/// Integer field representing a monotonically increasing sequence number
pub type SeqNumField = UnsignedIntField;
