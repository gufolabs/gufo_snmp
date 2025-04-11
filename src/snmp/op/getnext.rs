// ------------------------------------------------------------------------
// Gufo SNMP: GetNext operation
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{GetIter, PyOp};
use crate::ber::SnmpOid;
use crate::error::SnmpError;
use crate::snmp::{get::SnmpGet, msg::SnmpPdu, value::SnmpValue};
use pyo3::{
    exceptions::{PyStopAsyncIteration, PyValueError},
    prelude::*,
    types::PyTuple,
};

pub struct OpGetNext;

impl<'a> PyOp<'a, SnmpOid<'a>> for OpGetNext {
    // obj is iterable[str]
    fn from_python(obj: SnmpOid<'a>, request_id: i64) -> PyResult<SnmpPdu<'a>> {
        Ok(SnmpPdu::GetNextRequest(SnmpGet {
            request_id,
            vars: vec![obj],
        }))
    }
    fn to_python<'py>(
        pdu: &SnmpPdu,
        iter: Option<&mut GetIter>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let b_iter = iter.ok_or_else(|| PyValueError::new_err("GetIter expected"))?;
        match pdu {
            SnmpPdu::GetResponse(resp) => {
                // Check varbinds size
                match resp.vars.len() {
                    // Empty response, stop iteration
                    0 => Err(PyStopAsyncIteration::new_err("stop")),
                    // Return value
                    1 => {
                        // Extract iterator
                        let var = &resp.vars[0];
                        // Check if we can continue
                        if !b_iter.set_next_oid(&var.oid) {
                            return Err(PyStopAsyncIteration::new_err("stop"));
                        }
                        // v1 may return Null at end of mib
                        match &var.value {
                            SnmpValue::EndOfMibView | SnmpValue::Null => {
                                Err(PyStopAsyncIteration::new_err("stop"))
                            }
                            value => Ok(PyTuple::new(
                                py,
                                vec![var.oid.into_pyobject(py)?, value.into_pyobject(py)?],
                            )?
                            .as_any()
                            .to_owned()),
                        }
                    }
                    // Multiple response, surely an error
                    _ => Err(SnmpError::InvalidPdu.into()),
                }
            }
            SnmpPdu::Report(_) => Err(SnmpError::AuthenticationFailed.into()),
            _ => Err(SnmpError::InvalidPdu.into()),
        }
    }
}
