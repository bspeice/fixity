//! FIX data types representing string/byte-like values
use crate::data_types::{Field, ParseError};
use core::marker::PhantomData;
use nom::bytes::complete::take_till1;
use nom::combinator::all_consuming;
use nom::Err as NErr;
use typenum::{Unsigned, P1};

/// FIX data payload, allowed to contain arbitrary data
pub struct DataField<'a>(&'a [u8]);

impl<'a> Field<'a> for DataField<'a> {
    type Type = &'a [u8];

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        if payload.len() > 0 {
            Ok(DataField(payload))
        } else {
            Err(ParseError::DataField)
        }
    }

    fn value(&self) -> Self::Type {
        self.0
    }
}

/// FIX data payload that cannot contain the delimiter value.
pub struct DelimitedStringField<'a, T>(&'a [u8], PhantomData<T>)
where
    T: Unsigned;

impl<'a, T> Field<'a> for DelimitedStringField<'a, T>
where
    T: Unsigned,
{
    type Type = &'a [u8];

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        all_consuming(take_till1::<_, &[u8], (&[u8], _)>(|b| b == T::to_u8()))(payload)
            .map_err(|e| match e {
                NErr::Error(e) => ParseError::UnusedInput(e.0),
                _ => unreachable!(),
            })
            .map(|(_, v)| DelimitedStringField(v, PhantomData))
    }

    fn value(&self) -> Self::Type {
        self.0
    }
}

/// FIX data payload that cannot contain the FIX delimiter (ASCII SOH) value
pub type StringField<'a> = DelimitedStringField<'a, P1>;
