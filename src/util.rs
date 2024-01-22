// ------------------------------------------------------------------------
// Gufo SNMP: Utilities
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::auth::{AuthKey, SnmpAuth};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

// Convert key to master key
#[pyfunction]
pub fn get_master_key(py: Python, alg: u8, passwd: &[u8]) -> PyResult<PyObject> {
    let auth = AuthKey::new(alg)?;
    let mut out = vec![0u8; auth.get_key_size()];
    auth.password_to_master(passwd, &mut out);
    Ok(PyBytes::new(py, &out).into())
}
