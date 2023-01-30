// ------------------------------------------------------------------------
// Gufo SNMP: Module definition
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use pyo3::prelude::*;
pub(crate) mod ber;
pub(crate) mod buf;
pub(crate) mod error;
pub(crate) mod snmp;
mod socket;

/// Module index
#[pymodule]
#[pyo3(name = "_fast")]
fn gufo_ping(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("SNMPError", py.get_type::<error::SNMPError>())?;
    m.add("SNMPEncodeError", py.get_type::<error::SNMPEncodeError>())?;
    m.add("SNMPDecodeError", py.get_type::<error::SNMPDecodeError>())?;
    m.add("NoSuchInstance", py.get_type::<error::NoSuchInstance>())?;
    m.add_class::<socket::SnmpClientSocket>()?;
    m.add_class::<socket::GetNextIter>()?;
    m.add_class::<socket::GetBulkIter>()?;
    Ok(())
}
