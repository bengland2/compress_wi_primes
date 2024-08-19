use crate::dyn_bit_string::DynBitString;

// encode/decode a sequence of unsigned integer values

#[derive(Debug)]
pub struct UintEncoding {
    pub bstr : DynBitString,
}

pub const BITSTRING_CONTINUE : bool = true;
pub const BITSTRING_END : bool = false;

pub trait EncodingUint {
    // create new instance
    fn new() -> Self;

    // get bitstring encoding
    fn get_bitstr_encoding(&self) -> DynBitString;

    // prepare to decode an encoded bitstring using read_
    fn from_bitstr_encoding( bstr_in : DynBitString ) -> Self;

    // concatenate u32 encoding to a previously existing bit string
    fn append_uint32(&mut self, v_in : u32);

    // read the next u32 encoding from a bit string at the bit offset indicated by the cursor
    // cursor must be initialized to zero before using it
    fn read_uint32(&self, bitstring_cursor : & mut usize) -> u32;
}