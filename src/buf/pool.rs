// ------------------------------------------------------------------------
// Gufo SNMP: Buffer pool implementation
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::buffer::Buffer;
use std::sync::{Arc, Mutex, OnceLock};

pub struct BufferPool {
    pool: Arc<Mutex<Vec<Buffer>>>,
}

impl Default for BufferPool {
    fn default() -> Self {
        BufferPool {
            pool: Arc::new(Mutex::new(Vec::default())),
        }
    }
}

impl BufferPool {
    pub fn acquire(&self) -> BufferHandle {
        let mut pool = self.pool.lock().unwrap();
        BufferHandle {
            pool: Arc::clone(&self.pool),
            buf: Some(pool.pop().unwrap_or_default()),
        }
    }
}

pub struct BufferHandle {
    pool: Arc<Mutex<Vec<Buffer>>>,
    buf: Option<Buffer>, // to use with take
}

impl AsMut<Buffer> for BufferHandle {
    fn as_mut(&mut self) -> &mut Buffer {
        self.buf.as_mut().unwrap()
    }
}

impl Drop for BufferHandle {
    fn drop(&mut self) {
        if let Some(mut buf) = self.buf.take() {
            buf.reset();
            let mut pool = self.pool.lock().unwrap();
            pool.push(buf);
        }
    }
}

pub static BUFFER_POOL: OnceLock<BufferPool> = OnceLock::new();

pub fn get_buffer_pool() -> &'static BufferPool {
    BUFFER_POOL.get_or_init(BufferPool::default)
}
