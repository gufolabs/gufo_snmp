// ------------------------------------------------------------------------
// Gufo Snmp: SnmpError
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

#[derive(Debug)]
pub(crate) enum SnmpError {
    /// Too short
    Incomplete,
    /// Other tag is expected
    UnexpectedTag,
    ///
    InvalidTagFormat,
    /// Uknown SNMP version
    UnknownVersion,
    /// Unknown PDU type
    UnknownPdu,
    /// Malformed PDU
    InvalidPdu,
    /// Malformed variable data
    InvalidData,
    /// Unknown data type
    UnsupportedData,
    /// Data beyound PDU
    TrailingData,
}

impl From<nom::Err<SnmpError>> for SnmpError {
    fn from(value: nom::Err<SnmpError>) -> SnmpError {
        match value {
            nom::Err::Incomplete(_) => SnmpError::Incomplete,
            nom::Err::Error(e) => e,
            nom::Err::Failure(e) => e,
        }
    }
}

impl From<SnmpError> for nom::Err<SnmpError> {
    fn from(value: SnmpError) -> nom::Err<SnmpError> {
        nom::Err::Failure(value)
    }
}
