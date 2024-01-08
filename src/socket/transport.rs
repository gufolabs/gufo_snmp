// ------------------------------------------------------------------------
// Gufo SNMP: SnmpTransport implementation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::ber::BerEncoder;
use crate::buf::Buffer;
use crate::error::SnmpError;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::SocketAddr;
use std::os::fd::{AsRawFd, RawFd};

pub struct SnmpTransport {
    io: Socket,
    addr: SockAddr,
    buf: Buffer,
}

impl SnmpTransport {
    pub fn new(
        addr: String,
        tos: u32,
        send_buffer_size: usize,
        recv_buffer_size: usize,
    ) -> Result<Self, SnmpError> {
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
        // Mark socket as non-blocking
        io.set_nonblocking(true)
            .map_err(|e| SnmpError::SocketError(e.to_string()))?;
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
        //
        Ok(Self {
            io,
            addr: sock_addr.into(),
            buf: Buffer::default(),
        })
    }
    /// Send message to socket
    pub fn send<T>(&mut self, msg: T) -> Result<(), SnmpError>
    where
        T: BerEncoder,
    {
        self.buf.reset();
        msg.push_ber(&mut self.buf)?;
        // Send
        self.io
            .send_to(self.buf.data(), &self.addr)
            .map_err(|e| SnmpError::SocketError(e.to_string()))?;
        Ok(())
    }
    /// Receive message from socket
    pub fn recv<'a, 'b, T>(&'b mut self) -> Result<T, SnmpError>
    where
        T: TryFrom<&'a [u8], Error = SnmpError>,
        'b: 'a,
    {
        let size = match self.io.recv_from(self.buf.as_mut()) {
            Ok((s, _)) => s,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return Err(SnmpError::WouldBlock)
            }
            Err(e) => return Err(SnmpError::SocketError(e.to_string())),
        };
        // Parse response
        T::try_from(self.buf.as_slice(size))
    }
    /// Set internal socket's send buffer size
    fn set_send_buffer_size(io: &Socket, size: usize) -> Result<(), SnmpError> {
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
    fn set_recv_buffer_size(io: &Socket, size: usize) -> Result<(), SnmpError> {
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
