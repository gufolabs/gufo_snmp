// ------------------------------------------------------------------------
// Gufo SNMP: Get+Report operation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{GetIter, PyOp};
use crate::snmp::get::SnmpGet;
use crate::snmp::msg::SnmpPdu;
use pyo3::{prelude::*, types::PyNone};

pub struct OpRefresh;

impl<'a> PyOp<'a, ()> for OpRefresh {
    // Obj is str
    fn from_python(_obj: (), request_id: i64) -> PyResult<SnmpPdu<'a>> {
        Ok(SnmpPdu::GetRequest(SnmpGet {
            request_id,
            vars: vec![],
        }))
    }
    fn to_python(_pdu: &SnmpPdu, _iter: Option<&mut GetIter>, py: Python) -> PyResult<PyObject> {
        Ok(PyNone::get(py).into_py(py))
    }
}
