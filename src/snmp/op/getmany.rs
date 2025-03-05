// ------------------------------------------------------------------------
// Gufo SNMP: GetMany operation
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{GetIter, PyOp};
use crate::ber::SnmpOid;
use crate::error::SnmpError;
use crate::snmp::{get::SnmpGet, msg::SnmpPdu, value::SnmpValue};
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
    fn to_python<'py>(
        pdu: &SnmpPdu,
        _iter: Option<&mut GetIter>,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
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
                            .set_item(&var.oid, &var.value)
                            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
                    }
                }
                Ok(dict.as_any().to_owned())
            }
            SnmpPdu::Report(_) => Err(SnmpError::AuthenticationFailed.into()),
            _ => Err(SnmpError::InvalidPdu.into()),
        }
    }
}
