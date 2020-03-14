//! FIX types representing integral values
use crate::data_types::{Field, ParseError};
use nom::combinator::{all_consuming, verify};
use crate::wire_format::{tagnum, u_atoi, atoi};

/// Sequence of digits without commas or decimals and optional sign character (ASCII characters
/// "-" and "0" - "9" ). The sign character utilizes one byte (i.e. positive int is "99999" while
/// negative int is "-99999"). Note that int values may contain leading zeros (e.g. "00023" = "23").
pub struct IntField;

impl<'a> Field<'a> for IntField {
    type Output = i64;

    fn parse(payload: &'a [u8]) -> Result<Self::Output, ParseError> {
        all_consuming(atoi)(payload)
            .map(|(_, value)| value)
            .or(Err(ParseError::Int))
    }
}

// Used as a basis for other fields that need equivalent behavior
#[doc(hidden)]
pub struct UnsignedIntField;

impl<'a> Field<'a> for UnsignedIntField {
    type Output = u64;

    fn parse(payload: &'a [u8]) -> Result<Self::Output, ParseError> {
        all_consuming(u_atoi)(payload)
            .map(|(_, value)| value)
            .or(Err(ParseError::UnsignedInt))
    }
}

/// int field representing the length in bytes. Value must be positive.
pub type LengthField = UnsignedIntField;

/// int field representing a message sequence number. Value must be positive.
pub type SeqNumField = UnsignedIntField;

/// int field representing the number of entries in a repeating group. Value must be positive.
pub type NumInGroupField = UnsignedIntField;

/// int field representing a day during a particular monthy (values 1 to 31).
pub struct DayOfMonthField;

impl<'a> Field<'a> for DayOfMonthField {
    type Output = u8;

    fn parse(payload: &'a [u8]) -> Result<Self::Output, ParseError> {
        all_consuming(verify(u_atoi, |v| 1 <= *v && *v <= 31))(payload)
            .map(|(_, value)| value)
            .or(Err(ParseError::DayOfMonth))
    }
}

/// int field representing a field's tag number when using FIX "Tag=Value" syntax. Value must be
/// positive and may not contain leading zeros.
pub struct TagNumField;

impl<'a> Field<'a> for TagNumField {
    type Output = u16;

    fn parse(payload: &'a [u8]) -> Result<Self::Output, ParseError> {
        all_consuming(tagnum)(payload)
            .map(|(_, value)| value)
            .or(Err(ParseError::TagNum))
    }
}

#[cfg(test)]
mod tests {
    use crate::data_types::int::{IntField, TagNumField, UnsignedIntField, DayOfMonthField};
    use crate::data_types::Field;

    #[test]
    fn int_field_simple() {
        assert_eq!(IntField::parse(b"1234"), Ok(1234));
        assert_eq!(IntField::parse(b"-1234"), Ok(-1234));
        assert_eq!(IntField::parse(b"001234"), Ok(1234));
        assert_eq!(IntField::parse(b"-001234"), Ok(-1234));
        assert_eq!(IntField::parse(b"0"), Ok(0));
        assert_eq!(IntField::parse(b"-0"), Ok(0));

        assert!(IntField::parse(b"").is_err());
        assert!(IntField::parse(b"-").is_err());
        assert!(IntField::parse(b"-1234|").is_err());
        assert!(IntField::parse(&[b'1', 0x01][..]).is_err());
        assert!(IntField::parse(b"00-24").is_err());
    }

    #[test]
    fn uint_field_simple() {
        assert_eq!(UnsignedIntField::parse(b"1234"), Ok(1234));
        assert_eq!(UnsignedIntField::parse(b"001234"), Ok(1234));
        assert_eq!(UnsignedIntField::parse(b"0"), Ok(0));

        assert!(UnsignedIntField::parse(b"").is_err());
        assert!(UnsignedIntField::parse(b"-12").is_err());
        assert!(UnsignedIntField::parse(b"1234|").is_err());
        assert!(UnsignedIntField::parse(&[b'1', 0x01][..]).is_err());
        assert!(UnsignedIntField::parse(b"00|24").is_err());
    }

    #[test]
    fn day_of_month_field_simple() {
        assert_eq!(DayOfMonthField::parse(b"31"), Ok(31));
        assert_eq!(DayOfMonthField::parse(b"1"), Ok(1));

        assert!(DayOfMonthField::parse(b"0").is_err());
        assert!(DayOfMonthField::parse(b"-1").is_err());
        assert!(DayOfMonthField::parse(b"32").is_err());
        assert!(DayOfMonthField::parse(b"257").is_err());
    }

    #[test]
    fn tagnum_field_simple() {
        assert_eq!(TagNumField::parse(b"1234"), Ok(1234));

        assert!(TagNumField::parse(b"0").is_err());
        assert!(TagNumField::parse(b"").is_err());
        assert!(TagNumField::parse(b"-1234").is_err());
        assert!(TagNumField::parse(b"1234|").is_err());
        assert!(TagNumField::parse(&[b'1', 0x01][..]).is_err());
        assert!(TagNumField::parse(b"001234").is_err());
    }
}
