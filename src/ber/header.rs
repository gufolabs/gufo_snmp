// ------------------------------------------------------------------------
// Gufo Snmp: BerHeader class
// ------------------------------------------------------------------------
// Copyright (C) 2023, Gufo Labs
// See LICENSE.md for details
// ------------------------------------------------------------------------

use super::super::error::SnmpError;
use super::BerClass;
use nom::{Err, IResult, Needed};

#[derive(Debug)]
pub(crate) struct BerHeader {
    // Object class: universal, application, context-specific, or private
    pub class: BerClass,
    // True if costructed
    pub constructed: bool,
    // Tag
    pub tag: usize,
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
    pub(crate) fn from_ber(i: &[u8]) -> IResult<&[u8], BerHeader, SnmpError> {
        if i.len() < 2 {
            return Err(Err::Incomplete(Needed::new(2)));
        }
        // Parse identifier octets
        // bits 8, 7 - class
        let mut current = 1;
        let id_octets = i[0];
        let class = match (id_octets >> 6) & 0x3 {
            0 => BerClass::Universal,
            1 => BerClass::Application,
            2 => BerClass::Context,
            3 => BerClass::Private,
            _ => BerClass::Universal, // Unreachable
        };
        // bit 6 - costructed
        let constructed = ((id_octets >> 5) & 0x1) == 0x1;
        // bits 5 - 1 tag number
        let tag = match id_octets & 0x1f {
            0x1f => {
                // > 30
                let mut n = 0usize;
                loop {
                    // @todo: check size
                    let t = i[current];
                    current += 1;
                    n = (n << 7) | ((t & 0x7f) as usize);
                    if t & 0x80 == 0 {
                        break;
                    }
                }
                n
            }
            n => n as usize,
        };
        // Parse length offset
        // @todo: Indefinite length
        let mut length = 0usize;
        loop {
            // @todo: check size
            let n = i[current];
            current += 1;
            length = (length << 7) | ((n & 0x7f) as usize);
            if n & 0x80 == 0 {
                break;
            }
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // Too short header
    #[test]
    fn test_incomplete_header() {
        let header = [0u8];
        let r = BerHeader::from_ber(&header);
        assert!(r.is_err());
    }

    // Null, zero-length content
    #[test]
    fn test_header_null() -> Result<(), SnmpError> {
        let data = [5u8, 0u8];
        let (tail, hdr) = BerHeader::from_ber(&data)?;
        assert_eq!(hdr.class, BerClass::Universal);
        assert_eq!(hdr.constructed, false);
        assert_eq!(hdr.tag, 5);
        assert_eq!(hdr.length, 0);
        assert_eq!(tail.len(), 0);
        Ok(())
    }

    // Octet-stream
    #[test]
    fn test_header_str() -> Result<(), SnmpError> {
        let data = [4u8, 5, 0x74, 0x65, 0x73, 0x74, 0x2e];
        let (tail, hdr) = BerHeader::from_ber(&data)?;
        assert_eq!(hdr.class, BerClass::Universal);
        assert_eq!(hdr.constructed, false);
        assert_eq!(hdr.tag, 4);
        assert_eq!(hdr.length, 5);
        assert_eq!(tail.len(), 5);
        assert_eq!(tail, &data[2..]);
        Ok(())
    }

    // Non-primitive sequence
    #[test]
    fn test_header_sequence() -> Result<(), SnmpError> {
        // Sequece of 3 nulls
        let data = [0x30u8, 6, 5, 0, 5, 0, 5, 0];
        let (tail, hdr) = BerHeader::from_ber(&data)?;
        assert_eq!(hdr.class, BerClass::Universal);
        assert_eq!(hdr.constructed, true);
        assert_eq!(hdr.tag, 0x10);
        assert_eq!(hdr.length, 6);
        assert_eq!(tail.len(), 6);
        assert_eq!(tail, &data[2..]);
        Ok(())
    }
}
