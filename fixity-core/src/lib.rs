//! Fixity Core
//! Implementation of FIX wire protocol and types; has no opinion on session management,
//! transport protocol, etc.
#![no_std]
#![deny(missing_docs)]

pub mod data_types;
pub mod wire_format;

/// The default FIX delimiter token
pub const SOH: u8 = 0x01;
