// ------------------------------------------------------------------------
// Gufo SNMP: SnmpV1ClientSocket
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::snmpsocket::SnmpSocket;
use crate::{
    ber::BerEncoder,
    buf::Buffer,
    error::SnmpResult,
    reqid::RequestId,
    snmp::{
        msg::SnmpV1Message,
        op::{GetIter, OpGet, OpGetBulk, OpGetMany, OpGetNext},
        pdu::SnmpPdu,
    },
};
use pyo3::{prelude::*, pybacked::PyBackedStr};
use socket2::Socket;
use std::os::fd::AsRawFd;

/// Python class wrapping socket implementation
#[pyclass]
pub struct SnmpV1ClientSocket {
    io: Socket,
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
        Ok(Self {
            io: Self::get_socket(addr, tos, send_buffer_size, recv_buffer_size, timeout_ns)?,
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
    fn get(&mut self, py: Python, oid: PyBackedStr) -> PyResult<Py<PyAny>> {
        Self::send_and_recv::<OpGet, _>(self, oid, None, py)
    }
    // Prepare and send GET request with single oid
    fn send_get(&mut self, py: Python, oid: PyBackedStr) -> PyResult<()> {
        Self::send_request::<OpGet, _>(self, oid, py)
    }
    // Try to receive GETRESPONSE
    fn recv_get(&mut self, py: Python) -> PyResult<Py<PyAny>> {
        Self::recv_reply::<OpGet, _>(self, None, py)
    }
    // .get_many()
    // Prepare and send GET request with multiple oids and receive reply
    fn get_many(&mut self, py: Python, oids: Vec<PyBackedStr>) -> PyResult<Py<PyAny>> {
        Self::send_and_recv::<OpGetMany, _>(self, oids, None, py)
    }
    // Prepare and send GET request with multiple oids
    fn send_get_many(&mut self, py: Python, oids: Vec<PyBackedStr>) -> PyResult<()> {
        Self::send_request::<OpGetMany, _>(self, oids, py)
    }
    fn recv_get_many(&mut self, py: Python) -> PyResult<Py<PyAny>> {
        Self::recv_reply::<OpGetMany, _>(self, None, py)
    }
    // .get_next()
    fn get_next(&mut self, py: Python, iter: &mut GetIter) -> PyResult<Py<PyAny>> {
        let oid = iter.get_next_oid();
        Self::send_and_recv::<OpGetNext, _>(self, oid, Some(iter), py)
    }
    fn send_get_next(&mut self, py: Python, iter: &GetIter) -> PyResult<()> {
        let oid = iter.get_next_oid();
        Self::send_request::<OpGetNext, _>(self, oid, py)
    }
    fn recv_get_next(&mut self, py: Python, iter: &mut GetIter) -> PyResult<Py<PyAny>> {
        Self::recv_reply::<OpGetNext, _>(self, Some(iter), py)
    }
    // .get_bulk()
    fn get_bulk(&mut self, py: Python, iter: &mut GetIter) -> PyResult<Py<PyAny>> {
        Self::send_and_recv::<OpGetBulk, _>(
            self,
            (iter.get_next_oid(), iter.get_max_repetitions()),
            Some(iter),
            py,
        )
    }
    // Send GetBulk request according to iter
    fn send_get_bulk(&mut self, py: Python, iter: &GetIter) -> PyResult<()> {
        Self::send_request::<OpGetBulk, _>(
            self,
            (iter.get_next_oid(), iter.get_max_repetitions()),
            py,
        )
    }
    // Try to receive GETRESPONSE for GETBULK
    fn recv_get_bulk(&mut self, iter: &mut GetIter, py: Python) -> PyResult<Py<PyAny>> {
        Self::recv_reply::<OpGetBulk, _>(self, Some(iter), py)
    }
}

impl SnmpSocket for SnmpV1ClientSocket {
    type Message<'a> = SnmpV1Message<'a>;

    fn get_io(&mut self) -> &mut Socket {
        &mut self.io
    }

    fn get_request_id(&mut self) -> &mut RequestId {
        &mut self.request_id
    }

    fn push_pdu(&mut self, pdu: SnmpPdu, buf: &mut Buffer) -> SnmpResult<()> {
        let msg = Self::Message {
            community: self.community.as_ref(),
            pdu,
        };
        msg.push_ber(buf)
    }

    fn unwrap_pdu<'a>(&'a mut self, msg: Self::Message<'a>) -> Option<SnmpPdu<'a>> {
        // Check communnity
        if msg.community != self.community.as_bytes() {
            return None;
        }
        // Check request id
        let pdu = msg.pdu;
        if !pdu.check(&self.request_id) {
            return None;
        }
        Some(pdu)
    }
}
