// ------------------------------------------------------------------------
// Gufo SNMP: SnmpV3ClientSocket
// ------------------------------------------------------------------------
// Copyright (C) 2023-25, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::snmpsocket::SnmpSocket;
use crate::{
    auth::{AuthKey, SnmpAuth},
    ber::BerEncoder,
    buf::Buffer,
    error::SnmpResult,
    privacy::{PrivKey, SnmpPriv},
    reqid::RequestId,
    snmp::{
        msg::v3::{MsgData, ScopedPdu, SnmpV3Message, UsmParameters},
        op::{GetIter, OpGet, OpGetBulk, OpGetMany, OpGetNext, OpRefresh},
        pdu::SnmpPdu,
    },
};
use pyo3::types::PyBytes;
use pyo3::{prelude::*, pybacked::PyBackedStr};
use socket2::Socket;
use std::os::fd::AsRawFd;

/// Python class wrapping socket implementation
#[pyclass]
pub struct SnmpV3ClientSocket {
    io: Socket,
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
            io: Self::get_socket(addr, tos, send_buffer_size, recv_buffer_size, timeout_ns)?,
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
        user_name: String,
        auth_alg: u8,
        auth_key: &[u8],
        priv_alg: u8,
        priv_key: &[u8],
    ) -> PyResult<()> {
        // Replace user
        self.user_name = user_name;
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
        Ok(PyBytes::new(py, &self.engine_id).into())
    }
    // .get()
    // Prepare send GET request with single oid and receive reply
    fn get(&mut self, py: Python, oid: PyBackedStr) -> PyResult<PyObject> {
        Self::send_and_recv::<OpGet, _>(self, oid, None, py)
    }
    // Prepare and send GET request with single oid
    fn send_get(&mut self, py: Python, oid: PyBackedStr) -> PyResult<()> {
        Self::send_request::<OpGet, _>(self, oid, py)
    }
    // Try to receive GETRESPONSE
    fn recv_get(&mut self, py: Python) -> PyResult<PyObject> {
        Self::recv_reply::<OpGet, _>(self, None, py)
    }
    // .get_many()
    // Prepare and send GET request with multiple oids and receive reply
    fn get_many(&mut self, py: Python, oids: Vec<PyBackedStr>) -> PyResult<PyObject> {
        Self::send_and_recv::<OpGetMany, _>(self, oids, None, py)
    }
    // Prepare and send GET request with multiple oids
    fn send_get_many(&mut self, py: Python, oids: Vec<PyBackedStr>) -> PyResult<()> {
        Self::send_request::<OpGetMany, _>(self, oids, py)
    }
    fn recv_get_many(&mut self, py: Python) -> PyResult<PyObject> {
        Self::recv_reply::<OpGetMany, _>(self, None, py)
    }
    // .get_next()
    fn get_next(&mut self, py: Python, iter: &mut GetIter) -> PyResult<PyObject> {
        let oid = iter.get_next_oid();
        Self::send_and_recv::<OpGetNext, _>(self, oid, Some(iter), py)
    }
    fn send_get_next(&mut self, py: Python, iter: &GetIter) -> PyResult<()> {
        let oid = iter.get_next_oid();
        Self::send_request::<OpGetNext, _>(self, oid, py)
    }
    fn recv_get_next(&mut self, py: Python, iter: &mut GetIter) -> PyResult<PyObject> {
        Self::recv_reply::<OpGetNext, _>(self, Some(iter), py)
    }
    // .get_bulk()
    fn get_bulk(&mut self, py: Python, iter: &mut GetIter) -> PyResult<PyObject> {
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
    fn recv_get_bulk(&mut self, iter: &mut GetIter, py: Python) -> PyResult<PyObject> {
        Self::recv_reply::<OpGetBulk, _>(self, Some(iter), py)
    }
    // Send GET+Report to adjust boots and time
    fn refresh(&mut self, py: Python) -> PyResult<PyObject> {
        Self::send_and_recv::<OpRefresh, _>(self, (), None, py)
    }
    //
    fn send_refresh(&mut self, py: Python) -> PyResult<()> {
        Self::send_request::<OpRefresh, _>(self, (), py)
    }
    //
    fn recv_refresh(&mut self, py: Python) -> PyResult<PyObject> {
        Self::recv_reply::<OpRefresh, _>(self, None, py)
    }
}

impl SnmpSocket for SnmpV3ClientSocket {
    type Message<'a> = SnmpV3Message<'a>;

    fn get_io(&mut self) -> &mut Socket {
        &mut self.io
    }

    fn get_request_id(&mut self) -> &mut RequestId {
        &mut self.request_id
    }

    fn push_pdu(&mut self, pdu: SnmpPdu, buf: &mut Buffer) -> SnmpResult<()> {
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
            flag_report: true, // Some crazy boxes answer incorrectly if not set
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
        msg.push_ber(buf)?;
        // Apply auth
        let offset = buf.get_bookmark();
        self.auth_key.sign(buf.data_mut(), offset)
    }

    fn unwrap_pdu<'a>(&'a mut self, msg: Self::Message<'a>) -> Option<SnmpPdu<'a>> {
        // Get and decode scoped pdu
        let data = match msg.data {
            MsgData::Plaintext(x) => x,
            MsgData::Encrypted(x) => match self.priv_key.decrypt(x, &msg.usm) {
                Ok(x) => x,
                Err(_) => return None, // Failed to decrypt
            },
        };
        // Global header check
        if !(self.user_name.as_bytes() == msg.usm.user_name
            && (self.engine_id.is_empty() || msg.usm.engine_id == self.engine_id)
            && self.msg_id.check(msg.msg_id)
            && data.pdu.check(&self.request_id))
        {
            return None;
        }
        // Update engine parameters
        self.engine_boots = msg.usm.engine_boots;
        self.engine_time = msg.usm.engine_time;
        if self.engine_id.is_empty() {
            // Auto-detect engine id
            self.engine_id.extend_from_slice(msg.usm.engine_id);
        }
        Some(data.pdu)
    }
}
