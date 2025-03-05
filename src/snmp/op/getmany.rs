// ------------------------------------------------------------------------
// Gufo SNMP: GetMany operation
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{GetIter, PyOp};
use crate::ber::{SnmpOid, ToPython};
use crate::error::SnmpError;
use crate::snmp::get::SnmpGet;
use crate::snmp::msg::SnmpPdu;
use crate::snmp::value::SnmpValue;
use pyo3::{exceptions::PyRuntimeError, prelude::*, pybacked::PyBackedStr, types::PyDict};

pub struct OpGetMany;

impl<'a> PyOp<'a, Vec<PyBackedStr>> for OpGetMany {
    // obj is list[str]
    fn from_python(obj: Vec<PyBackedStr>, request_id: i64) -> PyResult<SnmpPdu<'a>> {
        Ok(SnmpPdu::GetRequest(SnmpGet {
            request_id,
            vars: obj
                .into_iter()
                .map(|x| SnmpOid::try_from(x.as_ref()))
                .collect::<Result<Vec<SnmpOid>, SnmpError>>()?,
        }))
    }
    fn to_python(pdu: &SnmpPdu, _iter: Option<&mut GetIter>, py: Python) -> PyResult<PyObject> {
        match pdu {
            SnmpPdu::GetResponse(resp) => {
                // Build resulting dict
                let dict = PyDict::new(py);
                for var in resp.vars.iter() {
                    match &var.value {
                        SnmpValue::Null
                        | SnmpValue::NoSuchObject
                        | SnmpValue::NoSuchInstance
                        | SnmpValue::EndOfMibView => continue,
                        _ => dict
                            .set_item(var.oid.try_to_python(py)?, var.value.try_to_python(py)?)
                            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
                    }
                }
                Ok(dict.into())
            }
            SnmpPdu::Report(_) => Err(SnmpError::AuthenticationFailed.into()),
            _ => Err(SnmpError::InvalidPdu.into()),
        }
    }
}
