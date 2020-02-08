//! FIX types representing integral values
use crate::data_types::{Field, ParseError};
use crate::utils::{atoi, u_atoi};
use nom::combinator::all_consuming;

/// Integer FIX value; allowed to be preceded by an arbitrary number of 0's
pub struct IntField(i64);

impl<'a> Field<'a> for IntField {
    type Type = i64;

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        all_consuming(atoi::<i64, ()>)(payload)
            .or(Err(ParseError::IntField))
            .map(|(_, v)| IntField(v))
    }

    fn value(&self) -> Self::Type {
        self.0
    }
}

/// Unsigned integer FIX value; used as a basis for parsing other related value types
pub struct UnsignedIntField(u64);

impl<'a> Field<'a> for UnsignedIntField {
    type Type = u64;

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        all_consuming(u_atoi::<u64, ()>)(payload)
            .or(Err(ParseError::UnsignedIntField))
            .map(|(_, v)| UnsignedIntField(v))
    }

    fn value(&self) -> Self::Type {
        self.0
    }
}

/// Integer field representing some data length in bytes
pub type LengthField = UnsignedIntField;

/// Integer field representing the number of times a FIX group repeats
pub type NumInGroupField = UnsignedIntField;

/// Integer field representing a monotonically increasing sequence number
pub type SeqNumField = UnsignedIntField;

///
pub struct DayOfMonthField(u8);

impl<'a> Field<'a> for DayOfMonthField {
    type Type = u8;

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        let (_, v) =
            all_consuming(u_atoi::<u64, ()>)(payload).or(Err(ParseError::DayOfMonthField))?;

        if 1 <= v && v <= 31 {
            Ok(DayOfMonthField(v as u8))
        } else {
            Err(ParseError::DayOfMonthField)
        }
    }

    fn value(&self) -> Self::Type {
        self.0
    }
}
