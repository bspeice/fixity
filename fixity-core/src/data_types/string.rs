//! FIX data types representing string/byte-like values
use crate::data_types::{Field, ParseError};

/// FIX data payload, allowed to contain arbitrary data
pub struct DataField<'a> {
    payload: &'a [u8],
}

impl<'a> Field<'a> for DataField<'a> {
    type Type = &'a [u8];

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        Ok(DataField { payload })
    }

    fn value(&self) -> Self::Type {
        self.payload
    }
}

/// FIX data payload, guaranteed to not contain the delimiter (SOH) value
pub struct StringField<'a> {
    payload: &'a [u8],
}

impl<'a> Field<'a> for StringField<'a> {
    type Type = &'a [u8];

    fn new(payload: &'a [u8]) -> Result<Self, ParseError> {
        // TODO: Can't contain SOH
        Ok(StringField { payload })
    }

    fn value(&self) -> Self::Type {
        self.payload
    }
}
