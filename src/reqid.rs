// ------------------------------------------------------------------------
// Gufo SNMP: Id Generator
// ------------------------------------------------------------------------
// Copyright (C) 2023-24, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use rand::Rng;

const MAX_REQUEST_ID: i64 = 0x7fffffff;

#[derive(Default)]
pub struct RequestId(i64);

impl RequestId {
    /// Get next value
    pub fn get_next(&mut self) -> i64 {
        let mut rng = rand::rng();
        let x: i64 = rng.random();
        self.0 = x & MAX_REQUEST_ID;
        self.0
    }
    /// Check values for match
    pub fn check(&self, v: i64) -> bool {
        self.0 == v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let r = RequestId::default();
        assert!(r.check(0))
    }

    #[test]
    fn test_check() {
        let mut r = RequestId::default();
        let v1 = r.get_next();
        assert!(r.check(v1))
    }

    #[test]
    fn test_seq() {
        let mut r = RequestId::default();
        let v1 = r.get_next();
        let v2 = r.get_next();
        assert!(v1 != v2)
    }
}
