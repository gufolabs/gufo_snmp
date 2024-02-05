// ------------------------------------------------------------------------
// Gufo SNMP: Buffer implementation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use crate::error::{SnmpError, SnmpResult};
use std::mem::MaybeUninit;
use std::ptr;
use std::slice;

const MAX_SIZE: usize = 4080;

// SNMP message is build starting from the end,
// So we use stack-like buffer.
pub struct Buffer {
    pos: usize,
    bookmark: usize,
    data: [MaybeUninit<u8>; MAX_SIZE],
}

impl Default for Buffer {
    #[allow(invalid_value)]
    #[allow(clippy::uninit_assumed_init)]
    fn default() -> Buffer {
        Buffer {
            pos: MAX_SIZE,
            bookmark: 0,
            data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl Buffer {
    #[inline]
    pub fn free(&self) -> usize {
        self.pos
    }
    #[inline]
    pub fn len(&self) -> usize {
        MAX_SIZE - self.pos
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pos == MAX_SIZE
    }
    #[inline]
    pub fn ensure_size(&self, v: usize) -> SnmpResult<()> {
        if self.pos < v {
            return Err(SnmpError::OutOfBuffer);
        }
        Ok(())
    }
    #[inline]
    pub fn is_full(&self) -> bool {
        self.pos == 0
    }
    #[inline]
    pub fn data(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.data.as_ptr().add(self.pos) as *const u8,
                MAX_SIZE - self.pos,
            )
        }
    }
    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                self.data.as_mut_ptr().add(self.pos) as *mut u8,
                MAX_SIZE - self.pos,
            )
        }
    }
    #[inline]
    pub fn set_bookmark(&mut self, delta: usize) {
        self.bookmark = self.pos + delta;
    }
    #[inline]
    pub fn get_bookmark(&self) -> usize {
        self.bookmark - self.pos
    }
    #[inline]
    pub fn skip(&mut self, size: usize) {
        if self.pos < size {
            self.pos = 0
        } else {
            self.pos -= size
        }
    }
    #[inline]
    fn push_u8_unchecked(&mut self, v: u8) {
        self.pos -= 1;
        unsafe {
            self.data[self.pos].as_mut_ptr().write(v);
        }
    }
    #[inline]
    pub fn push_u8(&mut self, v: u8) -> SnmpResult<()> {
        if self.is_full() {
            return Err(SnmpError::OutOfBuffer);
        }
        self.push_u8_unchecked(v);
        Ok(())
    }
    #[inline]
    pub fn push(&mut self, chunk: &[u8]) -> SnmpResult<()> {
        let cs = chunk.len();
        self.ensure_size(cs)?;
        self.pos -= cs;
        unsafe {
            ptr::copy_nonoverlapping(
                chunk.as_ptr(),
                self.data[self.pos..].as_mut_ptr() as *mut u8,
                cs,
            );
        }
        Ok(())
    }
    #[inline]
    pub fn push_tag_len(&mut self, tag: u8, v: usize) -> SnmpResult<()> {
        if v < 128 {
            // Short form, X.690 pp 8.3.1.4
            // <tag>, <v>
            self.ensure_size(2)?;
            self.push_u8_unchecked(v as u8);
            self.push_u8_unchecked(tag);
            return Ok(());
        }
        if v < 256 {
            // Long form, X.690 pp 8.1.3.5
            // <tag>, 0x81, <v>
            self.ensure_size(3)?;
            self.push_u8_unchecked(v as u8);
            self.push_u8_unchecked(0x81);
            self.push_u8_unchecked(tag);
        } else {
            // Long form, X.690 pp 8.1.3.5
            // <tag>, 0x82, <vh>, <vl>
            self.ensure_size(4)?;
            self.push_u8_unchecked(v as u8);
            self.push_u8_unchecked((v >> 8) as u8);
            self.push_u8_unchecked(0x82);
            self.push_u8_unchecked(tag);
        }
        Ok(())
    }
    // Push tag, len, data
    #[inline]
    pub fn push_tagged(&mut self, tag: u8, data: &[u8]) -> SnmpResult<()> {
        self.push(data)?;
        self.push_tag_len(tag, data.len())
    }
    #[inline]
    pub fn reset(&mut self) {
        self.pos = MAX_SIZE;
    }
    pub fn as_slice(&self, len: usize) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.as_ptr() as *const u8, len) }
    }
}

impl AsMut<[u8]> for Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                self.data[self.pos..].as_mut_ptr() as *mut u8,
                MAX_SIZE - self.pos,
            )
        }
    }
}

impl AsMut<[MaybeUninit<u8>]> for Buffer {
    fn as_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.data
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
    fn test_push_u8() -> SnmpResult<()> {
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
    fn test_push() -> SnmpResult<()> {
        let mut b = Buffer::default();
        let chunk = [1u8, 2, 3];
        b.push(&chunk)?;
        assert_eq!(b.len(), chunk.len());
        assert_eq!(b.data(), chunk);
        Ok(())
    }

    #[test]
    fn test_push_tag_len_short1() -> SnmpResult<()> {
        let mut b = Buffer::default();
        b.push_tag_len(4, 1)?;
        assert_eq!(b.len(), 2);
        assert_eq!(b.data(), &[4, 1]);
        Ok(())
    }
    #[test]
    fn test_push_tag_len_short2() -> SnmpResult<()> {
        let mut b = Buffer::default();
        b.push_tag_len(4, 127)?;
        assert_eq!(b.len(), 2);
        assert_eq!(b.data(), &[4, 127]);
        Ok(())
    }
    #[test]
    fn test_push_tag_len_long1() -> SnmpResult<()> {
        let mut b = Buffer::default();
        b.push_tag_len(4, 128)?;
        assert_eq!(b.len(), 3);
        assert_eq!(b.data(), &[4, 0x81, 128]);
        Ok(())
    }
    #[test]
    fn test_push_tag_len_long2() -> SnmpResult<()> {
        let mut b = Buffer::default();
        b.push_tag_len(4, 255)?;
        assert_eq!(b.len(), 3);
        assert_eq!(b.data(), &[4, 0x81, 255]);
        Ok(())
    }
    #[test]
    fn test_push_tag_len_long3() -> SnmpResult<()> {
        let mut b = Buffer::default();
        b.push_tag_len(4, 256)?;
        assert_eq!(b.len(), 4);
        assert_eq!(b.data(), &[4, 0x82, 1, 0]);
        Ok(())
    }
}
