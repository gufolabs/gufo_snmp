// ------------------------------------------------------------------------
// Gufo SNMP: SnmpV1ClientSocket
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::iter::{GetBulkIter, GetNextIter};
use super::transport::SnmpTransport;
use crate::ber::{SnmpOid, ToPython};
use crate::error::SnmpError;
use crate::reqid::RequestId;
use crate::snmp::get::SnmpGet;
use crate::snmp::getbulk::SnmpGetBulk;
use crate::snmp::msg::SnmpV1Message;
use crate::snmp::pdu::SnmpPdu;
use crate::snmp::value::SnmpValue;
use pyo3::exceptions::PyRuntimeError;
use pyo3::{
    exceptions::{PyStopAsyncIteration, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};
use std::os::fd::AsRawFd;

/// Python class wrapping socket implementation
#[pyclass]
pub struct SnmpV1ClientSocket {
    io: SnmpTransport,
    community: String,
    request_id: RequestId,
}

#[pymethods]
impl SnmpV1ClientSocket {
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
    // Prepare and send GET request with single oid
    fn send_get(&mut self, oid: &str) -> PyResult<()> {
        Ok(self.io.send(SnmpV1Message {
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetRequest(SnmpGet {
                request_id: self.request_id.get_next(),
                vars: vec![
                    SnmpOid::try_from(oid).map_err(|_| PyValueError::new_err("invalid oid"))?
                ],
            }),
        })?)
    }
    // Prepare and send GET request with multiple oids
    fn send_get_many(&mut self, oids: Vec<&str>) -> PyResult<()> {
        Ok(self.io.send(SnmpV1Message {
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetRequest(SnmpGet {
                request_id: self.request_id.get_next(),
                vars: oids
                    .into_iter()
                    .map(SnmpOid::try_from)
                    .collect::<Result<Vec<SnmpOid>, SnmpError>>()
                    .map_err(|_| PyValueError::new_err("invalid oid"))?,
            }),
        })?)
    }
    // Send GetNext request according to iter
    fn send_getnext(&mut self, iter: &GetNextIter) -> PyResult<()> {
        Ok(self.io.send(SnmpV1Message {
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetNextRequest(SnmpGet {
                request_id: self.request_id.get_next(),
                vars: vec![iter.get_next_oid()],
            }),
        })?)
    }
    // Send GetBulk request according to iter
    fn send_getbulk(&mut self, iter: &GetBulkIter) -> PyResult<()> {
        // Encode message
        Ok(self.io.send(SnmpV1Message {
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetBulkRequest(SnmpGetBulk {
                request_id: self.request_id.get_next(),
                non_repeaters: 0,
                max_repetitions: iter.get_max_repetitions(),
                vars: vec![iter.get_next_oid()],
            }),
        })?)
    }
    // Try to receive GETRESPONSE
    fn recv_getresponse(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        loop {
            // Receive and decode message
            let msg = self.io.recv::<SnmpV1Message>()?;
            // Check community match
            if msg.community != self.community.as_bytes() {
                continue; // Community mismatch, not our response.
            }
            match msg.pdu {
                SnmpPdu::GetResponse(resp) => {
                    // Check request id
                    if !self.request_id.check(resp.request_id) {
                        continue; // Not our request
                    }
                    // Check error_index
                    // Check varbinds size
                    match resp.vars.len() {
                        // Empty response, return None
                        0 => return Ok(None),
                        // Return value
                        1 => {
                            let var = &resp.vars[0];
                            let value = &var.value;
                            match value {
                                SnmpValue::NoSuchObject
                                | SnmpValue::NoSuchInstance
                                | SnmpValue::EndOfMibView => {
                                    return Err(SnmpError::NoSuchInstance.into())
                                }
                                SnmpValue::Null => return Ok(None),
                                _ => return Ok(Some(value.try_to_python(py)?)),
                            }
                        }
                        // Multiple response, surely an error
                        _ => return Err(SnmpError::InvalidPdu.into()),
                    }
                }
                _ => continue,
            }
        }
    }
    fn recv_getresponse_many(&mut self, py: Python) -> PyResult<PyObject> {
        loop {
            // Receive and decode message
            let msg = self.io.recv::<SnmpV1Message>()?;
            // Check community match
            if msg.community != self.community.as_bytes() {
                continue; // Community mismatch, not our response.
            }
            match msg.pdu {
                SnmpPdu::GetResponse(resp) => {
                    // Check request id
                    if !self.request_id.check(resp.request_id) {
                        continue; // Not our request
                    }
                    // Check error_index
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
                _ => continue,
            }
        }
    }
    // Try to receive GETRESPONSE for GETNEXT
    fn recv_getresponse_next(
        &mut self,
        iter: &mut GetNextIter,
        py: Python,
    ) -> PyResult<(PyObject, PyObject)> {
        loop {
            // Receive and decode message
            let msg = self.io.recv::<SnmpV1Message>()?;
            // Check community match
            if msg.community != self.community.as_bytes() {
                continue; // Community mismatch, not our response.
            }
            match msg.pdu {
                SnmpPdu::GetResponse(resp) => {
                    // Check request id
                    if !self.request_id.check(resp.request_id) {
                        continue; // Not our request
                    }
                    // Check error_index
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
                _ => continue,
            }
        }
    }
    // Try to receive GETRESPONSE for GETBULK
    fn recv_getresponse_bulk(&mut self, iter: &mut GetBulkIter, py: Python) -> PyResult<PyObject> {
        loop {
            // Receive and decode message
            let msg = self.io.recv::<SnmpV1Message>()?;
            // Check community match
            if msg.community != self.community.as_bytes() {
                continue; // Community mismatch, not our response.
            }
            match msg.pdu {
                SnmpPdu::GetResponse(resp) => {
                    // Check request id
                    if !self.request_id.check(resp.request_id) {
                        continue; // Not our request
                    }
                    // Check error_index
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
                _ => continue,
            }
        }
    }
    // Prepare send GET request with single oid and receive reply
    fn sync_get(&mut self, py: Python, oid: &str) -> PyResult<Option<PyObject>> {
        self.send_get(oid)?;
        self.recv_getresponse(py)
    }
    // Prepare and send GET request with multiple oids and receive reply
    fn sync_get_many(&mut self, py: Python, oids: Vec<&str>) -> PyResult<PyObject> {
        self.send_get_many(oids)?;
        self.recv_getresponse_many(py)
    }
    //
    fn sync_getnext(
        &mut self,
        py: Python,
        iter: &mut GetNextIter,
    ) -> PyResult<(PyObject, PyObject)> {
        self.send_getnext(iter)?;
        self.recv_getresponse_next(iter, py)
    }
    //
    fn sync_getbulk(&mut self, py: Python, iter: &mut GetBulkIter) -> PyResult<PyObject> {
        self.send_getbulk(iter)?;
        self.recv_getresponse_bulk(iter, py)
    }
}
