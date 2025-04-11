// ------------------------------------------------------------------------
// Gufo SNMP: GetBulk operation
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{GetIter, PyOp};
use crate::ber::SnmpOid;
use crate::error::SnmpError;
use crate::snmp::getbulk::SnmpGetBulk;
use crate::snmp::msg::SnmpPdu;
use crate::snmp::value::SnmpValue;
use pyo3::{
    exceptions::{PyRuntimeError, PyStopAsyncIteration, PyValueError},
    prelude::*,
    types::{PyList, PyTuple},
};

pub struct OpGetBulk;

impl<'a> PyOp<'a, (SnmpOid<'a>, i64)> for OpGetBulk {
    // obj is iterable[str]
    fn from_python(obj: (SnmpOid<'a>, i64), request_id: i64) -> PyResult<SnmpPdu<'a>> {
        let (oid, max_repetitions) = obj;
        Ok(SnmpPdu::GetBulkRequest(SnmpGetBulk {
            request_id,
            non_repeaters: 0,
            max_repetitions,
            vars: vec![oid],
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
                if resp.vars.is_empty() {
                    return Err(PyStopAsyncIteration::new_err("stop"));
                }
                let list = PyList::empty(py);
                for var in resp.vars.iter() {
                    match &var.value {
                        SnmpValue::Null
                        | SnmpValue::NoSuchObject
                        | SnmpValue::NoSuchInstance
                        | SnmpValue::EndOfMibView => continue,
                        _ => {
                            // Check if we can continue
                            if !b_iter.set_next_oid(&var.oid) {
                                let _ = list.append(py.None());
                                break;
                            }
                            // Append to list
                            list.append(PyTuple::new(
                                py,
                                vec![var.oid.into_pyobject(py)?, var.value.into_pyobject(py)?],
                            )?)
                            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
                        }
                    }
                }
                if list.is_empty() {
                    return Err(PyStopAsyncIteration::new_err("stop"));
                }
                Ok(list.as_any().to_owned())
            }
            SnmpPdu::Report(_) => Err(SnmpError::AuthenticationFailed.into()),
            _ => Err(SnmpError::InvalidPdu.into()),
        }
    }
}
