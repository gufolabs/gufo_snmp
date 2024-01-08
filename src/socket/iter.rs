// ------------------------------------------------------------------------
// Gufo SNMP: Iterator wrappers
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::SnmpOid;
use pyo3::{exceptions::PyValueError, prelude::*};

#[pyclass]
pub struct GetNextIter {
    start_oid: SnmpOid,
    next_oid: SnmpOid,
}

#[pyclass]
pub struct GetBulkIter {
    start_oid: SnmpOid,
    next_oid: SnmpOid,
    max_repetitions: i64,
}

#[pymethods]
impl GetNextIter {
    /// Python constructor
    #[new]
    fn new(oid: &str) -> PyResult<Self> {
        let b_oid = SnmpOid::try_from(oid).map_err(|_| PyValueError::new_err("invalid oid"))?;
        Ok(GetNextIter {
            start_oid: b_oid.clone(),
            next_oid: b_oid,
        })
    }
}

impl GetNextIter {
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
}

#[pymethods]
impl GetBulkIter {
    /// Python constructor
    #[new]
    fn new(oid: &str, max_repetitions: i64) -> PyResult<Self> {
        let b_oid = SnmpOid::try_from(oid).map_err(|_| PyValueError::new_err("invalid oid"))?;
        Ok(GetBulkIter {
            start_oid: b_oid.clone(),
            next_oid: b_oid,
            max_repetitions,
        })
    }
}

impl GetBulkIter {
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
    //
    pub fn get_max_repetitions(&self) -> i64 {
        self.max_repetitions
    }
}
