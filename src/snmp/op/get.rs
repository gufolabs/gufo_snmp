// ------------------------------------------------------------------------
// Gufo SNMP: Get operation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{GetIter, PyOp};
use crate::ber::{SnmpOid, ToPython};
use crate::error::SnmpError;
use crate::snmp::get::SnmpGet;
use crate::snmp::msg::SnmpPdu;
use crate::snmp::value::SnmpValue;
use pyo3::{prelude::*, types::PyNone};

pub struct OpGet;

impl<'a> PyOp<'a, &'a str> for OpGet {
    // Obj is str
    fn from_python(obj: &'a str, request_id: i64) -> PyResult<SnmpPdu<'a>> {
        Ok(SnmpPdu::GetRequest(SnmpGet {
            request_id,
            vars: vec![SnmpOid::try_from(obj)?],
        }))
    }
    fn to_python(pdu: &SnmpPdu, _iter: Option<&mut GetIter>, py: Python) -> PyResult<PyObject> {
        match pdu {
            SnmpPdu::GetResponse(resp) => {
                // Check varbinds size
                match resp.vars.len() {
                    // Empty response, return None
                    0 => Ok(PyNone::get_bound(py).into_py(py)),
                    // Return value
                    1 => {
                        let var = &resp.vars[0];
                        let value = &var.value;
                        match value {
                            SnmpValue::NoSuchObject
                            | SnmpValue::NoSuchInstance
                            | SnmpValue::EndOfMibView => Err(SnmpError::NoSuchInstance.into()),
                            SnmpValue::Null => Ok(PyNone::get_bound(py).into_py(py)),
                            _ => Ok(value.try_to_python(py)?),
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
