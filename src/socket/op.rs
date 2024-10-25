// ------------------------------------------------------------------------
// Gufo SNMP: Socket operations
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::{
    iter::{GetBulkIter, GetNextIter},
    transport::SnmpTransport,
};
use crate::{
    ber::{BerEncoder, ToPython},
    error::{SnmpError, SnmpResult},
    reqid::RequestId,
    snmp::{msg::SnmpMessage, value::SnmpValue},
};
use pyo3::{
    exceptions::{PyRuntimeError, PyStopAsyncIteration},
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};

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

pub(crate) trait SupportsGetBulk: SnmpSocket {
    fn request<'a>(&'a self, iter: &GetBulkIter, request_id: i64) -> SnmpResult<Self::Message<'a>>;
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
    // Send get request and receive and decode reply
    fn get_bulk(&mut self, py: Python, iter: &mut GetBulkIter) -> PyResult<PyObject> {
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
    fn send_get_bulk(&mut self, py: Python, iter: &GetBulkIter) -> SnmpResult<()> {
        py.allow_threads(|| {
            let req_id = self.get_request_id().get_next();
            self.send(self.request(iter, req_id)?)
        })
    }
    // Receiver and parse getresponse (for async)
    fn recv_get_bulk(&mut self, py: Python, iter: &mut GetBulkIter) -> PyResult<PyObject> {
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
