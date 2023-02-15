// ------------------------------------------------------------------------
// Gufo SNMP: Module definition
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use pyo3::prelude::*;
pub mod ber;
pub mod buf;
pub mod error;
pub mod snmp;
mod socket;

/// Module index
#[pymodule]
#[pyo3(name = "_fast")]
fn gufo_ping(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("SnmpError", py.get_type::<error::PySnmpError>())?;
    m.add("SnmpEncodeError", py.get_type::<error::PySnmpEncodeError>())?;
    m.add("SnmpDecodeError", py.get_type::<error::PySnmpDecodeError>())?;
    m.add("NoSuchInstance", py.get_type::<error::PyNoSuchInstance>())?;
    m.add_class::<socket::SnmpClientSocket>()?;
    m.add_class::<socket::GetNextIter>()?;
    m.add_class::<socket::GetBulkIter>()?;
    Ok(())
}
