use bitstring::BitString;
use crate::dyn_bit_string::DynBitString;
use crate::encoding_uint_trait::{EncodingUint, UintEncoding};

// encode very small integers using variable-length encoding

pub struct SmallIntEncoding {
    pub encoding : UintEncoding
}

impl EncodingUint for SmallIntEncoding {
    fn new() -> Self {
        let ec = UintEncoding { bstr: DynBitString::null() };
        SmallIntEncoding { encoding: ec }
    }

    fn get_encoding(&self) -> DynBitString {
        self.encoding.bstr.clone()
    }

    fn append_uint32(& mut self, len_in: u32) {
        assert!(len_in < 8);  // FIXME: handle case where it doesn't fit in 3 bits
        let mut v = len_in;
        let bs = &mut  self.encoding.bstr;
        for _k in 0..3 { // from least significant to most significant
            let next_bit = (v & 1) != 0;
            bs.append(next_bit);
            v >>= 1;
        }
    }

    // inverse of append_uint32_len
    // note that first bit is least significant bit, same as above

    fn read_uint32(&self, bitstring_cursor: &mut usize) -> u32 {
        let mut v = 0;
        let mut bitmask = 1;
        let bs = self.get_encoding();
        for _j in 0..3 {
            if bs.get(*bitstring_cursor) {
                v |= bitmask;
            }
            *bitstring_cursor += 1;
            bitmask <<= 1;
        }
        v
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    pub fn test_append_uint32() {
        use std::str::FromStr;
        let mut t = SmallIntEncoding::new();
        t.append_uint32(5);
        let bs = t.get_encoding();
        let expected_bs = DynBitString::from_str("b101").unwrap();
        assert_eq!(bs, expected_bs);
    }

    #[test]
    pub fn test_read_uint32() {
        let mut t = SmallIntEncoding::new();
        t.append_uint32(6);
        t.append_uint32(7);
        let mut cursor : usize = 0;
        let v1 = t.read_uint32(& mut cursor);
        let v2 = t.read_uint32(& mut cursor);
        assert_eq!(v1, 6);
        assert_eq!(v2, 7);
    }
}