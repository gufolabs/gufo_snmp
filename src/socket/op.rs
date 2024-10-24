// ------------------------------------------------------------------------
// Gufo SNMP: Socket operations
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{transport::SnmpTransport, GetNextIter};
use crate::{
    ber::BerEncoder,
    error::{SnmpError, SnmpResult},
    reqid::RequestId,
    snmp::msg::SnmpMessage,
};
use pyo3::prelude::*;

pub(crate) trait SnmpSocket
where
    Self: Send + Sync,
{
    type Message<'a>: TryFrom<&'a [u8], Error = SnmpError> + BerEncoder + SnmpMessage + Send;

    fn get_transport(&self) -> &SnmpTransport;

    fn get_transport_mut(&self) -> *mut SnmpTransport {
        self.get_transport() as *const SnmpTransport as *mut SnmpTransport
    }
    fn get_request_id(&mut self) -> &mut RequestId;

    fn send(&self, msg: Self::Message<'_>) -> SnmpResult<()> {
        unsafe {
            let io = self.get_transport_mut();
            (*io).send(msg)
        }
    }
    fn try_recv<'a>(&self) -> SnmpResult<Option<Self::Message<'a>>> {
        unsafe {
            let io = self.get_transport_mut();
            let reply = (*io).recv::<Self::Message<'a>>()?;
            if self.authenticate(&reply) {
                Ok(Some(reply))
            } else {
                Ok(None)
            }
        }
    }
    fn authenticate(&self, msg: &Self::Message<'_>) -> bool;
}

pub(crate) trait SupportsGet: SnmpSocket {
    fn request<'a>(&'a self, oid: &str, request_id: i64) -> SnmpResult<Self::Message<'a>>;
    fn parse(py: Python, msg: &Self::Message<'_>) -> PyResult<Option<PyObject>>;
    // Send get request and receive and decode reply
    fn get(&mut self, py: Python, oid: &str) -> PyResult<Option<PyObject>> {
        // Release GIL
        let reply = py.allow_threads(|| -> SnmpResult<Self::Message<'_>> {
            // Send request
            let req_id = self.get_request_id().get_next();
            self.send(self.request(oid, req_id)?)?;
            // Check until our reply is received
            loop {
                if let Some(reply) = self.try_recv()? {
                    if let Some(r) = reply.as_pdu().as_getresponse() {
                        if self.get_request_id().check(r.request_id) {
                            return Ok(reply);
                        }
                    }
                }
            }
        })?;
        // Convert to python structure under GIL
        Self::parse(py, &reply)
    }
    // Send get request (for async)
    fn send_get(&mut self, py: Python, oid: &str) -> SnmpResult<()> {
        py.allow_threads(|| {
            let req_id = self.get_request_id().get_next();
            self.send(self.request(oid, req_id)?)
        })
    }
    // Receiver and parse getresponse (for async)
    fn recv_get(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        // Release GIL
        let reply = py.allow_threads(|| -> SnmpResult<Self::Message<'_>> {
            // Check until our reply is received
            loop {
                if let Some(reply) = self.try_recv()? {
                    if let Some(r) = reply.as_pdu().as_getresponse() {
                        if self.get_request_id().check(r.request_id) {
                            return Ok(reply);
                        }
                    }
                }
            }
        })?;
        // Convert to python structure under GIL
        Self::parse(py, &reply)
    }
}

pub(crate) trait SupportsGetMany: SnmpSocket {
    fn request<'a>(&'a self, oids: Vec<&str>, request_id: i64) -> SnmpResult<Self::Message<'a>>;
    fn parse(py: Python, msg: &Self::Message<'_>) -> PyResult<PyObject>;
    // Send get request and receive and decode reply
    fn get_many(&mut self, py: Python, oids: Vec<&str>) -> PyResult<PyObject> {
        // Release GIL
        let reply = py.allow_threads(|| -> SnmpResult<Self::Message<'_>> {
            // Send request
            let req_id = self.get_request_id().get_next();
            self.send(self.request(oids, req_id)?)?;
            // Check until our reply is received
            loop {
                if let Some(reply) = self.try_recv()? {
                    if let Some(r) = reply.as_pdu().as_getresponse() {
                        if self.get_request_id().check(r.request_id) {
                            return Ok(reply);
                        }
                    }
                }
            }
        })?;
        // Convert to python structure under GIL
        Self::parse(py, &reply)
    }
    // Send get request (for async)
    fn send_get_many(&mut self, py: Python, oids: Vec<&str>) -> SnmpResult<()> {
        py.allow_threads(|| {
            let req_id = self.get_request_id().get_next();
            self.send(self.request(oids, req_id)?)
        })
    }
    // Receiver and parse getresponse (for async)
    fn recv_get_many(&mut self, py: Python) -> PyResult<PyObject> {
        // Release GIL
        let reply = py.allow_threads(|| -> SnmpResult<Self::Message<'_>> {
            // Check until our reply is received
            loop {
                if let Some(reply) = self.try_recv()? {
                    if let Some(r) = reply.as_pdu().as_getresponse() {
                        if self.get_request_id().check(r.request_id) {
                            return Ok(reply);
                        }
                    }
                }
            }
        })?;
        // Convert to python structure under GIL
        Self::parse(py, &reply)
    }
}

pub(crate) trait SupportsGetNext: SnmpSocket {
    fn request<'a>(&'a self, iter: &GetNextIter, request_id: i64) -> SnmpResult<Self::Message<'a>>;
    fn parse(
        py: Python,
        msg: &Self::Message<'_>,
        iter: &mut GetNextIter,
    ) -> PyResult<(PyObject, PyObject)>;
    // Send get request and receive and decode reply
    fn get_next(&mut self, py: Python, iter: &mut GetNextIter) -> PyResult<(PyObject, PyObject)> {
        // Release GIL
        let reply = py.allow_threads(|| -> SnmpResult<Self::Message<'_>> {
            // Send request
            let req_id = self.get_request_id().get_next();
            self.send(self.request(iter, req_id)?)?;
            // Check until our reply is received
            loop {
                if let Some(reply) = self.try_recv()? {
                    if let Some(r) = reply.as_pdu().as_getresponse() {
                        if self.get_request_id().check(r.request_id) {
                            return Ok(reply);
                        }
                    }
                }
            }
        })?;
        // Convert to python structure under GIL
        Self::parse(py, &reply, iter)
    }
    // Send get request (for async)
    fn send_get_next(&mut self, py: Python, iter: &GetNextIter) -> SnmpResult<()> {
        py.allow_threads(|| {
            let req_id = self.get_request_id().get_next();
            self.send(self.request(iter, req_id)?)
        })
    }
    // Receiver and parse getresponse (for async)
    fn recv_get_next(
        &mut self,
        py: Python,
        iter: &mut GetNextIter,
    ) -> PyResult<(PyObject, PyObject)> {
        // Release GIL
        let reply = py.allow_threads(|| -> SnmpResult<Self::Message<'_>> {
            // Check until our reply is received
            loop {
                if let Some(reply) = self.try_recv()? {
                    if let Some(r) = reply.as_pdu().as_getresponse() {
                        if self.get_request_id().check(r.request_id) {
                            return Ok(reply);
                        }
                    }
                }
            }
        })?;
        // Convert to python structure under GIL
        Self::parse(py, &reply, iter)
    }
}
