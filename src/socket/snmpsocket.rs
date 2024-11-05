// ------------------------------------------------------------------------
// Gufo SNMP: Socket operations
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::snmp::op::{GetIter, PyOp};
use crate::{
    ber::BerEncoder,
    buf::{get_buffer_pool, Buffer},
    error::{SnmpError, SnmpResult},
    reqid::RequestId,
    snmp::pdu::SnmpPdu,
};
use pyo3::prelude::*;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::time::Duration;

pub(crate) trait SnmpSocket
where
    Self: Send + Sync,
{
    type Message<'a>: TryFrom<&'a [u8], Error = SnmpError> + BerEncoder + Send
    where
        Self: 'a;

    fn get_io(&mut self) -> &mut Socket;
    fn get_socket(
        addr: String,
        tos: u32,
        send_buffer_size: usize,
        recv_buffer_size: usize,
        timeout_ns: u64,
    ) -> SnmpResult<Socket> {
        // Parse address
        let sock_addr = addr
            .parse()
            .map_err(|_| SnmpError::SocketError("invalid address".into()))?;
        // Detect the socket domain
        let domain = match sock_addr {
            SocketAddr::V4(_) => Domain::IPV4,
            SocketAddr::V6(_) => Domain::IPV6,
        };
        // Create internal socket
        let io = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))
            .map_err(|e| SnmpError::SocketError(e.to_string()))?;
        if timeout_ns > 0 {
            // Blocking mode
            io.set_read_timeout(Some(Duration::from_nanos(timeout_ns)))
                .map_err(|e| SnmpError::SocketError(e.to_string()))?;
        } else {
            // Mark socket as non-blocking
            io.set_nonblocking(true)
                .map_err(|e| SnmpError::SocketError(e.to_string()))?;
        }
        // Set ToS
        if tos > 0 {
            io.set_tos(tos)
                .map_err(|e| SnmpError::SocketError(e.to_string()))?;
        }
        // Set buffers
        if send_buffer_size > 0 {
            Self::set_send_buffer_size(&io, send_buffer_size)?;
        }
        if recv_buffer_size > 0 {
            Self::set_recv_buffer_size(&io, recv_buffer_size)?;
        }
        // Make socket connected
        io.connect(&sock_addr.into())
            .map_err(|e| SnmpError::SocketError(e.to_string()))?;
        Ok(io)
    }
    /// Set internal socket's send buffer size
    fn set_send_buffer_size(io: &Socket, size: usize) -> SnmpResult<()> {
        // @todo: get wmem_max limit on Linux
        let mut effective_size = size;
        while effective_size > 0 {
            if io.set_send_buffer_size(effective_size).is_ok() {
                return Ok(());
            }
            effective_size >>= 1;
        }
        Err(SnmpError::SocketError("unable to set buffer size".into()))
    }
    /// Set internal socket's receive buffer size
    fn set_recv_buffer_size(io: &Socket, size: usize) -> SnmpResult<()> {
        let mut effective_size = size;
        while effective_size > 0 {
            if io.set_recv_buffer_size(effective_size).is_ok() {
                return Ok(());
            }
            effective_size >>= 1;
        }
        Err(SnmpError::SocketError("unable to set buffer size".into()))
    }
    fn get_request_id(&mut self) -> &mut RequestId;
    fn push_pdu(&mut self, pdu: SnmpPdu, buf: &mut Buffer) -> SnmpResult<()>;
    fn unwrap_pdu<'a>(&'a mut self, msg: Self::Message<'a>) -> Option<SnmpPdu<'a>>;
    //
    fn recv_socket<'a>(io: &mut Socket, buf: &'a mut Buffer) -> SnmpResult<&'a [u8]> {
        match io.recv(buf.as_mut()) {
            Ok(s) => Ok(buf.as_slice(s)),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(SnmpError::WouldBlock),
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                Err(SnmpError::ConnectionRefused)
            }
            Err(e) => Err(SnmpError::SocketError(e.to_string())),
        }
    }
    // Send section with released GIL
    fn _send_inner(&mut self, pdu: SnmpPdu) -> PyResult<()> {
        // Get buffer for pool
        let mut pool = get_buffer_pool().acquire();
        let buf = pool.as_mut();
        self.push_pdu(pdu, buf)?;
        // Send message
        self.get_io()
            .send(buf.data())
            .map_err(|e| SnmpError::SocketError(e.to_string()))?;
        Ok(())
    }

    fn _recv_inner<'a, T, V>(&mut self, iter: Option<&mut GetIter>) -> PyResult<PyObject>
    where
        T: PyOp<'a, V>,
        V: 'a,
    {
        // Get buffer from pool
        let mut h = get_buffer_pool().acquire();
        let buf = h.as_mut();
        // We can catch unwanted replies, so do it in a loop
        loop {
            // Nested scope to release io early after receiving message
            let msg = {
                let io = self.get_io();
                let data = Self::recv_socket(io, buf)?;
                // Decode message
                Self::Message::try_from(data)?
            };
            match self.unwrap_pdu(msg) {
                Some(ref pdu) => return Python::with_gil(|py| T::to_python(pdu, iter, py)),
                None => {
                    buf.reset();
                    continue;
                }
            }
        }
    }

    fn send_request<'a, T, V>(&mut self, req: V, py: Python) -> PyResult<()>
    where
        T: PyOp<'a, V>,
        V: 'a,
    {
        // Parse python arguments, unnder GIL
        let request_id = self.get_request_id().get_next();
        let pdu = T::from_python(req, request_id)?;
        // Release GIL
        py.allow_threads(|| self._send_inner(pdu))
    }

    fn recv_reply<'a, T, V>(&mut self, iter: Option<&mut GetIter>, py: Python) -> PyResult<PyObject>
    where
        T: PyOp<'a, V>,
        V: 'a,
    {
        py.allow_threads(|| self._recv_inner::<T, V>(iter))
    }

    fn send_and_recv<'a, T, V>(
        &mut self,
        req: V,
        iter: Option<&mut GetIter>,
        py: Python,
    ) -> PyResult<PyObject>
    where
        T: PyOp<'a, V>,
        V: 'a,
    {
        let request_id = self.get_request_id().get_next();
        let pdu = T::from_python(req, request_id)?;
        py.allow_threads(|| {
            self._send_inner(pdu)?;
            self._recv_inner::<T, V>(iter)
        })
    }
}
