//! FIX data types representing string/byte-like values
use crate::data_types::{Field, ParseError};

/// string field containing raw data with no format or content restrictions. Data fields are always
/// immediately preceded by a length field. The length field should specify the number of bytes of
/// the value of the data field (up to but not including the terminating SOH).
///
/// Caution: the value of one of these fields may contain the delimiter (SOH) character. Note that
/// the value specified for this field should be followed by the delimiter (SOH) character as all
/// fields are terminated with an "SOH".
pub struct DataField;

impl<'a> Field<'a> for DataField {
    type Output = &'a [u8];

    fn parse(_payload: &'a [u8]) -> Result<Self::Output, ParseError> {
        unimplemented!()
    }
}

/// Alpha-numeric free format strings, can include any character or punctuation except the
/// delimiter. All String fields are case sensitive (i.e. morstatt != Morstatt).
pub struct StringField;

impl<'a> Field<'a> for StringField {
    type Output = &'a [u8];

    fn parse(_payload: &'a [u8]) -> Result<Self::Output, ParseError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::data_types::string::{DataField, StringField};
    use crate::data_types::Field;
    use crate::SOH;

    #[test]
    fn data_field_simple() {
        assert_eq!(DataField::parse(b"abc"), Ok(&b"abc"[..]));
        assert_eq!(DataField::parse(&[SOH][..]), Ok(&[SOH][..]));
        assert!(DataField::parse(b"").is_err());
    }

    #[test]
    fn string_field_simple() {
        assert_eq!(StringField::parse(b"abc"), Ok(&b"abc"[..]));
        assert_eq!(StringField::parse(&[0x14][..]), Ok(&[0x14][..]));
        assert!(StringField::parse(&[0x14, SOH][..]).is_err());
    }
}
