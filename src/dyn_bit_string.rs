use std::fmt;
//use std::num::NonZeroU32;
use bitstring::BitString;
use std::str::FromStr;

pub struct DynBitString {
    cnt: usize,       // number of bits
    b : Vec<u8>         // dynamically allocated byte array
}

//pub is_big_endian : bool = 0x12345678u32.to_be_bytes() == [ 0x12, 0x34, 0x56, 0x78 ];

#[derive(PartialEq)]
#[derive(Debug)]
pub enum DBSGetBitErr {
    CountPastEnd,
    StartingAtTooBig
}

pub const BITS_PER_BYTE : usize = 8;

impl BitString for DynBitString {


    fn get(&self, ndx: usize) -> bool {
        assert!(ndx < self.cnt);
        let byte_index = ndx / BITS_PER_BYTE;
        let bit_index_within_byte = ndx % BITS_PER_BYTE;
        self.b[byte_index] & (1 << bit_index_within_byte) != 0
    }

    fn set(&mut self, ndx: usize, bit: bool) {
        assert!(ndx < self.cnt);
        let byte_index = ndx / BITS_PER_BYTE;
        let bit_index_within_byte = ndx % BITS_PER_BYTE;
        let bit_shift = 1 << bit_index_within_byte;
        if bit {
            self.b[byte_index] |= bit_shift;
        } else {
            self.b[byte_index] &= !bit_shift;
        }
    }

    fn flip(&mut self, ndx: usize) {
        self.set(ndx, !self.get(ndx));
    }

    fn len(&self) -> usize { self.cnt }

    fn clip(&mut self, newsz: usize) {
        #[allow(clippy::comparison_chain)]
        if newsz < self.cnt {
            // don't shrink vector but zero out bits from newsz to end
            for k in newsz..self.cnt {
                self.set(k, false);
            }
            self.cnt = newsz;
        } else if newsz > self.cnt {
            let old_bitcnt = self.cnt;
            for _k in old_bitcnt..newsz {
                self.append(false);
            }
        }
        // clip does NOTHING if new size is same as old size
    }
    fn append(&mut self, bit: bool) {
        if self.cnt % BITS_PER_BYTE == 0 {
            if self.b.len() * BITS_PER_BYTE == self.cnt {
                self.b.push(0);  // allocate another 8 bits
            } else {
                assert!(self.b.len() * BITS_PER_BYTE > self.cnt);
            }
        }
        self.cnt += 1;
        let byte_index = (self.cnt - 1) / BITS_PER_BYTE;
        let mut last_byte = self.b[byte_index];
        let bit_within_byte = (self.cnt - 1) % BITS_PER_BYTE;
        let shifted_bit = 1 << bit_within_byte;
        if bit {
            last_byte |= shifted_bit;      // set the bit
        } else {
            last_byte &= !shifted_bit;     // clear the bit
        }
        self.b[byte_index] = last_byte;  // and update last byte in array
    }

    fn null() -> Self {
        DynBitString { cnt: 0, b: Vec::new() }
    }
}


impl Clone for DynBitString {
    fn clone(&self) -> Self {
        let mut cln : DynBitString = DynBitString::null();
        cln.cnt = self.cnt;
        cln.b.clone_from(&self.b);
        cln
    }
}

impl PartialEq for DynBitString {
    fn eq(&self, other: &Self) -> bool {
        if self.cnt != other.cnt { false }
        else if self.cnt == 0 { true }
        else {
            for k in 0..self.cnt {
                if self.get(k) != other.get(k) { return false; }
            }
            true
        }
    }
}

// support "{:?}" when printing DynBitString
impl fmt::Debug for DynBitString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Display for DynBitString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s : String = "".to_string();
        for k in 0..self.len() {
            if self.get(k) { s.push('1'); } else { s.push('0'); }
        }
        write!(f, "b{}", s)
    }
}

// parse bitstring string same format as fmt::Debug above

impl FromStr for DynBitString {
    type Err = String;

    fn from_str(bitstr_binary_code : &str) -> Result<Self, Self::Err> {
        let bstr : String = bitstr_binary_code.to_string();
        if !bstr.starts_with('b') { Err("prefix for DynBitString not seen".to_string()) }
        else {
            let zeroes_and_ones  = bstr.strip_prefix('b').unwrap().as_bytes();
            let mut bitstr = DynBitString::null();
            for c in zeroes_and_ones {
                let char  = *c as char;
                if char == '1' {
                    bitstr.append(true);
                } else if char == '0' {
                    bitstr.append(false);
                } else {
                    return Err("bits in DynBitString must be 0 or 1".to_string());
                }
            }
            Ok(bitstr)
        }
    }
}

