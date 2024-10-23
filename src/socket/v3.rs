// ------------------------------------------------------------------------
// Gufo SNMP: SnmpV3ClientSocket
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::iter::{GetBulkIter, GetNextIter};
use super::transport::SnmpTransport;
use crate::auth::{AuthKey, SnmpAuth};
use crate::ber::{SnmpOid, ToPython};
use crate::error::{SnmpError, SnmpResult};
use crate::privacy::{PrivKey, SnmpPriv};
use crate::reqid::RequestId;
use crate::snmp::get::SnmpGet;
use crate::snmp::getbulk::SnmpGetBulk;
use crate::snmp::msg::v3::{MsgData, ScopedPdu, SnmpV3Message, UsmParameters};
use crate::snmp::pdu::SnmpPdu;
use crate::snmp::value::SnmpValue;
use pyo3::exceptions::PyRuntimeError;
use pyo3::types::PyBytes;
use pyo3::{
    exceptions::{PyStopAsyncIteration, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};
use std::os::fd::AsRawFd;

/// Python class wrapping socket implementation
#[pyclass]
pub struct SnmpV3ClientSocket {
    io: SnmpTransport,
    engine_id: Vec<u8>,
    engine_boots: i64,
    engine_time: i64,
    user_name: String,
    auth_key: AuthKey,
    priv_key: PrivKey,
    msg_id: RequestId,
    request_id: RequestId,
}

const EMPTY: [u8; 0] = [];

#[pymethods]
impl SnmpV3ClientSocket {
    /// Python constructor
    #[allow(clippy::too_many_arguments)] // Internal interface
    #[new]
    fn new(
        addr: String,
        engine_id: Vec<u8>,
        user_name: String,
        auth_alg: u8,
        auth_key: &[u8],
        priv_alg: u8,
        priv_key: &[u8],
        tos: u32,
        send_buffer_size: usize,
        recv_buffer_size: usize,
        timeout_ns: u64,
    ) -> PyResult<Self> {
        // Transport
        let io = SnmpTransport::new(addr, tos, send_buffer_size, recv_buffer_size, timeout_ns)?;
        // Auth key
        let mut auth = AuthKey::new(auth_alg)?;
        auth.as_key_type(auth_alg, auth_key, &engine_id)?;
        // Priv key
        let mut pk = PrivKey::new(priv_alg)?;
        if pk.has_priv() {
            // Localize key
            let mut pk_auth = AuthKey::new(auth_alg)?;
            pk_auth.as_key_type(priv_alg, priv_key, &engine_id)?;
            pk.as_localized(pk_auth.get_key())?;
        }
        //
        Ok(Self {
            io,
            engine_id,
            engine_boots: 0,
            engine_time: 0,
            user_name,
            auth_key: auth,
            priv_key: pk,
            msg_id: RequestId::default(),
            request_id: RequestId::default(),
        })
    }
    /// Change keys
    fn set_keys(
        &mut self,
        auth_alg: u8,
        auth_key: &[u8],
        priv_alg: u8,
        priv_key: &[u8],
    ) -> PyResult<()> {
        // Auth key
        let mut auth = AuthKey::new(auth_alg)?;
        auth.as_key_type(auth_alg, auth_key, &self.engine_id)?;
        // Priv key
        let mut pk = PrivKey::new(priv_alg)?;
        if pk.has_priv() {
            // Localize key
            let mut pk_auth = AuthKey::new(auth_alg)?;
            pk_auth.as_key_type(priv_alg, priv_key, &self.engine_id)?;
            pk.as_localized(pk_auth.get_key())?;
        }
        self.auth_key = auth;
        self.priv_key = pk;
        Ok(())
    }
    /// Get socket's file descriptor
    fn get_fd(&self) -> PyResult<i32> {
        Ok(self.io.as_raw_fd())
    }
    /// Get engine id
    fn get_engine_id(&self, py: Python) -> PyResult<PyObject> {
        Ok(PyBytes::new_bound(py, &self.engine_id).into())
    }
    // .get()
    // Prepare send GET request with single oid and receive reply
    fn get(&mut self, py: Python, oid: &str) -> PyResult<Option<PyObject>> {
        self.send_get(oid)?;
        self.recv_get(py)
    }
    // Prepare and send GET request with single oid
    fn send_get(&mut self, oid: &str) -> PyResult<()> {
        let request_id = self.request_id.get_next();
        Ok(self.wrap_and_send(
            SnmpPdu::GetRequest(SnmpGet {
                request_id,
                vars: vec![
                    SnmpOid::try_from(oid).map_err(|_| PyValueError::new_err("invalid oid"))?
                ],
            }),
            false,
        )?)
    }
    // Try to receive GETRESPONSE
    fn recv_get(&mut self, py: Python) -> PyResult<Option<PyObject>> {
        loop {
            match self.recv_and_unwrap()? {
                Some(pdu) => match pdu {
                    SnmpPdu::GetResponse(resp) => {
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
                    SnmpPdu::Report(_) => return Err(SnmpError::AuthenticationFailed.into()),
                    _ => continue,
                },
                None => continue,
            }
        }
    }
    // .get_many()
    // Prepare and send GET request with multiple oids and receive reply
    fn get_many(&mut self, py: Python, oids: Vec<&str>) -> PyResult<PyObject> {
        self.send_get_many(oids)?;
        self.recv_get_many(py)
    }
    // Prepare and send GET request with multiple oids
    fn send_get_many(&mut self, oids: Vec<&str>) -> PyResult<()> {
        let request_id = self.request_id.get_next();
        Ok(self.wrap_and_send(
            SnmpPdu::GetRequest(SnmpGet {
                request_id,
                vars: oids
                    .into_iter()
                    .map(SnmpOid::try_from)
                    .collect::<SnmpResult<Vec<SnmpOid>>>()
                    .map_err(|_| PyValueError::new_err("invalid oid"))?,
            }),
            false,
        )?)
    }
    fn recv_get_many(&mut self, py: Python) -> PyResult<PyObject> {
        loop {
            match self.recv_and_unwrap()? {
                Some(pdu) => match pdu {
                    SnmpPdu::GetResponse(resp) => {
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
                                    .set_item(
                                        var.oid.try_to_python(py)?,
                                        var.value.try_to_python(py)?,
                                    )
                                    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
                            }
                        }
                        return Ok(dict.into());
                    }
                    SnmpPdu::Report(_) => return Err(SnmpError::AuthenticationFailed.into()),
                    _ => continue,
                },
                None => continue,
            }
        }
    }
    // Send GetNext request according to iter
    fn async_send_getnext(&mut self, iter: &GetNextIter) -> PyResult<()> {
        let request_id = self.request_id.get_next();
        Ok(self.wrap_and_send(
            SnmpPdu::GetNextRequest(SnmpGet {
                request_id,
                vars: vec![iter.get_next_oid()],
            }),
            false,
        )?)
    }
    // Send GetBulk request according to iter
    fn async_send_getbulk(&mut self, iter: &GetBulkIter) -> PyResult<()> {
        let request_id = self.request_id.get_next();
        Ok(self.wrap_and_send(
            SnmpPdu::GetBulkRequest(SnmpGetBulk {
                request_id,
                non_repeaters: 0,
                max_repetitions: iter.get_max_repetitions(),
                vars: vec![iter.get_next_oid()],
            }),
            false,
        )?)
    }
    // Send GET+Report to adjust boots and time
    fn async_send_refresh(&mut self) -> PyResult<()> {
        let request_id = self.request_id.get_next();
        Ok(self.wrap_and_send(
            SnmpPdu::GetRequest(SnmpGet {
                request_id,
                vars: vec![],
            }),
            true,
        )?)
    }
    // Try to receive GETRESPONSE for GETNEXT
    fn async_recv_getresponse_next(
        &mut self,
        iter: &mut GetNextIter,
        py: Python,
    ) -> PyResult<(PyObject, PyObject)> {
        loop {
            match self.recv_and_unwrap()? {
                Some(pdu) => match pdu {
                    SnmpPdu::GetResponse(resp) => {
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
                    SnmpPdu::Report(_) => return Err(SnmpError::AuthenticationFailed.into()),
                    _ => continue,
                },
                None => continue,
            }
        }
    }
    // Try to receive GETRESPONSE for GETBULK
    fn async_recv_getresponse_bulk(
        &mut self,
        iter: &mut GetBulkIter,
        py: Python,
    ) -> PyResult<PyObject> {
        loop {
            match self.recv_and_unwrap()? {
                Some(pdu) => match pdu {
                    SnmpPdu::GetResponse(resp) => {
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
                                        vec![
                                            var.oid.try_to_python(py)?,
                                            var.value.try_to_python(py)?,
                                        ],
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
                    SnmpPdu::Report(_) => return Err(SnmpError::AuthenticationFailed.into()),
                    _ => continue,
                },
                None => continue,
            }
        }
    }
    // Receive refresh report
    fn async_recv_refresh(&mut self) -> PyResult<()> {
        loop {
            match self.recv_and_unwrap()? {
                Some(_) => {
                    return Ok(());
                }
                None => continue,
            }
        }
    }
    //
    fn sync_getnext(
        &mut self,
        py: Python,
        iter: &mut GetNextIter,
    ) -> PyResult<(PyObject, PyObject)> {
        self.async_send_getnext(iter)?;
        self.async_recv_getresponse_next(iter, py)
    }
    //
    fn sync_getbulk(&mut self, py: Python, iter: &mut GetBulkIter) -> PyResult<PyObject> {
        self.async_send_getbulk(iter)?;
        self.async_recv_getresponse_bulk(iter, py)
    }
    // Send and receive refresh report
    fn sync_refresh(&mut self) -> PyResult<()> {
        self.async_send_refresh()?;
        self.async_recv_refresh()
    }
}
impl SnmpV3ClientSocket {
    // Wrap PDU and send
    fn wrap_and_send(&mut self, pdu: SnmpPdu, flag_report: bool) -> SnmpResult<()> {
        //
        let flag_priv = self.priv_key.has_priv();
        let scoped_pdu = ScopedPdu {
            engine_id: &self.engine_id,
            pdu,
        };
        let (privacy_params, data) = if flag_priv {
            // Encrypted
            let (enc_data, privacy_params) = self.priv_key.encrypt(
                &scoped_pdu,
                self.engine_boots as u32,
                self.engine_time as u32,
            )?;
            (privacy_params, MsgData::Encrypted(enc_data))
        } else {
            (EMPTY.as_ref(), MsgData::Plaintext(scoped_pdu))
        };
        // Prepare message
        let msg = SnmpV3Message {
            msg_id: self.msg_id.get_next(),
            flag_auth: self.auth_key.has_auth(),
            flag_priv,
            flag_report,
            usm: UsmParameters {
                engine_id: &self.engine_id,
                engine_boots: self.engine_boots,
                engine_time: self.engine_time,
                user_name: self.user_name.as_ref(),
                auth_params: self.auth_key.placeholder(),
                privacy_params,
            },
            data,
        };
        // Serialize BER to buffer
        self.io.push_ber(msg)?;
        // Apply auth
        let offset = self.io.get_bookmark();
        self.auth_key.sign(self.io.data_mut(), offset)?;
        // Send buffer
        self.io.send_buffer()
    }
    // Receive and unwrap PDU
    fn recv_and_unwrap(&mut self) -> SnmpResult<Option<SnmpPdu>> {
        let msg = self.io.recv::<SnmpV3Message>()?;
        let data = match msg.data {
            MsgData::Plaintext(x) => x,
            MsgData::Encrypted(x) => {
                // decode
                self.priv_key.decrypt(x, &msg.usm)?
            }
        };
        // Global header check
        if !(self.user_name.as_bytes() == msg.usm.user_name
            && (self.engine_id.is_empty() || msg.usm.engine_id == self.engine_id)
            && self.msg_id.check(msg.msg_id)
            && data.pdu.check(&self.request_id))
        {
            return Ok(None);
        }
        self.engine_boots = msg.usm.engine_boots;
        self.engine_time = msg.usm.engine_time;
        if self.engine_id.is_empty() {
            // Auto-detect engine id
            self.engine_id.extend_from_slice(msg.usm.engine_id);
        }
        Ok(Some(data.pdu))
    }
}
