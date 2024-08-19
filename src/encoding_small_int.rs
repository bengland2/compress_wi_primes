use bitstring::BitString;
use crate::dyn_bit_string::DynBitString;
use crate::encoding_uint_trait::{BITSTRING_CONTINUE, BITSTRING_END, EncodingUint, UintEncoding};

// encode very small unsigned integers using variable-length encoding
// this has to handle numbers up through 31, which is:
// - maximum possible exponent-1
// - theoretical upper bound on number of prime factors
// encode starting from least significant bit to
// most significant bit

pub struct SmallIntEncoding {
    pub encoding : UintEncoding
}

impl EncodingUint for SmallIntEncoding {
    fn new() -> Self {
        SmallIntEncoding { encoding: UintEncoding { bstr: DynBitString::null() }}
    }

    fn get_bitstr_encoding(&self) -> DynBitString {
        self.encoding.bstr.clone()
    }

    fn from_bitstr_encoding(bs : DynBitString) -> Self {
        SmallIntEncoding { encoding: UintEncoding { bstr: bs }}
    }

    fn append_uint32(& mut self, v_in: u32) {
        assert!(v_in < (1 << 5));
        let mut v = v_in;
        let bs = &mut  self.encoding.bstr;
        let first_bit = (v & 1) != 0;
        bs.append(first_bit);
        v >>= 1;
        if v == 0 {
            bs.append(BITSTRING_END);
        } else {
            bs.append(BITSTRING_CONTINUE);
            for _k in 0..2 { // from least significant to most significant
                let next_bit = (v & 1) != 0;
                bs.append(next_bit);
                v >>= 1;
            }
            if v == 0 {
                bs.append(BITSTRING_END);
            } else {
                bs.append(BITSTRING_CONTINUE);
                for _k in 0..2 {
                    let next_bit = (v & 1) != 0;
                    bs.append(next_bit);
                    v >>= 1;
                }
            }
        }
        assert_eq!(v, 0);
    }

    // inverse of append_uint32_len
    // note that first bit is least significant bit, same as above

    fn read_uint32(&self, bitstring_cursor: &mut usize) -> u32 {
        let mut v = 0;
        let mut bitmask = 1;
        let bs = self.get_bitstr_encoding();
        if bs.get(*bitstring_cursor) {
            v |= bitmask;
        }
        *bitstring_cursor += 1;
        bitmask <<= 1;
        let continue_bit = bs.get(*bitstring_cursor);
        *bitstring_cursor += 1;
        if continue_bit == BITSTRING_CONTINUE {
            for _j in 0..2 {
                if bs.get(*bitstring_cursor) {
                    v |= bitmask;
                }
                *bitstring_cursor += 1;
                bitmask <<= 1;
            }
            let continue_bit2 = bs.get(*bitstring_cursor);
            *bitstring_cursor += 1;
            if continue_bit2 == BITSTRING_CONTINUE {
                for _j in 0..2 {
                    if bs.get(*bitstring_cursor) {
                        v |= bitmask;
                    }
                    *bitstring_cursor += 1;
                    bitmask <<= 1;
                }
            }
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
        let test_cases = vec![
            (0, "b00"),
            (1, "b10"),
            (2, "b01100"),
            (5, "b11010"),
            (6, "b01110"),
            (7, "b11110"),
            (15,"b1111110"),
            (16,"b0100101"),
            (31,"b1111111") ];

        for (v, enc_str) in test_cases {
            let mut t = SmallIntEncoding::new();
            t.append_uint32(v);
            let bs = t.get_bitstr_encoding();
            let expected_bs = DynBitString::from_str(enc_str).unwrap();
            assert_eq!(bs, expected_bs);
        }
    }

    #[test]
    pub fn test_read_uint32() {
        let mut t = SmallIntEncoding::new();
        let v_in : [u32; 5] = [ 0, 1, 2, 6, 7 ];
        for k in 0..5 {
            t.append_uint32(v_in[k]);
        }
        let mut cursor : usize = 0;
        for i in 0..5 {
            assert_eq!(t.read_uint32(&mut cursor), v_in[i]);
        }
    }
}