// ------------------------------------------------------------------------
// Gufo SNMP: Buffer implementation
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::error::SnmpError;
use std::mem::MaybeUninit;

const MAX_SIZE: usize = 65536;

// SNMP message is build starting from the end,
// So we use stack-like buffer.
pub(crate) struct Buffer {
    len: usize,
    data: [u8; MAX_SIZE], // @todo: MaybeUninit<u8>
}

impl Default for Buffer {
    #[allow(invalid_value)]
    #[allow(clippy::uninit_assumed_init)]
    fn default() -> Buffer {
        Buffer {
            len: 0,
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl Buffer {
    #[inline]
    pub(crate) fn free(&self) -> usize {
        MAX_SIZE - self.len
    }
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.len
    }
    #[inline]
    pub(crate) fn data(&self) -> &[u8] {
        &self.data[MAX_SIZE - self.len..]
    }
    pub(crate) fn push_u8(&mut self, v: u8) -> Result<(), SnmpError> {
        if self.free() < 1 {
            return Err(SnmpError::OutOfBuffer);
        }
        self.data[MAX_SIZE - self.len - 1] = v;
        self.len += 1;
        Ok(())
    }
    pub(crate) fn push(&mut self, chunk: &[u8]) -> Result<(), SnmpError> {
        let cs = chunk.len();
        if self.free() < cs {
            return Err(SnmpError::OutOfBuffer);
        }
        let i = MAX_SIZE - self.len;
        self.data[i - cs..i].copy_from_slice(chunk);
        self.len += cs;
        Ok(())
    }
    pub(crate) fn push_ber_len(&mut self, v: usize) -> Result<(), SnmpError> {
        if v < 128 {
            // Short form, X.690 pp 8.3.1.4
            self.push_u8(v as u8)?;
        } else {
            // Long form, X.690 pp 8.1.3.5
            let mut left = v;
            let start = self.len();
            while left > 0 {
                self.push_u8((left & 0xff) as u8)?;
                left >>= 8;
            }
            let size = self.len() - start;
            // Push size with high-bit set
            self.push_u8((size | 0x80) as u8)?;
        }
        Ok(())
    }
    pub(crate) fn reset(&mut self) {
        self.len = 0;
    }
    pub(crate) fn as_slice(&self, len: usize) -> &[u8] {
        &self.data[..len]
    }
}

impl AsMut<[u8]> for Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl AsMut<[MaybeUninit<u8>]> for Buffer {
    fn as_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe { &mut *(&mut self.data as *mut [u8] as *mut [MaybeUninit<u8>]) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let b = Buffer::default();
        assert_eq!(b.free(), MAX_SIZE);
        assert_eq!(b.len(), 0);
    }

    #[test]
    fn test_push_u8() -> Result<(), SnmpError> {
        let mut b = Buffer::default();
        b.push_u8(1)?;
        assert_eq!(b.len(), 1);
        assert_eq!(b.free(), MAX_SIZE - 1);
        assert_eq!(b.data(), &[1u8]);
        b.push_u8(2)?;
        assert_eq!(b.len(), 2);
        assert_eq!(b.free(), MAX_SIZE - 2);
        assert_eq!(b.data(), &[2u8, 1]);
        Ok(())
    }

    #[test]
    fn test_push() -> Result<(), SnmpError> {
        let mut b = Buffer::default();
        let chunk = [1u8, 2, 3];
        b.push(&chunk)?;
        assert_eq!(b.len(), chunk.len());
        assert_eq!(b.data(), chunk);
        Ok(())
    }

    #[test]
    fn test_push_ber_len_short1() -> Result<(), SnmpError> {
        let mut b = Buffer::default();
        b.push_ber_len(1)?;
        assert_eq!(b.len(), 1);
        assert_eq!(b.data(), &[1]);
        Ok(())
    }
    #[test]
    fn test_push_ber_len_short2() -> Result<(), SnmpError> {
        let mut b = Buffer::default();
        b.push_ber_len(127)?;
        assert_eq!(b.len(), 1);
        assert_eq!(b.data(), &[127]);
        Ok(())
    }
    #[test]
    fn test_push_ber_len_long1() -> Result<(), SnmpError> {
        let mut b = Buffer::default();
        b.push_ber_len(128)?;
        assert_eq!(b.len(), 2);
        assert_eq!(b.data(), &[0x81, 128]);
        Ok(())
    }
    #[test]
    fn test_push_ber_len_long2() -> Result<(), SnmpError> {
        let mut b = Buffer::default();
        b.push_ber_len(255)?;
        assert_eq!(b.len(), 2);
        assert_eq!(b.data(), &[0x81, 255]);
        Ok(())
    }
    #[test]
    fn test_push_ber_len_long3() -> Result<(), SnmpError> {
        let mut b = Buffer::default();
        b.push_ber_len(256)?;
        assert_eq!(b.len(), 3);
        assert_eq!(b.data(), &[0x82, 1, 0]);
        Ok(())
    }
}
