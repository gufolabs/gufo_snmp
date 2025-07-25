// ------------------------------------------------------------------------
// Gufo SNMP: BerHeader class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::super::error::SnmpError;
use super::{BerClass, Tag};
use nom::{Err, IResult, Needed};

#[derive(Debug)]
pub struct BerHeader {
    // Object class: universal, application, context-specific, or private
    pub class: BerClass,
    // True if costructed
    pub constructed: bool,
    // Tag
    pub tag: Tag,
    // Object length
    pub length: usize,
}

//
// BER TLV format.
// According to ITU-T X.690 pp 8.1.1:
//
// +-------------------+---------------+-----------------+
// | Identifier octets | Length octets | Contents octets |
// +-------------------+---------------+-----------------+
// 8.1.2:
// Identitifier octets:
// Bits 7 and 8 encode class:
// +------------------+----+----+
// | Class            | B8 | B7 |
// +------------------+----+----+
// | Universal        |  0 |  0 |
// | Application      |  0 |  1 |
// | Context-specific |  1 |  0 |
// | Private          |  1 |  1 |
// +------------------+----+----+
// Bit 6
// 0 - primitive
// 1 - constructed
// Bits 5 - 1: tag number
//
impl BerHeader {
    // Implement X.690 pp 8.1.1: Structure of an encoding
    pub fn from_ber(i: &[u8]) -> IResult<&[u8], BerHeader, SnmpError> {
        if i.len() < 2 {
            return Err(Err::Incomplete(Needed::new(2)));
        }
        // Parse identifier octets
        // bits 8, 7 - class
        let mut current = 1;
        let id_octets = i[0];
        let class = match id_octets & 0b1100_0000 {
            0 => BerClass::Universal,
            0b0100_0000 => BerClass::Application,
            0b1000_0000 => BerClass::Context,
            0b1100_0000 => BerClass::Private,
            _ => BerClass::Universal, // Unreachable
        };
        // bit 6 - costructed
        let constructed = (id_octets & 0b0010_0000) != 0;
        // bits 5 - 1 tag number
        let tag = match id_octets & 0x1f {
            0x1f => {
                // > 30
                let mut n = 0 as Tag;
                loop {
                    // @todo: check size
                    let t = i[current];
                    current += 1;
                    n = (n << 7) | ((t & 0x7f) as Tag);
                    if t & 0x80 == 0 {
                        break;
                    }
                }
                n
            }
            n => n as Tag,
        };
        // Parse length offset
        // X.690 8.3.1.4-8.3.1.5
        // @todo: Indefinite length
        let n = i[current];
        current += 1;
        let length = if n & 0x80 == 0 {
            // Short form, X.690 pp 8.3.1.4
            n as usize
        } else {
            // Long form, X.690 pp 8.1.3.5
            let mut ln = 0;
            for _ in 0..n & 0x7f {
                ln = (ln << 8) + (i[current] as usize);
                current += 1;
            }
            ln
        };
        // Check content size
        if i[current..].len() < length {
            return Err(Err::Incomplete(Needed::new(length - i[current..].len())));
        }
        Ok((
            &i[current..],
            BerHeader {
                class,
                constructed,
                tag,
                length,
            },
        ))
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SnmpResult;

    // Too short header
    #[test]
    fn test_incomplete_header() {
        let header = [0u8];
        let r = BerHeader::from_ber(&header);
        assert!(r.is_err());
    }

    // Null, zero-length content
    #[test]
    fn test_header_null() -> SnmpResult<()> {
        let data = [5u8, 0u8];
        let (tail, hdr) = BerHeader::from_ber(&data)?;
        assert_eq!(hdr.class, BerClass::Universal);
        assert!(!hdr.constructed);
        assert_eq!(hdr.tag, 5);
        assert_eq!(hdr.length, 0);
        assert_eq!(tail.len(), 0);
        Ok(())
    }

    // Octet-stream
    #[test]
    fn test_header_str() -> SnmpResult<()> {
        let data = [4u8, 5, 0x74, 0x65, 0x73, 0x74, 0x2e];
        let (tail, hdr) = BerHeader::from_ber(&data)?;
        assert_eq!(hdr.class, BerClass::Universal);
        assert!(!hdr.constructed);
        assert_eq!(hdr.tag, 4);
        assert_eq!(hdr.length, 5);
        assert_eq!(tail.len(), 5);
        assert_eq!(tail, &data[2..]);
        Ok(())
    }

    // Non-primitive sequence
    #[test]
    fn test_header_sequence() -> SnmpResult<()> {
        // Sequece of 3 nulls
        let data = [0x30u8, 6, 5, 0, 5, 0, 5, 0];
        let (tail, hdr) = BerHeader::from_ber(&data)?;
        assert_eq!(hdr.class, BerClass::Universal);
        assert!(hdr.constructed);
        assert_eq!(hdr.tag, 0x10);
        assert_eq!(hdr.length, 6);
        assert_eq!(tail.len(), 6);
        assert_eq!(tail, &data[2..]);
        Ok(())
    }
}
