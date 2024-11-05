// ------------------------------------------------------------------------
// Gufo SNMP: GetNext operation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{GetIter, PyOp};
use crate::ber::SnmpOid;
use crate::ber::ToPython;
use crate::error::SnmpError;
use crate::snmp::get::SnmpGet;
use crate::snmp::msg::SnmpPdu;
use crate::snmp::value::SnmpValue;
use pyo3::{
    exceptions::{PyStopAsyncIteration, PyValueError},
    prelude::*,
    types::PyTuple,
};

pub struct OpGetNext;

impl<'a> PyOp<'a, SnmpOid> for OpGetNext {
    // obj is iterable[str]
    fn from_python(obj: SnmpOid, request_id: i64) -> PyResult<SnmpPdu<'a>> {
        Ok(SnmpPdu::GetNextRequest(SnmpGet {
            request_id,
            vars: vec![obj],
        }))
    }
    fn to_python(pdu: &SnmpPdu, iter: Option<&mut GetIter>, py: Python) -> PyResult<PyObject> {
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
                            value => Ok(PyTuple::new_bound(
                                py,
                                vec![var.oid.try_to_python(py)?, value.try_to_python(py)?],
                            )
                            .into()),
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
