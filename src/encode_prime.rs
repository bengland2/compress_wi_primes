

// encode an unsigned integer already in factorized form
// encode factorization array into bitstring using variable-length encoding
// or decode bitstring into factorization array
// factorization array contains INDICES of primes not prime numbers
// see primes::indices_to_prime_factors to convert indices to prime numbers

use crate::dyn_bit_string::DynBitString;
use crate::encoding_small_int::SmallIntEncoding;
use crate::encoding_u32::U32Encoding;
use crate::encoding_uint_trait::EncodingUint;

#[derive(Debug)]
pub struct PrmPwr {
    pub exp : u8,          // exponent
    pub prm_idx : u32      // prime number zero-based index
}

#[derive(Debug)]
pub struct IntAsPrms {
    pub prm_powers : Vec<PrmPwr>
}

// input is a vector of non-decreasing prime number integers
// representing integer factorization into primes
// output is hopefully more compact representation of
// factors as powers of primes

pub fn factors_to_int_as_prms( prm_factors : &[u32] ) -> IntAsPrms {
    let mut iap = IntAsPrms { prm_powers: Vec::new() };
    let first_prmpwr = PrmPwr { exp: 0, prm_idx: prm_factors[0] };
    iap.prm_powers.push(first_prmpwr);
    let mut last_iap_index = iap.prm_powers.len() - 1;
    for p in prm_factors {
        let last_prmpwr = &mut iap.prm_powers[last_iap_index];
        if last_prmpwr.prm_idx == *p {
            last_prmpwr.exp += 1;
        } else {
            let next_prmpwr = PrmPwr { exp: 1, prm_idx: *p };
            iap.prm_powers.push(next_prmpwr);
            last_iap_index += 1;
        }
    }
    iap
}

// encode a IntAsPrms structure as a bit string using
// variable-length unsigned integer encoding
// v - prime number factorization,
//      each number must be prime number index
//      sequence must be of non-zero length and non-decreasing

pub fn encode_factors( v : &[u32] ) -> DynBitString {
    assert!(!v.is_empty());
    let iap = factors_to_int_as_prms(v);


    // append SmallIntEncoding containing
    // first encode the length of IntAsPrms
    // followed by each exponent

    let mut small_int_encoding = SmallIntEncoding::new();

    // the number of elements in the IntAsPrms structure
    // is encoded by subtracting 1 first, since there is no
    // reason to use a structure with zero elements

    let l = iap.prm_powers.len();
    assert!(l > 0);
    small_int_encoding.append_uint32(l as u32 - 1);

    for nxt_ppwr in iap.prm_powers.as_slice() {
        // there is no reason to include a prime
        // with an exponent of zero, which would just be
        // a factor 1 anyway
        // if the exponent was 1 then we would not be here.
        // so we can subtract 2 from the exponent to improve
        // compression
        assert!(nxt_ppwr.exp > 0);
        small_int_encoding.append_uint32((nxt_ppwr.exp - 1) as u32);
    }

    let encoding_so_far = small_int_encoding.get_bitstr_encoding();
    let mut index_encoding = U32Encoding::from_bitstr_encoding(encoding_so_far);
    let mut prev_index: u32 = 0;
    for nxt_ppwr in iap.prm_powers.as_slice() {
        // we encode the INDEX of the prime, because the
        // index of the prime will be significantly smaller than
        // the prime itself for large primes so this
        // may improve compression.

        // encode the difference between this index and the last index
        // to further shrink the size of the encoding.

        index_encoding.append_uint32(nxt_ppwr.prm_idx - prev_index);
        prev_index = nxt_ppwr.prm_idx;
    }
    index_encoding.get_bitstr_encoding()
}

// decode the bitstring into a factorization array
// output array is non-decreasing and contains INDICES of prime numbers

pub fn decode_factors( bs : &DynBitString ) -> Vec<u32> {
    let mut ppwrs : Vec<PrmPwr> = vec![];
    let mut exponents : Vec<u32> = vec![];
    let mut cursor : usize = 0;
    let small_int_encoding = SmallIntEncoding::from_bitstr_encoding(bs.clone());
    let mut prev_index : u32 = 0;
    let l = small_int_encoding.read_uint32(&mut cursor) + 1;
    ppwrs.reserve_exact(l as usize);
    exponents.reserve_exact(l as usize);
    for _k in 0..l {
        let next_exponent = small_int_encoding.read_uint32(&mut cursor) + 1;
        exponents.push(next_exponent);
    }
    let encoding_so_far = small_int_encoding.get_bitstr_encoding();
    let index_encoding = U32Encoding::from_bitstr_encoding(encoding_so_far);
    for k  in 0..l as usize {
        let next_prm_index = index_encoding.read_uint32(&mut cursor) + prev_index;
        prev_index = next_prm_index;
        let nxt_prime_power = PrmPwr { exp: exponents[k] as u8, prm_idx: next_prm_index };
        ppwrs.push(nxt_prime_power);
    }
    let mut factors : Vec<u32> = vec![];
    for ppwr in ppwrs {
        for _k in 0..ppwr.exp {
            factors.push(ppwr.prm_idx);
        }
    }
    factors
}

// format factorization encoding in a way that lets you see how
// effective/ineffective the encoding is for the components
// this implementation is closely tied to encode_factors()