pub fn append_bits(dest : &mut DynBitString, bits: & DynBitString) {
    for k in 0..bits.len() {
        dest.append(bits.get(k));
    }
}

pub fn get_bits(bs : &DynBitString, starting_at : u32, count : u32) -> Result<DynBitString, DBSGetBitErr> {
    if starting_at >= bs.len() as u32 {
        return Err(DBSGetBitErr::StartingAtTooBig);
    } else if starting_at + count > bs.len() as u32 {
        return Err(DBSGetBitErr::CountPastEnd);
    }
    let mut substr = DynBitString::null();
    for k in 0..count {
        substr.append(bs.get((starting_at + k) as usize));
    }
    Ok(substr)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_null() {
        use super::DynBitString;
        use bitstring::BitString;
        let bs = DynBitString::null();
        assert_eq!(bs.len(), 0);
    }

    #[test]
    pub fn test_append() {
        use super::DynBitString;
        use bitstring::BitString;

        let mut bs = DynBitString::null();
        bs.append(true);
        assert_eq!(bs.len(), 1);
        assert!(bs.get(0));
        bs.append(false);
        assert_eq!(bs.len(), 2);
        assert!(!bs.get(1));
    }

    #[test]
    pub fn test_append_bits() {
        use super::DynBitString;
        use bitstring::BitString;

        let mut bs = DynBitString::null();
        bs.append(true);
        let mut bs2 = DynBitString::null();
        bs2.append(false);
        bs2.append(true);
        append_bits(&mut bs, &bs2);
        assert!(bs.get(0) && !bs.get(1) && bs.get(2));
    }

    #[test]
    pub fn test_setget() {
        use bitstring::BitString;

        let mut bs = DynBitString::null();
        bs.append(false);
        bs.set(0, true);
        assert!(bs.get(0));
        bs.set(0, false);
        assert!(!bs.get(0));
        bs.set(0, true);
        assert!(bs.get(0));
        bs.append(true);
        assert_eq!(bs.b[0], 3);
    }

    #[test]
    pub fn test_debug_fmt() {
        let bs = DynBitString::from_str("b011").unwrap();
        println!("debug format for empty string: {:?}", bs)
    }
    #[test]
    pub fn test_parse() {
        use bitstring::BitString;
        use std::str::FromStr;

        let bs_result1 = DynBitString::from_str("");
        match bs_result1 {
            Err(_) => { },
            Ok(_) => { assert!(false); }
        }
        let bs2 = DynBitString::from_str("b").unwrap();
        assert_eq!(bs2.len(), 0);
        let bs3 = DynBitString::from_str("b1").unwrap();
        assert!(bs3.len() == 1 && bs3.get(0));
        let bs4 = DynBitString::from_str("b01").unwrap();
        assert!(bs4.len() == 2 && !bs4.get(0) && bs4.get(1));
    }

    #[test]
    pub fn test_flip() {
        use bitstring::BitString;

        let mut bs = DynBitString::null();
        bs.append(false);
        bs.flip(0);
        assert!(bs.get(0));
        bs.flip(0);
        assert!(!bs.get(0));
    }

    #[test]
    pub fn test_clip() {
        use bitstring::BitString;

        let mut bs = DynBitString::null();
        for k in 0..9 {
            bs.append(k%2 == 1);
        }
        assert_eq!(bs.len(), 9);
        for k in 0..9 {
            assert_eq!(bs.get(k), k%2 == 1);
        }
        // case where bitstring shrinks
        bs.clip(2);
        assert_eq!(bs.len(), 2);
        // previously existing bits should be unchanged
        assert!(!bs.get(0));
        assert!(bs.get(1));

        // case where bitstring grows
        bs.clip(15);
        assert_eq!(bs.len(), 15);
        // check that the buffer length is right
        assert_eq!(bs.b.len(), 2);
        // previously existing bits should be unchanged
        assert!(!bs.get(0));
        assert!(bs.get(1));
        // newly allocated bits should be set to zero
        for k in 2..15 {
            assert!(!bs.get(k));
        }
    }

    #[test]
    pub fn test_get_bits() {
        let mut bs = DynBitString::null();
        bs.append(false); bs.append(true); bs.append(true);
        match get_bits(&bs, 2, 2) {
            Err(ecode) => { assert_eq!(ecode, DBSGetBitErr::CountPastEnd); },
            Ok(_) => {}
        }
        match get_bits(&bs, 3, 2) {
            Err(ecode) => { assert_eq!(ecode, DBSGetBitErr::StartingAtTooBig); },
            Ok(_) => {}
        }
        let substr = get_bits(&bs, 1, 2).unwrap();
        assert!(substr.get(0) && substr.get(1));
    }
}