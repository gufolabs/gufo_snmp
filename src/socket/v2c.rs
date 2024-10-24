// ------------------------------------------------------------------------
// Gufo SNMP: SnmpV2cClientSocket
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::iter::{GetBulkIter, GetNextIter};
use super::op::{SnmpSocket, SupportsGet, SupportsGetBulk, SupportsGetMany, SupportsGetNext};
use super::transport::SnmpTransport;
use crate::ber::{SnmpOid, ToPython};
use crate::error::{SnmpError, SnmpResult};
use crate::reqid::RequestId;
use crate::snmp::get::SnmpGet;
use crate::snmp::getbulk::SnmpGetBulk;
use crate::snmp::msg::{SnmpMessage, SnmpV2cMessage};
use crate::snmp::pdu::SnmpPdu;
use crate::snmp::value::SnmpValue;
use pyo3::{
    exceptions::{PyRuntimeError, PyStopAsyncIteration},
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};
use std::os::fd::AsRawFd;

/// Python class wrapping socket implementation
#[pyclass]
pub struct SnmpV2cClientSocket {
    io: SnmpTransport,
    community: String,
    request_id: RequestId,
}

#[pymethods]
impl SnmpV2cClientSocket {
    /// Python constructor
    #[new]
    fn new(
        addr: String,
        community: String,
        tos: u32,
        send_buffer_size: usize,
        recv_buffer_size: usize,
        timeout_ns: u64,
    ) -> PyResult<Self> {
        // Transport
        let io = SnmpTransport::new(addr, tos, send_buffer_size, recv_buffer_size, timeout_ns)?;
        //
        Ok(Self {
            io,
            community,
            request_id: RequestId::default(),
        })
    }
    /// Get socket's file descriptor
    fn get_fd(&self) -> PyResult<i32> {
        Ok(self.io.as_raw_fd())
    }
    // .get()
    // Prepare send GET request with single oid and receive reply
    fn get(&mut self, py: Python, oid: &str) -> PyResult<Option<PyObject>> {
        SupportsGet::get(self, py, oid)
    }
    // Prepare and send GET request with single oid
    fn send_get(&mut self, py: Python, oid: &str) -> PyResult<()> {
        Ok(SupportsGet::send_get(self, py, oid)?)
    }
    // Try to receive GETRESPONSE
    fn recv_get(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        SupportsGet::recv_get(self, py)
    }
    // .get_many()
    // Prepare and send GET request with multiple oids and receive reply
    fn get_many(&mut self, py: Python, oids: Vec<&str>) -> PyResult<PyObject> {
        SupportsGetMany::get_many(self, py, oids)
    }
    // Prepare and send GET request with multiple oids
    fn send_get_many(&mut self, py: Python, oids: Vec<&str>) -> PyResult<()> {
        Ok(SupportsGetMany::send_get_many(self, py, oids)?)
    }
    fn recv_get_many(&mut self, py: Python) -> PyResult<PyObject> {
        SupportsGetMany::recv_get_many(self, py)
    }
    // .get_next()
    fn get_next(&mut self, py: Python, iter: &mut GetNextIter) -> PyResult<(PyObject, PyObject)> {
        SupportsGetNext::get_next(self, py, iter)
    }
    fn send_get_next(&mut self, py: Python, iter: &GetNextIter) -> PyResult<()> {
        Ok(SupportsGetNext::send_get_next(self, py, iter)?)
    }
    fn recv_get_next(
        &mut self,
        py: Python,
        iter: &mut GetNextIter,
    ) -> PyResult<(PyObject, PyObject)> {
        SupportsGetNext::recv_get_next(self, py, iter)
    }
    // .get_bulk()
    fn get_bulk(&mut self, py: Python, iter: &mut GetBulkIter) -> PyResult<PyObject> {
        SupportsGetBulk::get_bulk(self, py, iter)
    }
    // Send GetBulk request according to iter
    fn send_get_bulk(&mut self, py: Python, iter: &GetBulkIter) -> PyResult<()> {
        Ok(SupportsGetBulk::send_get_bulk(self, py, iter)?)
    }
    // Try to receive GETRESPONSE for GETBULK
    fn recv_get_bulk(&mut self, iter: &mut GetBulkIter, py: Python) -> PyResult<PyObject> {
        SupportsGetBulk::recv_get_bulk(self, py, iter)
    }
}

impl SnmpSocket for SnmpV2cClientSocket {
    type Message<'a> = SnmpV2cMessage<'a>;

    fn get_transport(&self) -> &SnmpTransport {
        &self.io
    }

    fn get_request_id(&mut self) -> &mut RequestId {
        &mut self.request_id
    }

    fn authenticate(&self, msg: &Self::Message<'_>) -> bool {
        msg.community == self.community.as_bytes()
    }
}

