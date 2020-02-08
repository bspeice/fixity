//! Fixity Core
//! Implementation of FIX wire protocol and types; has no opinion on session management,
//! transport protocol, etc.
#![no_std]
#![deny(missing_docs, warnings)]

pub mod data_types;
pub mod utils;
pub mod wire_format;
