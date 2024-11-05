// ------------------------------------------------------------------------
// Gufo SNMP: Iterator wrapper
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::SnmpOid;
use pyo3::{exceptions::PyValueError, prelude::*};

#[pyclass]
pub struct GetIter {
    start_oid: SnmpOid,
    next_oid: SnmpOid,
    max_repetitions: i64,
}

#[pymethods]
impl GetIter {
    /// Python constructor
    #[new]
    #[pyo3(signature = (oid, max_repetitions = None))]
    fn new(oid: &str, max_repetitions: Option<i64>) -> PyResult<Self> {
        let b_oid = SnmpOid::try_from(oid).map_err(|_| PyValueError::new_err("invalid oid"))?;
        Ok(GetIter {
            start_oid: b_oid.clone(),
            next_oid: b_oid,
            max_repetitions: max_repetitions.unwrap_or_default(),
        })
    }
}

impl GetIter {
    pub fn get_next_oid(&self) -> SnmpOid {
        self.next_oid.clone()
    }
    // Save oid for next request.
    // Return true if next request may be send or return false otherwise
    pub fn set_next_oid(&mut self, oid: &SnmpOid) -> bool {
        if self.start_oid.contains(oid) {
            self.next_oid = oid.clone();
            true
        } else {
            false
        }
    }
    pub fn get_max_repetitions(&self) -> i64 {
        self.max_repetitions
    }
}