pub fn format_factor_encoding_as_string( v : &[u32] ) -> String {
    let iap = factors_to_int_as_prms(v);
    let mut out_str = "".to_string();

    // append SmallIntEncoding containing
    // first encode the length of IntAsPrms
    // followed by each exponent


    // the number of elements in the IntAsPrms structure
    // is encoded by subtracting 1 first, since there is no
    // reason to use a structure with zero elements

    let l = iap.prm_powers.len();
    assert!(l > 0);
    let mut length_encoding = SmallIntEncoding::new();
    length_encoding.append_uint32(l as u32 - 1);
    let length_bs = length_encoding.get_bitstr_encoding();
    out_str += length_bs.to_string().as_str();
    out_str += " [ ";

    for nxt_ppwr in iap.prm_powers.as_slice() {
        // there is no reason to include a prime
        // with an exponent of zero, which would just be
        // a factor 1 anyway
        // if the exponent was 1 then we would not be here.
        // so we can subtract 2 from the exponent to improve
        // compression
        assert!(nxt_ppwr.exp > 0);
        let mut small_int_encoding = SmallIntEncoding::new();
        small_int_encoding.append_uint32((nxt_ppwr.exp - 1) as u32);
        out_str += small_int_encoding.get_bitstr_encoding().to_string().as_str();
        out_str += " ";
    }
    out_str += "] [ ";

    let mut prev_index: u32 = 0;
    for nxt_ppwr in iap.prm_powers.as_slice() {
        let mut index_encoding = U32Encoding::new();
        index_encoding.append_uint32(nxt_ppwr.prm_idx - prev_index);
        prev_index = nxt_ppwr.prm_idx;
        out_str += index_encoding.get_bitstr_encoding().to_string().as_str();
        out_str += " ";
    }
    out_str += " ] ";

    out_str
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use bitstring::BitString;

    #[allow(dead_code)]
    fn encode_it(n : u32, prms : & Vec<u32>) -> DynBitString {
        use crate::primes;

        let f = primes::factor(n, prms).unwrap();
        encode_factors(&f)
    }

    #[allow(dead_code)]
    fn encoded_int_as_str(bs : & DynBitString) -> String {
        let ixs = decode_factors(bs);
        let iap = factors_to_int_as_prms(&ixs);
        let lenstr = iap.prm_powers.len().to_string();
        let mut bstr = lenstr.to_string();
        bstr += "[";
        for prmpwr in iap.prm_powers {
            bstr += "(";
            bstr += prmpwr.prm_idx.to_string().as_str();
            bstr += "_";
            bstr += prmpwr.exp.to_string().as_str();
            bstr += ")";
        }
        bstr += "]";
        bstr
    }

    #[allow(dead_code)]
    fn prod( factors : Vec<u32> ) -> u32 {
        let mut product : u32 = 1;
        for next_factor in factors {
            product *= next_factor;
        }
        product
    }

    #[test]
    pub fn test_factors_to_int_as_prms()  {
        let f : Vec<u32>  = [2u32, 2u32, 3u32, 5u32].to_vec();
        let iap = factors_to_int_as_prms(&f);

        assert_eq!(iap.prm_powers.len(), 3);

        let a1 = &iap.prm_powers[0];
        assert_eq!(a1.exp, 2);
        assert_eq!(a1.prm_idx, 2);

        let a2 = &iap.prm_powers[1];
        assert_eq!(a2.exp, 1);
        assert_eq!(a2.prm_idx, 3);

        let a3 = &iap.prm_powers[2];
        assert_eq!(a3.prm_idx, 5);
        assert_eq!(a3.exp, 1);
    }

    #[test]
    pub fn test_encode_factors() {
        use crate::primes;

        let prms: Vec<u32> = primes::gen_primes_up_to(1 << 16);
        let mut two_to_the_k: u32 = 2;
        for _k in 1..15 {
            two_to_the_k *= 2;
            for j in 0..3 {
                let next_to_factor = two_to_the_k + j - 1;
                let bs = encode_it(next_to_factor, &prms);
                let f = primes::factor(next_to_factor, &prms);
                let encoded_str = format_factor_encoding_as_string(&f.unwrap().as_slice());
                // FIXME: get rid of println statements
                println!("bitstring for {} len {} bitstring {} binary {:?} formatted {}",
                         next_to_factor, bs.len(), encoded_int_as_str(&bs), bs, encoded_str);
            }
        }
    }
    #[test]
    pub fn test_decode_factors() {
        use crate::primes::gen_primes_up_to;

        let prms : Vec<u32> = gen_primes_up_to(1 << 16);

        for k in 1<<1..1<<15 {
            let bs = encode_it(k, &prms);
            let idx_out = decode_factors(&bs);
            let mut f_out: Vec<u32> = vec![];
            for idx in idx_out {
                f_out.push(prms[idx as usize]);
            }
            assert_eq!(prod(f_out), k);
        }
    }

    #[test]
    pub fn test_int_as_prm_to_string() {
        use crate::primes;

        let prms : Vec<u32> = primes::gen_primes_up_to(1 << 8);
        let bs = encode_it(30, &prms);
        println!("encoded_int_to_string {}", encoded_int_as_str(&bs));
    }
}