// ------------------------------------------------------------------------
// Gufo SNMP: SnmpError
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use pyo3::{
    create_exception,
    exceptions::{PyException, PyNotImplementedError},
    PyErr,
};

#[derive(Debug)]
pub enum SnmpError {
    /// Too short
    Incomplete,
    /// Other tag is expected
    UnexpectedTag, // @todo: Expand
    ///
    InvalidTagFormat,
    /// Unknown PDU type
    UnknownPdu,
    /// Malformed PDU
    InvalidPdu,
    /// Malformed variable data
    InvalidData,
    /// Unimplemented tag
    UnsupportedTag(String),
    /// Data beyound PDU
    TrailingData,
    /// Unsupported SNMP version
    InvalidVersion(u8),
    /// Buffer is too small
    OutOfBuffer,
    /// Not implemented still
    NotImplemented,
    /// No such instance
    NoSuchInstance,
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

create_exception!(
    _fast,
    PySnmpError,
    PyException,
    "Base class for Gufo SNMP errors"
);
create_exception!(
    _fast,
    PySnmpDecodeError,
    PySnmpError,
    "Message decoding error"
);
create_exception!(
    _fast,
    PySnmpEncodeError,
    PySnmpError,
    "Message encoding error"
);
create_exception!(
    _fast,
    PyNoSuchInstance,
    PySnmpError,
    "Requested OID is not found"
);

impl From<SnmpError> for PyErr {
    fn from(value: SnmpError) -> PyErr {
        match value {
            SnmpError::Incomplete => PySnmpDecodeError::new_err("incomplete"),
            SnmpError::UnexpectedTag => PySnmpDecodeError::new_err("unexpected tag"),
            SnmpError::InvalidTagFormat => PySnmpDecodeError::new_err("invalid tag format"),
            SnmpError::UnknownPdu => PySnmpDecodeError::new_err("unknown pdu"),
            SnmpError::InvalidPdu => PySnmpDecodeError::new_err("invalid pdu"),
            SnmpError::InvalidData => PySnmpDecodeError::new_err("invalid data"),
            SnmpError::UnsupportedTag(e) => {
                PySnmpDecodeError::new_err(format!("Unsupported tag: {}", e))
            }
            SnmpError::TrailingData => PySnmpDecodeError::new_err("trailing data"),
            SnmpError::InvalidVersion(v) => {
                PySnmpDecodeError::new_err(format!("unsupported version: {}", v))
            }
            SnmpError::OutOfBuffer => PySnmpEncodeError::new_err("out of buffer"),
            SnmpError::NotImplemented => PyNotImplementedError::new_err("not implemented"),
            SnmpError::NoSuchInstance => PyNoSuchInstance::new_err("no such instance"),
        }
    }
}
