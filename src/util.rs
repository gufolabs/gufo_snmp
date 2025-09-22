// ------------------------------------------------------------------------
// Gufo SNMP: Utilities
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::auth::{AuthKey, SnmpAuth};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

// Convert key to master key
#[pyfunction]
pub fn get_master_key(py: Python, alg: u8, passwd: &[u8]) -> PyResult<Py<PyAny>> {
    let auth = AuthKey::new(alg)?;
    let mut out = vec![0u8; auth.get_key_size()];
    auth.password_to_master(passwd, &mut out);
    Ok(PyBytes::new(py, &out).into())
}

// Convert master key to localized key
#[pyfunction]
pub fn get_localized_key(
    py: Python,
    alg: u8,
    master_key: &[u8],
    engine_id: &[u8],
) -> PyResult<Py<PyAny>> {
    let auth = AuthKey::new(alg)?;
    let ks = auth.get_key_size();
    if master_key.len() != ks {
        return Err(PyValueError::new_err("invalid key size"));
    }
    let mut out = vec![0u8; auth.get_key_size()];
    auth.localize(master_key, engine_id, &mut out);
    Ok(PyBytes::new(py, &out).into())
}
