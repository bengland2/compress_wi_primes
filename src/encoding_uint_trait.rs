use crate::dyn_bit_string::DynBitString;

// encode/decode a sequence of unsigned integer values

pub struct UintEncoding {
    pub bstr : DynBitString,
}

pub trait EncodingUint {
    // create new instance
    fn new() -> Self;

    // get bitstring encoding
    fn get_encoding(&self) -> DynBitString;

    // concatenate u32 encoding to a previously existing bit string
    fn append_uint32(&mut self, v_in : u32);

    // generate bit string with just this u32 encoded in it
    //fn encode_uint32(v_in : u32) -> DynBitString;

    // read the next u32 encoding from a bit string at the bit offset indicated by the cursor
    // cursor must be initialized to zero before using it
    fn read_uint32(&self, bitstring_cursor : & mut usize) -> u32;

    // convert bitstring to a single u32 value (use if only 1 u32 encoding in the bitstring)
    //fn decode_uint32(enc_len_val: &DynBitString) -> u32;
}