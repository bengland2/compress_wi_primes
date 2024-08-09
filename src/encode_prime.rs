

// encode integers using variable-length encoding
use crate::encoding_uint::*;
use crate::dyn_bit_string::DynBitString;
use bitstring::BitString;

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

// encode a IntAsPrms structure as a bitstring using
// variable-length unsigned integer encoding to compress
// the input array must be of non-zero length

pub fn encode_factors( v : &[u32] ) -> DynBitString {
    assert!(!v.is_empty());
    let iap = factors_to_int_as_prms( v );
    let mut encbuf  = DynBitString::null();
    let mut prev_index : u32 = 0;

    // the number of elements in the IntAsPrms structure
    // is encoded by subtracting 1 first, since there is no
    // reason to use a structure with zero elements

    let l = iap.prm_powers.len();
    assert!(l > 0);
    append_uint32_len(l as u32 - 1, & mut encbuf);

    for nxt_ppwr in iap.prm_powers {
        if nxt_ppwr.exp == 1 {
            encbuf.append(false);
        } else {
            encbuf.append(true);
            // there is no reason to include a prime
            // with an exponent of zero, which would just be
            // a factor 1 anyway
            // if the exponent was 1 then we would not be here.
            // so we can subtract 2 from the exponent to improve
            // compression
            assert!(nxt_ppwr.exp > 1);
            append_uint32((nxt_ppwr.exp-2) as u32, & mut encbuf);
        }

        // encode the INDEX of the prime, because the
        // index of the prime will be significantly smaller than
        // the prime itself for large primes so this
        // may improve compression.  Specifically density
        // of primes is approximately 1/ln(N)

        append_uint32(nxt_ppwr.prm_idx - prev_index, & mut encbuf);
        prev_index = nxt_ppwr.prm_idx;
    }
    encbuf
}

pub fn decode_factors( bs : &DynBitString, prms: &[u32] ) -> Vec<u32> {
    let mut ppwrs : Vec<PrmPwr> = vec![];
    let mut cursor : usize = 0;
    let mut prev_index : u32 = 0;
    let mut l = read_uint32_len(bs, &mut cursor) + 1;
    while l > 0 {
        l -= 1;
        let mut next_exponent = 1;  // if no exponent present, it is 1
        // read the first bit to see if an exponent is present
        let has_exponent = bs.get(cursor);
        cursor += 1;
        if has_exponent {
            next_exponent = read_uint32( bs, &mut cursor ) + 2;
            assert!(next_exponent <= u8::MAX as u32);
        }
        let next_prm_index = read_uint32( bs, &mut cursor) + prev_index;
        prev_index = next_prm_index;
        let nxt_prime_power = PrmPwr { exp: next_exponent as u8, prm_idx: prms[next_prm_index as usize] };
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

#[cfg(test)]
pub mod tests {

    use crate::dyn_bit_string::DynBitString;

    #[allow(dead_code)]
    fn encode_it(n : u32, prms : & Vec::<u32>) -> DynBitString {
        use crate::primes;

        let f = primes::factor(n, prms).unwrap();
        super::encode_factors(&f)
    }

    #[allow(dead_code)]
    fn encoded_int_as_str(bs : & DynBitString, prms : &[u32] ) -> String {
        let ixs = super::decode_factors(bs, prms);
        let iap = super::factors_to_int_as_prms(&ixs);
        let lenstr = iap.prm_powers.len().to_string();
        let mut bstr = lenstr.to_string();
        bstr += "[";
        for prmpwr in iap.prm_powers {
            bstr += "(";
            bstr += prmpwr.prm_idx.to_string().as_str();
            bstr += "_";
            if prmpwr.exp == 1 {
                bstr += "N"
            } else {
                bstr += (prmpwr.exp - 2).to_string().as_str();
            }
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
        use super::factors_to_int_as_prms;

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
        use bitstring::BitString;

        let prms : Vec<u32> = primes::gen_primes_up_to(1 << 16);
        let mut two_to_the_k : u32 = 2;
        for _k in 1..15 {
            two_to_the_k *= 2;
            for j in 0..3 {
                let next_to_factor = two_to_the_k+j-1;
                let bs = encode_it(next_to_factor, & prms);
                println!("bitstring for {} len {} bitstring {} binary {:?}", next_to_factor, bs.len(), encoded_int_as_str(&bs, &prms), bs);
            }
        }
    }

    #[test]
    pub fn test_decode_factors() {
        use crate::primes::gen_primes_up_to;

        let prms : Vec<u32> = gen_primes_up_to(1 << 16);

        for k in 1<<1..1<<15 {
            let bs = encode_it(k, &prms);
            let k_out = super::decode_factors(&bs, &prms);

            assert!(prod(k_out) == k);
        }
    }

    #[test]
    pub fn test_int_as_prm_to_string() {
        use crate::primes;

        let prms : Vec<u32> = primes::gen_primes_up_to(1 << 8);
        let bs = encode_it(30, &prms);
        println!("encoded_int_to_string {}", encoded_int_as_str(&bs, &prms));
    }
}