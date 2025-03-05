// ------------------------------------------------------------------------
// Gufo SNMP: Module definition
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use pyo3::prelude::*;
pub mod auth;
pub mod ber;
pub mod buf;
pub mod error;
mod privacy;
pub mod reqid;
pub mod snmp;
mod socket;
mod util;

/// Module index
#[pymodule]
#[pyo3(name = "_fast")]
fn gufo_snmp(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("SnmpError", py.get_type::<error::PySnmpError>())?;
    m.add("SnmpEncodeError", py.get_type::<error::PySnmpEncodeError>())?;
    m.add("SnmpDecodeError", py.get_type::<error::PySnmpDecodeError>())?;
    m.add("SnmpAuthError", py.get_type::<error::PySnmpAuthError>())?;
    m.add("NoSuchInstance", py.get_type::<error::PyNoSuchInstance>())?;
    m.add_class::<socket::SnmpV1ClientSocket>()?;
    m.add_class::<socket::SnmpV2cClientSocket>()?;
    m.add_class::<socket::SnmpV3ClientSocket>()?;
    m.add_class::<snmp::op::GetIter>()?;
    m.add_function(wrap_pyfunction!(util::get_master_key, m)?)?;
    m.add_function(wrap_pyfunction!(util::get_localized_key, m)?)?;
    Ok(())
}
