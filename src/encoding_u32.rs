use bitstring::BitString;
use crate::encoding_uint_trait::{EncodingUint, UintEncoding};
use crate::dyn_bit_string::*;
use std::num::NonZeroU32;
use crate::encoding_uint_trait::{BITSTRING_CONTINUE, BITSTRING_END};

// bit offsets where decision to end/continue bitstring
// the last array value is just there so we don't get out-of-bounds array reference

const CONTINUE_OFFSETS: [usize; 2] = [ 1, 5 ];

pub struct U32Encoding {
    pub encoding : UintEncoding
}

impl EncodingUint for U32Encoding {

    fn new() -> Self {
        U32Encoding { encoding: UintEncoding { bstr : DynBitString::null() }}
    }

    fn get_bitstr_encoding(&self) -> DynBitString {
        self.encoding.bstr.clone()
    }
    fn from_bitstr_encoding( bs : DynBitString ) -> Self {
        U32Encoding { encoding: UintEncoding { bstr : bs }}
    }
    fn append_uint32(&mut self, v_in: u32) {
        let bstr = &mut self.encoding.bstr;
        // encode the length of the length
        let leading_0s = if v_in == 0 {
            NonZeroU32::BITS
        } else {
            NonZeroU32::new(v_in).unwrap().leading_zeros()
        };
        let mut len_bitct = NonZeroU32::BITS - leading_0s;
        #[allow(clippy::implicit_saturating_sub)]
        if len_bitct != 0 { len_bitct -= 1; } // so it fits in 5 bits
        let mut continue_offsets_index: usize = 0;  // position in continue_offsets array
        for k in 0..5 {  // length of length in bits is at most 2^5 - 1
            let next_bit: bool = len_bitct & 1 != 0;
            bstr.append(next_bit);
            len_bitct >>= 1;
            if CONTINUE_OFFSETS[continue_offsets_index] == k {
                if len_bitct == 0 {
                    bstr.append(BITSTRING_END);
                    break;
                } else {
                    bstr.append(BITSTRING_CONTINUE);
                    continue_offsets_index += 1;
                }
            }
        }
        assert_eq!(len_bitct, 0);

        // we could replace this bit-by-bit loop
        // with something more efficient later

        let mut v = v_in;
        if v == 0 {
            // special case v=0 to have a 1-bit 0 encoded
            bstr.append(false);
        } else {
            while v > 0 {
                let next_bit = (v & 1) != 0;
                bstr.append(next_bit);
                v >>= 1;
            }
        }
    }

    // inverse of append_uint32()
    // read an unsigned 32-bit integer from the current position in the bitstring
    // and return it, while also updating the bitstring cursor
    // the caller must initialize the cursor to zero before calling
    // read_uint32 for the first time.

    fn read_uint32(&self, bitstring_cursor: &mut usize) -> u32 {
        let enc_len_val = &self.encoding.bstr;
        let mut vlen: u32 = 0;
        let mut continue_offsets_index: usize = 0;  // position in continue_offsets array
        let mut bitct_mask: u32 = 1;                // next bit to process from bitstring length
        for k in 0..5 {
            let next_bit_1: bool = enc_len_val.get(*bitstring_cursor);
            *bitstring_cursor += 1;
            if next_bit_1 {
                vlen |= bitct_mask;
            }
            bitct_mask <<= 1;
            if CONTINUE_OFFSETS[continue_offsets_index] == k {
                continue_offsets_index += 1;

                let next_continue_bit: bool = enc_len_val.get(*bitstring_cursor);
                *bitstring_cursor += 1;
                if !next_continue_bit {
                    break;
                }
            }
        }
        vlen += 1;
        assert!(vlen < 33);
        // we now have the length of the integer in vlen
        // now decode integer of vlen bits
        // someday we can stop doing this bit-by-bit

        let mut v = 0;
        bitct_mask = 1;
        for _j in 0..vlen {
            if enc_len_val.get(*bitstring_cursor) {
                v |= bitct_mask;
            }
            *bitstring_cursor += 1;
            bitct_mask <<= 1;
        }
        v
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::str::FromStr;

    // expect a dynamic bitstring specified by the text string parameter
    // useful for regression testing

    #[allow(dead_code)]
    fn expect_dbstr(bs: crate::dyn_bit_string::DynBitString, expected_bitstring_text: &str) {
        let r = DynBitString::from_str(expected_bitstring_text);
        match r {
            Err(e) => {
                panic!("ERROR - {}", e);
            }
            Ok(expected_bs) => {
                if expected_bs != bs {
                    panic!("expected bitstring {:?}, got bitstring {:?}\n", expected_bs, bs);
                }
            }
        }
    }

    #[allow(dead_code)]
    // encode a single integer, not an integer sequence
    fn encode_uint32(v_in: u32) -> DynBitString {
        let mut int_encoding = U32Encoding::new();
        int_encoding.append_uint32(v_in);
        int_encoding.get_bitstr_encoding()
    }

    // when you only need to decode a single integer, not an integer sequence
    #[allow(dead_code)]
    fn decode_uint32(enc_len_val: &DynBitString) -> u32 {
        let mut bitstring_cursor: usize = 0;
        let u32_enc = U32Encoding::from_bitstr_encoding(enc_len_val.clone());
        u32_enc.read_uint32(&mut bitstring_cursor)
    }

    // each bit string length test expected value is encoded as the sum of the
    // length of the length and the length of the actual value
    #[test]
    pub fn test_encode_uint32() {
        let mut sm = encode_uint32(0);
        expect_dbstr(sm, "b0000");

        sm = encode_uint32(1);
        expect_dbstr(sm, "b0001");

        sm = encode_uint32(2);
        expect_dbstr(sm, "b10001");

        sm = encode_uint32(3);
        expect_dbstr(sm, "b10011");

        sm = encode_uint32(7);
        //expect_dbstr(sm, "b01100111");
        expect_dbstr(sm, "b010111");

        sm = encode_uint32(8);
        expect_dbstr(sm, "b1100001");

        sm = encode_uint32(65535);
        expect_dbstr(sm, "b1111101111111111111111");

        sm = encode_uint32(0xffffffff);
        expect_dbstr(sm, "b11111111111111111111111111111111111111");
    }

    #[test]
    pub fn test_decode_uint32() {
        for j in 0..2 << 20 {
            let sm = encode_uint32(j as u32);
            let v: u32 = decode_uint32(&sm);
            assert_eq!(v, j as u32);
        }
    }
}
