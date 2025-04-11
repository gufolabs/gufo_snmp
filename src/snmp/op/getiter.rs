// ------------------------------------------------------------------------
// Gufo SNMP: Iterator wrapper
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{SnmpOid, objectid::OidStorage};
use pyo3::{exceptions::PyValueError, prelude::*};

#[pyclass]
pub struct GetIter {
    start_oid: Vec<u8>,
    next_oid: Vec<u8>,
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
            start_oid: (&b_oid).into(),
            next_oid: (&b_oid).into(),
            max_repetitions: max_repetitions.unwrap_or_default(),
        })
    }
}

impl GetIter {
    pub fn get_next_oid<'a>(&self) -> SnmpOid<'a> {
        self.next_oid.as_owned()
    }
    // Save oid for next request.
    // Return true if next request may be send or return false otherwise
    pub fn set_next_oid(&mut self, oid: &SnmpOid) -> bool {
        if self.start_oid.as_borrowed().starts_with(oid) {
            self.next_oid.store(oid);
            true
        } else {
            false
        }
    }
    pub fn get_max_repetitions(&self) -> i64 {
        self.max_repetitions
    }
}