impl SupportsGet for SnmpV2cClientSocket {
    fn request<'a>(&'a self, oid: &str, request_id: i64) -> SnmpResult<Self::Message<'a>> {
        Ok(Self::Message {
            community: self.community.as_bytes(),
            pdu: SnmpPdu::GetRequest(SnmpGet {
                request_id,
                vars: vec![SnmpOid::try_from(oid)?],
            }),
        })
    }
    fn parse(py: Python, msg: &Self::Message<'_>) -> PyResult<Option<PyObject>> {
        if let Some(resp) = msg.as_pdu().as_getresponse() {
            // Check varbinds size
            match resp.vars.len() {
                // Empty response, return None
                0 => Ok(None),
                // Return value
                1 => {
                    let var = &resp.vars[0];
                    let value = &var.value;
                    match value {
                        SnmpValue::NoSuchObject
                        | SnmpValue::NoSuchInstance
                        | SnmpValue::EndOfMibView => Err(SnmpError::NoSuchInstance.into()),
                        SnmpValue::Null => Ok(None),
                        _ => Ok(Some(value.try_to_python(py)?)),
                    }
                }
                // Multiple response, surely an error
                _ => Err(SnmpError::InvalidPdu.into()),
            }
        } else {
            Err(SnmpError::InvalidPdu.into())
        }
    }
}

impl SupportsGetMany for SnmpV2cClientSocket {
    fn request<'a>(&'a self, oids: Vec<&str>, request_id: i64) -> SnmpResult<Self::Message<'a>> {
        Ok(Self::Message {
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetRequest(SnmpGet {
                request_id,
                vars: oids
                    .into_iter()
                    .map(SnmpOid::try_from)
                    .collect::<Result<Vec<SnmpOid>, SnmpError>>()?,
            }),
        })
    }
    fn parse(py: Python, msg: &Self::Message<'_>) -> PyResult<PyObject> {
        if let Some(resp) = msg.as_pdu().as_getresponse() {
            // Build resulting dict
            let dict = PyDict::new_bound(py);
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
            return Ok(dict.into());
        }
        Err(SnmpError::InvalidPdu.into())
    }
}

impl SupportsGetNext for SnmpV2cClientSocket {
    fn request<'a>(&'a self, iter: &GetNextIter, request_id: i64) -> SnmpResult<Self::Message<'a>> {
        Ok(Self::Message {
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetNextRequest(SnmpGet {
                request_id,
                vars: vec![iter.get_next_oid()],
            }),
        })
    }
    fn parse(
        py: Python,
        msg: &Self::Message<'_>,
        iter: &mut GetNextIter,
    ) -> PyResult<(PyObject, PyObject)> {
        if let Some(resp) = msg.as_pdu().as_getresponse() {
            // Check varbinds size
            match resp.vars.len() {
                // Empty response, stop iteration
                0 => return Err(PyStopAsyncIteration::new_err("stop")),
                // Return value
                1 => {
                    let var = &resp.vars[0];
                    // Check if we can continue
                    if !iter.set_next_oid(&var.oid) {
                        return Err(PyStopAsyncIteration::new_err("stop"));
                    }
                    // v1 may return Null at end of mib
                    return match &var.value {
                        SnmpValue::EndOfMibView | SnmpValue::Null => {
                            Err(PyStopAsyncIteration::new_err("stop"))
                        }
                        value => Ok((var.oid.try_to_python(py)?, value.try_to_python(py)?)),
                    };
                }
                // Multiple response, surely an error
                _ => return Err(SnmpError::InvalidPdu.into()),
            }
        }
        Err(SnmpError::InvalidPdu.into())
    }
}

impl SupportsGetBulk for SnmpV2cClientSocket {
    fn request<'a>(&'a self, iter: &GetBulkIter, request_id: i64) -> SnmpResult<Self::Message<'a>> {
        Ok(Self::Message {
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetBulkRequest(SnmpGetBulk {
                request_id,
                non_repeaters: 0,
                max_repetitions: iter.get_max_repetitions(),
                vars: vec![iter.get_next_oid()],
            }),
        })
    }
    fn parse(py: Python, msg: &Self::Message<'_>, iter: &mut GetBulkIter) -> PyResult<PyObject> {
        if let Some(resp) = msg.as_pdu().as_getresponse() {
            // Check varbinds size
            if resp.vars.is_empty() {
                return Err(PyStopAsyncIteration::new_err("stop"));
            }
            let list = PyList::empty_bound(py);
            for var in resp.vars.iter() {
                match &var.value {
                    SnmpValue::Null
                    | SnmpValue::NoSuchObject
                    | SnmpValue::NoSuchInstance
                    | SnmpValue::EndOfMibView => continue,
                    _ => {
                        // Check if we can continue
                        if !iter.set_next_oid(&var.oid) {
                            break;
                        }
                        // Append to list
                        list.append(PyTuple::new_bound(
                            py,
                            vec![var.oid.try_to_python(py)?, var.value.try_to_python(py)?],
                        ))
                        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
                    }
                }
            }
            if list.is_empty() {
                return Err(PyStopAsyncIteration::new_err("stop"));
            }
            return Ok(list.into());
        }
        Err(SnmpError::InvalidPdu.into())
    }
}
