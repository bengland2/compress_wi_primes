use byteorder::ReadBytesExt;
use std::time::SystemTime;

const STARTER_PRIMES: [u32; 3] = [2, 3, 5];

// generate prime numbers in a range, given array of possible prime factors
// for numbers in that range.
// prms input array must contain at least all primes
// between 2 and sqrt(upper_bound)+1
// if one of these isn't a factor, then we know a number in range
// [lower_bound, upper_bound] is a prime.
// this approach lets us multi-thread prime generation
// because we can divide up the range of numbers being tested
// for primality into non-overlapping intervals where we can execute
// gen_primes_in_range() in parallel

#[derive(Debug)]
#[derive(PartialEq)]
pub enum GenPrimesErrcode {
    PrimesNotEnoughForRange
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum FactorPrimesErrcode {
    NotEnoughPrimesToFactorIt,  // we don't have large enough prime number array to prove it is prime
    NIsBigPrime,  // we proved N is prime but we cannot return its index in prime array
    AlgorithmFailed, // should never get here
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub struct PrimeComputeRange {
    lower: u32,
    upper: u32,
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Debug)]
pub struct PerThreadRanges {
    thread_index: u32,
    start_time: std::time::SystemTime,
    ranges: Vec<PrimeComputeRange>,
}


#[derive(Debug)]
#[derive(PartialEq)]
pub enum PrimeIndexError {
    NotInList
}

// return index of matching prime in primes list
pub fn index_in_prime_list(k: u32, prms: &[u32]) -> Result<u32, PrimeIndexError> {
    assert!(!prms.is_empty());
    match prms[..].binary_search(&k) {
        Ok(ix) => Ok(ix as u32),
        Err(_) => Err(PrimeIndexError::NotInList)
    }
}

// return true if n is in array of primes
pub fn is_prime(n: u32, prms: &[u32]) -> bool {
    index_in_prime_list(n, prms).is_ok()
}

// factor any positive integer > 1 into a list of non-decreasing prime indexes
// prms is an increasing array of primes, cannot be empty

pub fn factor(n: u32, prms: &[u32]) -> Result<Vec<u32>, FactorPrimesErrcode> {
    let last_prime = *prms.last().unwrap() as u64;
    if last_prime * last_prime < n as u64 {
        return Err(FactorPrimesErrcode::NotEnoughPrimesToFactorIt);
    }
    let mut next_prime_index_to_try: usize = 0;
    let mut num_to_factor = n;
    let mut factors: Vec<u32> = vec![];
    let prms_len = prms.len();
    while num_to_factor > 1 {
        if let Ok(i) = index_in_prime_list(num_to_factor, prms) {
            factors.push(i);
            return Ok(factors);
        }
        let mut next_factor_found = false;
        while next_prime_index_to_try < prms_len {
            next_factor_found = false;
            let p: u32 = prms[next_prime_index_to_try];
            if num_to_factor % p == 0 {
                factors.push(next_prime_index_to_try as u32);
                num_to_factor /= p;
                next_factor_found = true;
                next_prime_index_to_try = 0;
                break;
            }
            if p * p > n {
                break;
            }
            next_prime_index_to_try += 1;
        }
        if !next_factor_found {
            break;
        }
    }
    if (n as f64).sqrt() as u32 > *prms.last().unwrap() {
        return Err(FactorPrimesErrcode::NotEnoughPrimesToFactorIt);
    } else if factors.is_empty() {
        return Err(FactorPrimesErrcode::NIsBigPrime);
    } else if num_to_factor > 1 {
        return Err(FactorPrimesErrcode::AlgorithmFailed);
    }
    Ok(factors)
}

// convert indexes of prime numbers in prime number array into the primes
// represented by those indexes

pub fn indices_to_prime_factors(ixs: &Vec<u32>, prms: &[u32]) -> Vec<u32> {
    let mut f: Vec<u32> = Vec::with_capacity(ixs.len());
    for ix in ixs {
        f.push(prms[*ix as usize]);
    }
    f
}

/*
 * gen_primes_in_range - find all prime numbers in a specified range,
 *   returning them as array of prime numbers
 * old_prms     - prime numbers from 2 through primes_up_to
 * primes_up_to - old_prms contains all primes up to this value
 * lower_bound  - bottom of range in which we compute prime numbers
 * upper_bound  - top of range in which we compute prime numbers
 */
pub fn gen_primes_in_range(old_prms: &Vec<u32>, primes_up_to: u32, lower_bound: u32, upper_bound: u32) -> Result<Vec<u32>, GenPrimesErrcode>
{
    let primes_up_to_u64 = primes_up_to as u64;
    if primes_up_to_u64 * primes_up_to_u64 < upper_bound as u64 {
        Err(GenPrimesErrcode::PrimesNotEnoughForRange)
    } else {
        let mut candidate = lower_bound;
        if candidate % 2 == 0 { candidate += 1 };
        let mut new_prms: Vec<u32> = vec![];
        // since we will never test an even number, we can exclude 2 (prime index 0) in old_prms
        let old_prms_slice: &[u32] =
            if old_prms[0] == 2 {
                &old_prms[1..]
            } else {
                &old_prms[0..]
            };

        while candidate <= upper_bound {
            let mut factor_found = false;
            for prime_ref in old_prms_slice {
                if candidate % *prime_ref == 0 {
                    factor_found = true;
                    break;
                }
            }
            if !factor_found {
                new_prms.push(candidate);
            }
            // FIXME replace with a sieve algorithm for speed
            if candidate < u32::MAX - 1 {
                candidate += 2;  // excludes even numbers
            } else {
                break;  // before integer overflow occurs
            }
        }
        Ok(new_prms)
    }
}

pub fn gen_primes_up_to(n: u32) -> Vec<u32> {
    let mut prms = STARTER_PRIMES.to_vec();
    const U32_MAX_U64: u64 = u32::MAX as u64;
    loop {
        let last_prime = *prms.last().unwrap();
        let mut largest_factorable: u64 = last_prime as u64 * last_prime as u64;
        if largest_factorable > U32_MAX_U64 { largest_factorable = U32_MAX_U64 }
        let hi = if largest_factorable > n as u64 { n } else { largest_factorable as u32 };
        let mut prms2 = gen_primes_in_range(&prms, last_prime, last_prime + 1, hi).unwrap();
        prms.append(&mut prms2);
        if hi == n {
            break;  // this can only happen if there were no more primes between last prime and n
        }
    }
    prms
}

fn prime_data_pathname(last_prime: u32) -> String {
    use std::env;
    let tmpdir = env::var("TMPDIR").unwrap();
    tmpdir + "/primes_up_to_" + last_prime.to_string().as_str()
}

// write out array of primes to file, returning size of array in u32 words
pub fn write_primes(prms: &Vec<u32>, upper_bound: u32) -> Result<usize, std::io::Error> {
    use std::fs::File;
    use std::io::Write;

    let last_prime = *prms.last().unwrap();
    let fnstr = prime_data_pathname(upper_bound);
    println!("creating prime array file {} containing {} primes with last prime {}", &fnstr, prms.len(), last_prime);
    use std::io::BufWriter;
    match File::create(fnstr) {
        Ok(file_handle) => {
            let mut stream = BufWriter::new(file_handle);
            match prms.iter().try_for_each(|&x| stream.write_all(&x.to_be_bytes())) {
                Ok(_) => {
                    stream.flush().unwrap();
                    Ok(prms.len())
                }
                Err(e) => Err(e)
            }
            /* for p in prms {
                stream.write_all(&p.to_be_bytes()).unwrap();
            } */
        }
        Err(e) => {
            panic!("could not create prime file : {:?}", e);
        }
    }
}

// FIXME: fast way to load a u32 array into memory from a file
pub fn read_primes(upper_bound: u32) -> Result<Vec<u32>, std::io::Error> {
    use std::fs::File;
    use std::io::BufReader;
    use byteorder::BigEndian;

    let fnstr = prime_data_pathname(upper_bound);
    match File::open(fnstr.clone()) {
        Ok(file_handle) => {
            const BYTES_PER_U32: u32 = 4;
            let fsz = file_handle.metadata().unwrap().len();
            let mut stream = BufReader::new(file_handle);
            let prime_count: usize = (fsz as u32 / BYTES_PER_U32) as usize;
            let mut prms: Vec<u32> = Vec::with_capacity(prime_count);
            prms.resize(prime_count, 0);
            match stream.read_u32_into::<BigEndian>(prms.as_mut_slice()) {
                Ok(_) => Ok(prms),
                Err(e) => Err(e)
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
}

// lower priority so massive thread use doesn't lock up laptop
fn lower_priority() {
    assert!(thread_priority::set_current_thread_priority(thread_priority::ThreadPriority::Min).is_ok());
}

// break up prime search into chunks for multiple threads to work on

pub fn shard_prime_calc(chunks: usize, prime_upper_bound: u32, prime_lower_bound: u32) -> Vec<PrimeComputeRange> {
    let mut child_state: Vec<PrimeComputeRange> = Vec::with_capacity(chunks);
    let total_range = (prime_upper_bound - prime_lower_bound) as f64;
    for i in 0..chunks {
        let rng_lower = prime_lower_bound as f64 + (i as f64 * total_range) / chunks as f64;
        let next_rng_lower: f64 = prime_lower_bound as f64 + ((i + 1) as f64 * total_range) / chunks as f64;
        let rng_upper = if i == chunks - 1 {
            prime_upper_bound
        } else {
            next_rng_lower as u32 - 1
        };
        let next_child_state: PrimeComputeRange = PrimeComputeRange {
            upper: rng_upper,
            lower: rng_lower as u32
        };
        child_state.push(next_child_state);
        //println!("chunk {} lower {} upper {}", i, child_state[i].lower, child_state[i].upper);
    }
    child_state
}


// use multithreading to calculate prime numbers up to 2^32 much faster

pub fn parallel_calc_primes(nthreads: usize, highest_candidate: u32) -> Vec<u32> {
    use std::sync::mpsc;
    use std::thread;

    const MIN_PER_THREAD_RANGE_COUNT: usize = 10; // FIXME: this value is for debugging
    const NORMAL_RANGE_SIZE: usize = 1000000;  // FIXME: this value is for debugging

    let prime_base_range = ((highest_candidate as f64).sqrt() + 1.0) as u32;
    let base_prms = gen_primes_up_to(prime_base_range);
    let last_base_prm = *base_prms.last().unwrap();
    let per_thread_range_count = MIN_PER_THREAD_RANGE_COUNT.max((highest_candidate - last_base_prm) as usize / (nthreads * NORMAL_RANGE_SIZE));
    if per_thread_range_count == 0 {
        panic!("unable to shard prime number generation, check environment variables NTHREADS and LARGEST_UINT");
    }
    let mut prms = base_prms.clone();

    println!("parallel_calc_primes: nthreads {} base prms len {} first {} last {}",
             nthreads, base_prms.len(), base_prms[0], last_base_prm);

    let small_ranges = shard_prime_calc(
        nthreads * per_thread_range_count,
        highest_candidate,
        prime_base_range + 1);

    let mut per_thread_ranges: Vec<PerThreadRanges> = vec![];
    let mut candidate_count: u32 = 0;
    for t in 0..nthreads {
        let mut per_thread_range = PerThreadRanges {
            thread_index: t as u32,
            start_time: SystemTime::now(),
            ranges: vec![],
        };
        for k in 0..per_thread_range_count {
            let next_range_index = t + (nthreads * k);
            let next_small_range = small_ranges[next_range_index];
            per_thread_range.ranges.push(next_small_range);
            candidate_count += (next_small_range.upper - next_small_range.lower) + 1;
        }
        per_thread_ranges.push(per_thread_range);
    }
    assert!(candidate_count == highest_candidate - prime_base_range);

    // create channel for each thread to send its primes back to this thread
    // we will clone the transmit side of channel for each thread

    let mut receive_ends: Vec<mpsc::Receiver<Vec<u32>>> = vec![];

    // launch threads to split up calculation of primes

    thread::scope(|s| {
        let mut children = vec![];
        for (i, per_thread_range_set) in per_thread_ranges.iter().enumerate() {

            // build transmitter-receiver pair for passing back array of primes from each thread

            let (next_tx, next_rx) = mpsc::channel();
            let tx1 = next_tx.clone(); // tx1 will be owned by child thread
            receive_ends.push(next_rx);  // save receiver for later

            // Spin up another thread

            let prms_clone = base_prms.clone();  // prms here is small so no cost to doing
            let ranges = per_thread_range_set.ranges.clone();
            let thrd_spawn_result = s.spawn(move || {
                // lower priority so massive thread use doesn't lock up laptop
                lower_priority();
                let before = SystemTime::now();
                for chunk in ranges {
                    let lower = chunk.lower;
                    let upper = chunk.upper;
                    let before_gen_chunk = SystemTime::now();
                    println!("thread {} time since start {:?} gen_primes_in_range base {} lower {} upper {}",
                            i, SystemTime::now().duration_since(before).unwrap(),
                             prime_base_range, lower, upper);
                    let gen_result = gen_primes_in_range(&prms_clone, prime_base_range, lower, upper);
                    let after = SystemTime::now();
                    let duration = after.duration_since(before_gen_chunk).unwrap();
                    println!("for thread {}, duration of gen_primes_in_range = {:?}", i, duration);
                    match gen_result {
                        Ok(chunk_prms) => {
                            match tx1.send(chunk_prms) {
                                Ok(_) => {}
                                Err(e) => { panic!("channel send: {:?}", e); }
                            }
                        }
                        Err(e) => {
                            panic!("for thread {}, error in chunk {:?}: {:?}",
                                   per_thread_range_set.thread_index,
                                   chunk,
                                   e);
                        }
                    }
                }
            });
            children.push(thrd_spawn_result);
        }
        assert!(children.len() == nthreads);

        // collect vectors of primes from all threads

        for k in 0..per_thread_range_count {
            for t in 0..nthreads {
                let next_receiver = &receive_ends[t];
                let rcv_max_duration = std::time::Duration::from_secs(100);
                let chunk_rcv_result = next_receiver.recv_timeout(rcv_max_duration);
                if let Err(e) = chunk_rcv_result {
                    panic!("unable to receive prime chunk {} from thread {} : {:?}", k, t, e);
                }
                let mut rcvd_prms = chunk_rcv_result.unwrap();
                let highest_prm = if k > 0 || t > 0 { *base_prms.last().unwrap() } else { 0 };
                assert!(highest_prm < rcvd_prms[0]);
                prms.append(&mut rcvd_prms);
            }
        }
        // wait for threads to shut down

        for next_child in children {
            // very important to return primes from each child in ORDER
            // with just one receiver, they come back in random order
            next_child.join().unwrap();
        }
    }); // end thread::scope
    prms
}

// verify that factoring algorithm works for every number in an interval

fn test_factors_in_range(thread_id: String, lo: u32, hi: u32, prms: &[u32]) -> u32 {
    for k in lo..hi {
        if k % 10000000 == 0 {
            let pct_left = 100.0 * (hi - k) as f32 / (hi - lo) as f32;
            println!("thread ID {} factoring {} % left {} ", thread_id, k, pct_left);
        }
        let r = factor(k, prms);
        match r {
            Ok(ixs) => {
                let f = indices_to_prime_factors(&ixs, prms);
                let mut prod: u32 = 1;
                for p in f {
                    prod *= p;
                    match index_in_prime_list(p, prms) {
                        Ok(ix) => { assert!(prms[ix as usize] == p) }
                        Err(e) => {
                            assert!(e == PrimeIndexError::NotInList);
                            println!("prime {} in factor array not found in prms", p);
                            return k;
                        }
                    }
                }
                assert!(prod == k);
            }
            Err(e) => {
                println!("ERROR factoring {} : {:?}", k, e);
                return k;
            }
        }
    }
    hi
}

// use multithreading to speed up testing of factoring algorithm
pub fn parallel_factor_all(biggest_number: u32, nthreads: usize, prms: &[u32]) {
    use std::thread;

    let factor_threads = nthreads;
    let child_range = shard_prime_calc(factor_threads, biggest_number, 2);
    thread::scope(|s| {
        let mut children = vec![];
        for (i, next_range) in child_range.iter().enumerate().take(factor_threads) {
            // Spin up another thread
            let lower = next_range.lower;
            let upper = next_range.upper;
            let thrd_spawn_result = s.spawn(move || {
                lower_priority();
                test_factors_in_range(i.to_string(), lower, upper, prms)
            });
            children.push(thrd_spawn_result);
        }

        // wait for threads to finish generating primes

        for (k, next_child) in children.into_iter().enumerate() {
            let result: u32 = next_child.join().unwrap();
            println!("thread {} low {} hi {} result {}", k, child_range[k].lower, child_range[k].upper, result);
            assert!(result == child_range[k].upper);
        }
    });
}

// calculate compression inherent in using index to represent prime number
// as a function of the prime number's size (log2)
// to do this, we can shard the range of prime numbers and multi-thread the calculation
// if necessary

pub fn prime_index_ratio_hist(prm_idx_lo: usize, prm_idx_hi: usize, prms: &[u32], hist: &mut Vec<f64>) {
    hist.resize(32, 0.0);
    let mut pcount: Vec<u32> = vec![0; 32];
    for (k, prm) in prms.iter().enumerate().take(prm_idx_hi).skip(prm_idx_lo) {
        let p = *prm as f64;
        let r = k as f64 / p;
        let idx = p.log2() as usize;
        hist[idx] += r;
        pcount[idx] += 1;
    }
    for k in 0..hist.len() {
        if pcount[k] != 0 && hist[k] != 0.0 {
            hist[k] /= pcount[k] as f64;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub const PRIMES_UP_TO_271: [u32; 58] =
        [2, 3, 5, 7, 11, 13, 17, 19, 23, 29,
            31, 37, 41, 43, 47, 53, 59, 61, 67, 71,
            73, 79, 83, 89, 97, 101, 103, 107, 109, 113,
            127, 131, 137, 139, 149, 151, 157, 163, 167, 173,
            179, 181, 191, 193, 197, 199, 211, 223, 227, 229,
            233, 239, 241, 251, 257, 263, 269, 271];

    #[test]
    pub fn test_index_in_prime_list() {
        let test_prms: Vec<u32> = STARTER_PRIMES.to_vec();
        assert_eq!(index_in_prime_list(5, &test_prms), Ok(2));
        assert_eq!(index_in_prime_list(4, &test_prms), Err(PrimeIndexError::NotInList));
        assert_eq!(index_in_prime_list(1, &test_prms), Err(PrimeIndexError::NotInList));
    }

    #[test]
    pub fn test_is_prime() {
        let test_prms: Vec<u32> = STARTER_PRIMES.to_vec();
        assert!(is_prime(2, &test_prms));
        assert!(!is_prime(4, &test_prms));
    }

    #[test]
    pub fn test_indices_to_prime_factors() {
        let test_prms: Vec<u32> = PRIMES_UP_TO_271.to_vec();
        let f = [0, 1, 2].to_vec();
        let fct = indices_to_prime_factors(&f, &test_prms);
        assert!(fct.len() == 3 && fct[0] == 2 && fct[1] == 3 && fct[2] == 5);
    }

    #[test]
    pub fn test_factors() {
        use crate::primes::FactorPrimesErrcode::*;

        let prms: Vec<u32> = PRIMES_UP_TO_271.to_vec();

        // must have enough primes so that sqrt(n) <= largest prime

        let too_big_to_factor = 271 * 271 + 1;
        let rslt = factor(too_big_to_factor, &prms);
        match rslt {
            Ok(_) => { assert!(false); }
            Err(e) => { assert!(e == NotEnoughPrimesToFactorIt); }
        };

        // n could be prime but larger than any in prms
        let big_prime = 23;
        let small_prime_list = STARTER_PRIMES.to_vec();
        let rslt2 = factor(big_prime, &small_prime_list);
        match rslt2 {
            Ok(_) => { assert!(false); }
            Err(e) => { assert!(e == NIsBigPrime); }
        };

        // check it for primes that we know of already

        for i in 2..272 {
            let f = factor(i, &prms).unwrap();
            println!("{} factors {:?}", i, f);
            let mut prod: u32 = 1;
            let mut last_val: u32 = 0;
            // test that elements are non-decreasing and prime
            // test that product of these elements is the number being factored, i
            for k in f {
                let next_prime = prms[k as usize];

                prod *= next_prime;

                let factors_of_prime = factor(next_prime, &prms).unwrap();
                assert!(factors_of_prime.len() == 1);
                assert!(factors_of_prime[0] == k);

                assert!(last_val <= k);
                last_val = k;
            }
            assert!(prod == i);
        }
    }

    #[test]
    pub fn test_gen_primes_in_range() {
        let mut old_prms: Vec<u32> = PRIMES_UP_TO_271.to_vec();
        let gen_primes_result = gen_primes_in_range(&old_prms, 271, 1000000, 2000000);
        match gen_primes_result {
            Err(GenPrimesErrcode::PrimesNotEnoughForRange) => {}
            Ok(_) => { assert!(false); }
        }
        let lower_bound: u32 = 273;
        let upper_bound: u32 = 1000;
        let mut new_prms = gen_primes_in_range(&old_prms, 271, lower_bound, upper_bound).unwrap();
        let mut old_and_new_prms: Vec<u32> = Vec::new();
        old_and_new_prms.append(&mut old_prms);
        println!("primes in range [{}, {}] = {:?}", lower_bound, upper_bound, new_prms);
        old_and_new_prms.append(&mut new_prms);
        let nonmut_old_and_new = &old_and_new_prms;

        for j in 0..old_and_new_prms.len() {
            let next_primes_factors = factor(old_and_new_prms[j], nonmut_old_and_new).unwrap();
            assert!(next_primes_factors.len() == 1 && next_primes_factors[0] == j as u32);
        }
    }

    pub fn test_gen_primes_up_to() {
        let prms_up_to_271 = gen_primes_up_to(271);
        assert!(prms_up_to_271 == PRIMES_UP_TO_271);
        let prms_up_to_10000 = gen_primes_up_to(10000);
        let mut last_k: u32 = 0;
        for k in &prms_up_to_10000 {
            assert!(*k > last_k);
            last_k = *k;
            assert!(is_prime(*k, &prms_up_to_10000));
        }
        println!("last prime before 10000 is {}", last_k);
    }


    #[test]
    pub fn test_write_primes() {
        write_primes(&PRIMES_UP_TO_271.to_vec(), 271).unwrap();
    }

    #[test]
    pub fn test_read_primes() {
        test_write_primes();
        let primes_we_read = read_primes(271).unwrap();
        assert!(primes_we_read == PRIMES_UP_TO_271.to_vec());
    }
}
