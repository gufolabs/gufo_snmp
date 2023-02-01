// ------------------------------------------------------------------------
// Gufo SNMP: SnmpClientSocket
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::{BerEncoder, SnmpOid, ToPython};
use crate::buf::Buffer;
use crate::error::SnmpError;
use crate::snmp::get::SnmpGet;
use crate::snmp::getbulk::SnmpGetBulk;
use crate::snmp::msg::SnmpMessage;
use crate::snmp::pdu::SnmpPdu;
use crate::snmp::var::SnmpValue;
use crate::snmp::SnmpVersion;
use pyo3::exceptions::PyRuntimeError;
use pyo3::{
    exceptions::PyBlockingIOError,
    exceptions::{PyOSError, PyStopAsyncIteration, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};
use rand::Rng;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::SocketAddr;
use std::os::fd::AsRawFd;

/// Python class wrapping socket implementation
#[pyclass]
pub(crate) struct SnmpClientSocket {
    io: Socket,
    addr: SockAddr,
    community: String,
    version: SnmpVersion,
    request_id: i64,
    buf: Buffer,
}

#[pyclass]
pub(crate) struct GetNextIter {
    start_oid: SnmpOid,
    next_oid: SnmpOid,
}

#[pyclass]
pub(crate) struct GetBulkIter {
    start_oid: SnmpOid,
    next_oid: SnmpOid,
    max_repetitions: i64,
}

#[pymethods]
impl SnmpClientSocket {
    /// Python constructor
    #[new]
    fn new(
        addr: String,
        community: String,
        version: u8,
        tos: u32,
        send_buffer_size: usize,
        recv_buffer_size: usize,
    ) -> PyResult<Self> {
        // Check version
        let version = version
            .try_into()
            .map_err(|_| PyValueError::new_err("invalid version"))?;
        // Parse address
        let sock_addr = addr
            .parse()
            .map_err(|_| PyOSError::new_err("invalid address"))?;
        // Detect the socket domain
        let domain = match sock_addr {
            SocketAddr::V4(_) => Domain::IPV4,
            SocketAddr::V6(_) => Domain::IPV6,
        };
        // Create internal socket
        let io = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        // Mark socket as non-blocking
        io.set_nonblocking(true)
            .map_err(|e| PyOSError::new_err(e.to_string()))?;
        // Set ToS
        if tos > 0 {
            io.set_tos(tos)
                .map_err(|e| PyOSError::new_err(e.to_string()))?;
        }
        // Set buffers
        if send_buffer_size > 0 {
            Self::set_send_buffer_size(&io, send_buffer_size)?;
        }
        if recv_buffer_size > 0 {
            Self::set_recv_buffer_size(&io, recv_buffer_size)?;
        }
        //
        Ok(Self {
            io,
            addr: sock_addr.into(),
            community,
            version,
            request_id: 0,
            buf: Buffer::default(),
        })
    }
    /// Get socket's file descriptor
    fn get_fd(&self) -> PyResult<i32> {
        Ok(self.io.as_raw_fd())
    }
    // Prepare and send GET request with single oid
    fn send_get(&mut self, oid: &str) -> PyResult<()> {
        // Encode oid
        let b_oid = SnmpOid::try_from(oid).map_err(|_| PyValueError::new_err("invalid oid"))?;
        // Send
        self._send_get(vec![b_oid])
    }
    // Prepare and send GET request with multiple oids
    fn send_get_many(&mut self, oids: Vec<&str>) -> PyResult<()> {
        // Encode oids
        let vars = oids
            .into_iter()
            .map(SnmpOid::try_from)
            .collect::<Result<Vec<SnmpOid>, SnmpError>>()
            .map_err(|_| PyValueError::new_err("invalid oid"))?;
        // Send
        self._send_get(vars)
    }
    // Send GetNext request according to iter
    fn send_getnext(&mut self, iter: &GetNextIter) -> PyResult<()> {
        // Start from clear buffer
        self.buf.reset();
        // Get new request id
        let request_id = self.new_request_id();
        // Encode message
        let msg = SnmpMessage {
            version: self.version.clone(),
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetNextRequest(SnmpGet {
                request_id,
                vars: vec![iter.get_next_oid()],
            }),
        };
        msg.push_ber(&mut self.buf)
            .map_err(|_| PyValueError::new_err("failed to encode message"))?;
        // Send
        self.io
            .send_to(self.buf.data(), &self.addr)
            .map_err(|_| PyOSError::new_err("failed to send"))?;
        Ok(())
    }
    // Send GetBulk request according to iter
    fn send_getbulk(&mut self, iter: &GetBulkIter) -> PyResult<()> {
        // Start from clear buffer
        self.buf.reset();
        // Get new request id
        let request_id = self.new_request_id();
        // Encode message
        let msg = SnmpMessage {
            version: self.version.clone(),
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetBulkRequest(SnmpGetBulk {
                request_id,
                non_repeaters: 0,
                max_repetitions: iter.get_max_repetitions(),
                vars: vec![iter.get_next_oid()],
            }),
        };
        msg.push_ber(&mut self.buf)
            .map_err(|_| PyValueError::new_err("failed to encode message"))?;
        // Send
        self.io
            .send_to(self.buf.data(), &self.addr)
            .map_err(|_| PyOSError::new_err("failed to send"))?;
        Ok(())
    }
    // Try to receive GETRESPONSE
    fn recv_getresponse(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        loop {
            // Receive response
            let size = match self.io.recv_from(self.buf.as_mut()) {
                Ok((s, _)) => s,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Err(PyBlockingIOError::new_err("blocked"))
                }
                Err(e) => return Err(PyOSError::new_err(e.to_string())),
            };
            // Parse response
            let msg = SnmpMessage::try_from(self.buf.as_slice(size))?;
            // Check version match
            if msg.version != self.version {
                continue; // Mismatched version, not our response.
            }
            // Check community match
            if msg.community != self.community.as_bytes() {
                continue; // Community mismatch, not our response.
            }
            match msg.pdu {
                SnmpPdu::GetResponse(resp) => {
                    // Check request id
                    if resp.request_id != self.request_id {
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
            // Receive response
            let size = match self.io.recv_from(self.buf.as_mut()) {
                Ok((s, _)) => s,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Err(PyBlockingIOError::new_err("blocked"))
                }
                Err(e) => return Err(PyOSError::new_err(e.to_string())),
            };
            // Parse response
            let msg = SnmpMessage::try_from(self.buf.as_slice(size))?;
            // Check version match
            if msg.version != self.version {
                continue; // Mismatched version, not our response.
            }
            // Check community match
            if msg.community != self.community.as_bytes() {
                continue; // Community mismatch, not our response.
            }
            match msg.pdu {
                SnmpPdu::GetResponse(resp) => {
                    // Check request id
                    if resp.request_id != self.request_id {
                        continue; // Not our request
                    }
                    // Check error_index
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
            // Receive response
            let size = match self.io.recv_from(self.buf.as_mut()) {
                Ok((s, _)) => s,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Err(PyBlockingIOError::new_err("blocked"))
                }
                Err(e) => return Err(PyOSError::new_err(e.to_string())),
            };
            // Parse response
            let msg = SnmpMessage::try_from(self.buf.as_slice(size))?;
            // Check version match
            if msg.version != self.version {
                continue; // Mismatched version, not our response.
            }
            // Check community match
            if msg.community != self.community.as_bytes() {
                continue; // Community mismatch, not our response.
            }
            match msg.pdu {
                SnmpPdu::GetResponse(resp) => {
                    // Check request id
                    if resp.request_id != self.request_id {
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
                            let value = &var.value;
                            if let SnmpValue::EndOfMibView = value {
                                // End of MIB, stop iteration
                                return Err(PyStopAsyncIteration::new_err("stop"));
                            }
                            return Ok((var.oid.try_to_python(py)?, value.try_to_python(py)?));
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
            // Receive response
            let size = match self.io.recv_from(self.buf.as_mut()) {
                Ok((s, _)) => s,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Err(PyBlockingIOError::new_err("blocked"))
                }
                Err(e) => return Err(PyOSError::new_err(e.to_string())),
            };
            // Parse response
            let msg = SnmpMessage::try_from(self.buf.as_slice(size))?;
            // Check version match
            if msg.version != self.version {
                continue; // Mismatched version, not our response.
            }
            // Check community match
            if msg.community != self.community.as_bytes() {
                continue; // Community mismatch, not our response.
            }
            match msg.pdu {
                SnmpPdu::GetResponse(resp) => {
                    // Check request id
                    if resp.request_id != self.request_id {
                        continue; // Not our request
                    }
                    // Check error_index
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
                                if !iter.set_next_oid(&var.oid) {
                                    break;
                                }
                                // Append to list
                                list.append(PyTuple::new(
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
}

impl SnmpClientSocket {
    /// Set internal socket's send buffer size
    fn set_send_buffer_size(io: &Socket, size: usize) -> PyResult<()> {
        // @todo: get wmem_max limit on Linux
        let mut effective_size = size;
        while effective_size > 0 {
            if io.set_send_buffer_size(effective_size).is_ok() {
                return Ok(());
            }
            effective_size >>= 1;
        }
        Err(PyOSError::new_err("unable to set buffer size"))
    }

    /// Set internal socket's receive buffer size
    fn set_recv_buffer_size(io: &Socket, size: usize) -> PyResult<()> {
        let mut effective_size = size;
        while effective_size > 0 {
            if io.set_recv_buffer_size(effective_size).is_ok() {
                return Ok(());
            }
            effective_size >>= 1;
        }
        Err(PyOSError::new_err("unable to set buffer size"))
    }
    //
    fn new_request_id(&mut self) -> i64 {
        let mut rng = rand::thread_rng();
        let x: i64 = rng.gen();
        self.request_id = x & 0x7fffffff;
        self.request_id
    }
    /// Send GET request
    fn _send_get(&mut self, vars: Vec<SnmpOid>) -> PyResult<()> {
        // Start from clear buffer
        self.buf.reset();
        // Get new request id
        let request_id = self.new_request_id();
        // Encode message
        let msg = SnmpMessage {
            version: self.version.clone(),
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetRequest(SnmpGet { request_id, vars }),
        };
        msg.push_ber(&mut self.buf)
            .map_err(|_| PyValueError::new_err("failed to encode message"))?;
        // Send
        self.io
            .send_to(self.buf.data(), &self.addr)
            .map_err(|_| PyOSError::new_err("failed to send"))?;
        Ok(())
    }
}

#[pymethods]
impl GetNextIter {
    /// Python constructor
    #[new]
    fn new(oid: &str) -> PyResult<Self> {
        let b_oid = SnmpOid::try_from(oid).map_err(|_| PyValueError::new_err("invalid oid"))?;
        Ok(GetNextIter {
            start_oid: b_oid.clone(),
            next_oid: b_oid,
        })
    }
}

impl GetNextIter {
    pub(crate) fn get_next_oid(&self) -> SnmpOid {
        self.next_oid.clone()
    }
    // Save oid for next request.
    // Return true if next request may be send or return false otherwise
    pub(crate) fn set_next_oid(&mut self, oid: &SnmpOid) -> bool {
        if self.start_oid.contains(oid) {
            self.next_oid = oid.clone();
            true
        } else {
            false
        }
    }
}

#[pymethods]
impl GetBulkIter {
    /// Python constructor
    #[new]
    fn new(oid: &str, max_repetitions: i64) -> PyResult<Self> {
        let b_oid = SnmpOid::try_from(oid).map_err(|_| PyValueError::new_err("invalid oid"))?;
        Ok(GetBulkIter {
            start_oid: b_oid.clone(),
            next_oid: b_oid,
            max_repetitions,
        })
    }
}

impl GetBulkIter {
    pub(crate) fn get_next_oid(&self) -> SnmpOid {
        self.next_oid.clone()
    }
    // Save oid for next request.
    // Return true if next request may be send or return false otherwise
    pub(crate) fn set_next_oid(&mut self, oid: &SnmpOid) -> bool {
        if self.start_oid.contains(oid) {
            self.next_oid = oid.clone();
            true
        } else {
            false
        }
    }
    //
    pub(crate) fn get_max_repetitions(&self) -> i64 {
        self.max_repetitions
    }
}
