// ------------------------------------------------------------------------
// Gufo SNMP: SnmpV1ClientSocket
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::iter::{GetBulkIter, GetNextIter};
use super::op::{SnmpSocket, SupportsGet, SupportsGetBulk, SupportsGetMany, SupportsGetNext};
use super::transport::SnmpTransport;
use crate::ber::SnmpOid;
use crate::error::{SnmpError, SnmpResult};
use crate::reqid::RequestId;
use crate::snmp::get::SnmpGet;
use crate::snmp::getbulk::SnmpGetBulk;
use crate::snmp::msg::SnmpV1Message;
use crate::snmp::pdu::SnmpPdu;
use pyo3::prelude::*;
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

impl SnmpSocket for SnmpV1ClientSocket {
    type Message<'a> = SnmpV1Message<'a>;

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

impl SupportsGet for SnmpV1ClientSocket {
    fn request<'a>(&'a self, oid: &str, request_id: i64) -> SnmpResult<Self::Message<'a>> {
        Ok(Self::Message {
            community: self.community.as_bytes(),
            pdu: SnmpPdu::GetRequest(SnmpGet {
                request_id,
                vars: vec![SnmpOid::try_from(oid)?],
            }),
        })
    }
}

impl SupportsGetMany for SnmpV1ClientSocket {
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
}

impl SupportsGetNext for SnmpV1ClientSocket {
    fn request<'a>(&'a self, iter: &GetNextIter, request_id: i64) -> SnmpResult<Self::Message<'a>> {
        Ok(Self::Message {
            community: self.community.as_ref(),
            pdu: SnmpPdu::GetNextRequest(SnmpGet {
                request_id,
                vars: vec![iter.get_next_oid()],
            }),
        })
    }
}

impl SupportsGetBulk for SnmpV1ClientSocket {
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
}
