// ------------------------------------------------------------------------
// Gufo SNMP: SnmpTransport implementation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::BerEncoder;
use crate::buf::Buffer;
use crate::error::{SnmpError, SnmpResult};
use core::time;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::os::fd::{AsRawFd, RawFd};
use time::Duration;

pub struct SnmpTransport {
    io: Socket,
    buf: Buffer,
}

impl SnmpTransport {
    pub fn new(
        addr: String,
        tos: u32,
        send_buffer_size: usize,
        recv_buffer_size: usize,
        timeout_ns: u64,
    ) -> SnmpResult<Self> {
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
        //
        Ok(Self {
            io,
            buf: Buffer::default(),
        })
    }
    /// Get buffer as mutable slice
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.buf.data_mut()
    }
    /// Get buffer bookmark
    pub fn get_bookmark(&self) -> usize {
        self.buf.get_bookmark()
    }
    /// Serialize message to buffer
    pub fn push_ber<T>(&mut self, msg: T) -> SnmpResult<()>
    where
        T: BerEncoder,
    {
        self.buf.reset();
        msg.push_ber(&mut self.buf)?;
        Ok(())
    }
    /// Send content of the buffer
    pub fn send_buffer(&mut self) -> SnmpResult<()> {
        self.io
            .send(self.buf.data())
            .map_err(|e| SnmpError::SocketError(e.to_string()))?;
        Ok(())
    }
    /// Send message to socket
    pub fn send<T>(&mut self, msg: T) -> SnmpResult<()>
    where
        T: BerEncoder,
    {
        self.push_ber(msg)?;
        self.send_buffer()
    }
    /// Receive message from socket
    pub fn recv<'a, 'b, T>(&'b mut self) -> SnmpResult<T>
    where
        T: TryFrom<&'a [u8], Error = SnmpError>,
        'b: 'a,
    {
        self.buf.reset();
        let size = match self.io.recv(self.buf.as_mut()) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return Err(SnmpError::WouldBlock)
            }
            Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                return Err(SnmpError::ConnectionRefused)
            }
            Err(e) => return Err(SnmpError::SocketError(e.to_string())),
        };
        // Parse response
        T::try_from(self.buf.as_slice(size))
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
}

impl AsRawFd for SnmpTransport {
    fn as_raw_fd(&self) -> RawFd {
        self.io.as_raw_fd()
    }
}
