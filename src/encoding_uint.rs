
use std::num::NonZeroU32;
use bitstring::BitString;
use crate::dyn_bit_string::*;

const BITSTRING_CONTINUE : bool = true;
const BITSTRING_END : bool = false;

// bit offsets where decision to end/continue bitstring
// the last array value [2] is just there so we don't get out-of-bounds array reference

const CONTINUE_OFFSETS: [usize; 3] = [ 1, 3, 5 ];  // used to be 0, 2, 5

// since the values of a IntAsPrms prm_powers array length are < 8 almost always,
// encode them in 3 bits where possible instead of using variable-length encoding

pub fn append_uint32_len(len_in : u32, length_and_val : & mut DynBitString) {
    assert!(len_in < 8);  // FIXME: handle case where it doesn't fit in 3 bits
    let mut v = len_in;
    for _k in 0..3 {
        let next_bit = (v & 1) != 0;
        length_and_val.append(next_bit);
        v >>= 1;
    }
}

// inverse of append_uint32_len
// note that first bit is least significant bit, same as above

pub fn read_uint32_len(enc_len_val: &DynBitString, bitstring_cursor : & mut usize) -> u32 {
    let mut v = 0;
    let mut bitmask = 1;
    for _j in 0..3 {
        if enc_len_val.get(*bitstring_cursor) {
            v |= bitmask;
        }
        *bitstring_cursor += 1;
        bitmask <<= 1;
    }
    v

}

// append an encoding of a u32 to the length_and_val bitstring input
// encode unsigned integers, particularly small ones, in as few bits as possible.
// first we compute the length in bits of the integer to be encoded using
// native machine instruction
// then we encode the length using a variable length encoding
// maximum length of a length for a 32-bit unsigned is 5 bits
// we disallow 0-length length (not meaningful)
// and allow 32-bit length by subtracting 1 from actual length of length
// examples:
// 0 requires 1 bit,
// 1 requires 1 bit,
// 2 requires 2 bits
// 3 requires 2 bits
// 4 requires 3 bits
// etc.


pub fn append_uint32(v_in : u32, length_and_val : & mut DynBitString) {
    // encode the length of the length
    let leading_0s = if v_in == 0 {
        NonZeroU32::BITS
    } else {
        std::num::NonZeroU32::new(v_in).unwrap().leading_zeros()
    };
    let mut len_bitct = NonZeroU32::BITS - leading_0s;
    #[allow(clippy::implicit_saturating_sub)]
    if len_bitct != 0 { len_bitct -= 1;  } // so it fits in 5 bits
    let mut continue_offsets_index : usize = 0;  // position in continue_offsets array
    for k in 0..5 {
        let next_bit : bool = len_bitct & 1 != 0;
        length_and_val.append(next_bit);
        len_bitct >>= 1;
        if CONTINUE_OFFSETS[continue_offsets_index] == k {
            if len_bitct == 0 {
                length_and_val.append(BITSTRING_END);
                break;
            } else {
                length_and_val.append(BITSTRING_CONTINUE);
                continue_offsets_index += 1;
            }
        }
    }
    // FIXME:: can we optimize away this loop?

    let mut v = v_in;
    if v == 0 {
        // special case v=0 to have a 1-bit 0 encoded
        length_and_val.append(false);
    } else {
        while v > 0 {
            let next_bit = (v & 1) != 0;
            length_and_val.append(next_bit);
            v >>= 1;
        }
    }
}

// use this when you only need to encode a single integer, not an integer sequence
pub fn encode_uint32(v_in : u32) -> DynBitString {
    let mut l_and_v = DynBitString::null();
    append_uint32(v_in, &mut l_and_v);
    l_and_v
}

// inverse of append_uint32()
// read an unsigned 32-bit integer from the current position in the bitstring
// and return it, while also updating the bitstring cursor
// the caller must initialize the cursor to zero before calling
// read_uint32 for the first time.

pub fn read_uint32(enc_len_val: &DynBitString, bitstring_cursor : & mut usize) -> u32 {
    let mut vlen : u32 = 0;
    let mut continue_offsets_index : usize = 0;  // position in continue_offsets array
    let mut bitct_mask : u32 = 1;                // next bit to process from bitstring length
    for k in 0..5 {
        let next_bit_1 : bool = enc_len_val.get(*bitstring_cursor);
        *bitstring_cursor += 1;
        if next_bit_1 {
            vlen |= bitct_mask;
        }
        bitct_mask <<= 1;
        if CONTINUE_OFFSETS[continue_offsets_index] == k {
            continue_offsets_index += 1;

            let next_continue_bit : bool = enc_len_val.get(*bitstring_cursor);
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
    let mut v = 0;
    bitct_mask =  1;
    for _j in 0..vlen {
        if enc_len_val.get(*bitstring_cursor) {
            v |= bitct_mask;
        }
        *bitstring_cursor += 1;
        bitct_mask <<= 1;
    }
    v
}

// when you only need to decode a single integer, not an integer sequence
pub fn decode_uint32(enc_len_val: &DynBitString) -> u32 {
    let mut bitstring_cursor: usize = 0;
    read_uint32(enc_len_val, & mut bitstring_cursor)
}

#[cfg(test)]
pub mod tests {

    // expect a dynamic bitstring specified by the text string parameter
    // useful for regression testing

    #[allow(dead_code)]
    fn expect_dbstr( bs : crate::dyn_bit_string::DynBitString, expected_bitstring_text : &str) {
        use std::str::FromStr;
        use crate::dyn_bit_string::DynBitString;

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

    // each bit string length test expected value is encoded as the sum of the
    // length of the length and the length of the actual value
    #[test]
    pub fn test_encode_uint32() {
        use super::encode_uint32;

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
        expect_dbstr(sm, "b111111111111111111111111111111111111111");
    }


    #[test]
    pub fn test_decode_uint32() {
        use super::encode_uint32;
        use super::decode_uint32;

        for j in 0..2<<20 {
            let sm = encode_uint32(j as u32);
            let v : u32 = decode_uint32(&sm);
            assert!(v == (j as u32));
        }
    }

    #[test]
    pub fn test_uint32_len_encoding() {
        use super::{append_uint32_len, read_uint32_len};
        use crate::dyn_bit_string::DynBitString;
        use bitstring::BitString;

        for l in 0..8 {
            let mut sm : DynBitString = DynBitString::null();
            append_uint32_len(l, &mut sm);
            let mut bs_cursor : usize = 0;
            let l2 = read_uint32_len(&sm, &mut bs_cursor);
            assert!(l == l2);
        }
    }
}
