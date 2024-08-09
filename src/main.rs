
//#![allow(unused)]

use bitstring::BitString;
use rand::RngCore;
use crate::encode_prime::IntAsPrms;
use crate::get_env_var::EnvVarFailure::VarNotFound;
use crate::plot::{plot_histogram_u32, plot_histogram_f64};
use std::time::SystemTime;

pub mod encode_prime;
pub mod primes;
pub mod encoding_uint;
pub mod dyn_bit_string;
pub mod get_env_var;
pub mod plot;

fn hist_to_expected_value( hist : &[u32] ) -> f64 {
    let mut expected_value = 0.0;
    let mut sum = 0.0;
    for (k, v) in hist.iter().enumerate() {
        expected_value += (k as f64) * (*v as f64);
        sum += *v as f64;
    }
    expected_value /= sum;
    expected_value
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    use crate::get_env_var::{get_env_var_u32, get_env_var_u32_with_default};

    // read in parameters from environment variables and syscalls

    let num_cores = num_cpus::get();
    let nthreads : usize = get_env_var_u32_with_default(
        "NTHREADS",
        num_cores as u32).unwrap() as usize;

    let biggest_prime : u32 = get_env_var_u32_with_default(
        "LARGEST_UINT",
        u32::MAX).unwrap();

    let samples = get_env_var_u32_with_default(
        "SAMPLES",
        10000).unwrap();

    let time_before_primes = SystemTime::now();

    let mut prms : Vec<u32> = Vec::new();
    let read_result = primes::read_primes(biggest_prime);
    if let Ok(read_prms) = read_result {
        let time_after_read = SystemTime::now();
        let duration_read_primes = time_after_read.duration_since(time_before_primes)?;
        println!("time to read {} primes: {:?}", read_prms.len(), duration_read_primes);
        prms = read_prms;
    }
    if prms.is_empty() {  // if primes were not read from file
        prms = primes::parallel_calc_primes(nthreads, biggest_prime);

        let time_after_threads = SystemTime::now();
        let duration_after_threads = time_after_threads.duration_since(time_before_primes)?;
        println!("time to compute primes: {:?}", duration_after_threads);

        if let Err(e) = primes::write_primes(&prms, biggest_prime) {
            panic!("failed to write {} primes : {:?}", prms.len(), e);
        }

        let time_after_file = SystemTime::now();
        let duration_file_write = time_after_file.duration_since(time_after_threads)?;
        println!("time to write file: {:?}", duration_file_write);
    }

    // at this point, prms contains the primes we need to factor any u32
    // either we read it in from a file or we generated+wrote it to a file
    // so we only generate it if it isn't already saved
    // now we can experiment with it

    if let Ok(num_to_factor) = get_env_var_u32("NUM_TO_FACTOR") {
        println!("number to factor for debug: {}", num_to_factor);
        let f = primes::factor(num_to_factor, &prms).unwrap();
        println!("factor indexes of {} are {:?}", num_to_factor, f);
        let prmpwrs = encode_prime::factors_to_int_as_prms(&f);
        println!("prime powers of {} are {:?}", num_to_factor, prmpwrs);
        let bs = encode_prime::encode_factors(&f);
        println!("encoded value of {} bits is {:?}", bs.len(), bs);
        return Ok(());
    }

    if get_env_var_u32("TEST_FACTORING_ALL").is_ok() {
        println!("factoring all numbers up to {}", biggest_prime);
        let time_before_factoring = SystemTime::now();
        primes::parallel_factor_all(biggest_prime, nthreads, &prms);
        let time_after_factoring = SystemTime::now();
        let duration_factoring = time_after_factoring.duration_since(time_before_factoring)?;
        println!("factored all numbers in {:?}", duration_factoring);
    }

    let pics_env_var_name = "PRIME_INDEX_COMPRESSION_STATS".to_string();
    match get_env_var::get_env_var_bool_with_default(
        pics_env_var_name.as_str(),
        false) {
        Err(e) => { if e != VarNotFound { get_env_var::env_var_usage(e, &pics_env_var_name); }},
        Ok(calc_compression_stats) => {
            if calc_compression_stats {
                let mut prime_index_hist: Vec<f64> = vec![];
                primes::prime_index_ratio_hist(0, prms.len(), &prms, &mut prime_index_hist);
                println!("prime index compression histogram: {:?}", prime_index_hist);
                plot_histogram_f64(
                    "index_compression.png",
                    "prime index compression ratio",
                    "log base 2 of prime number",
                    "ratio of index to prime number",
                    &prime_index_hist)?;
            }
        }
    }
    // any compressed data will look like uniform random distribution
    // so we need to see prime-based compression work in this setting

    let mut compressions : u32 = 0;

    let interval_divisor = 10.0;
    let mut rng = rand::thread_rng();

    let mut histogrm_vs_u32 : Vec<u32> = vec![0; 100];
    let mut histogrm_fct_len : Vec<u32> = vec![0; 31]; // worst case is 2^31
    let mut histogrm_prmpwr_len : Vec<u32> = vec![0; 31]; // worst case is < factor array length
    let mut histogrm_exponent : Vec<u32> = vec![0; 31];
    let mut histogrm_log2_prime_index : Vec<u32> = vec![0; 31]; // worst case is 2^31

    for _j in 0..samples {
        let mut next_rand = rng.next_u32();
        if biggest_prime != u32::MAX { next_rand %= biggest_prime + 1 }
        if next_rand < 2 { next_rand = 2; }
        let ixs  = primes::factor(next_rand, &prms).unwrap();
        histogrm_fct_len[ixs.len()] += 1;

        let prmpwrs : IntAsPrms = encode_prime::factors_to_int_as_prms(&ixs);
        histogrm_prmpwr_len[prmpwrs.prm_powers.len()] += 1;
        for k in 0..prmpwrs.prm_powers.len() {
            histogrm_exponent[prmpwrs.prm_powers[k].exp as usize] += 1;
            let next_prime_index = prmpwrs.prm_powers[k].prm_idx;
            let log2_index = (next_prime_index as f64).log2() as u32;
            histogrm_log2_prime_index[log2_index as usize] += 1;
        }
        let e = encode_prime::encode_factors(&ixs);
        if (e.len() as u32) < u32::BITS {
            println!("COMPRESSED {} prime powers {:?} encoding {:?} len {}", next_rand, prmpwrs, e, e.len());
            compressions += 1;
        }

        let u32_szratio : f64 = e.len() as f64 / u32::BITS as f64;
        histogrm_vs_u32[(u32_szratio * interval_divisor) as usize] += 1;

        /* let f = primes::indices_to_prime_factors(&ixs, &prms);
        println!("int {} ind {:?} fct {:?} prmpwr {:?} buf {:?} buflen {} u32 ratio {}",
                 next_rand, ixs, f, prmpwrs, e, e.len(), u32_szratio); */
    }

    println!("compressions: {}", compressions);

    println!("histogram of encode_factors compression ratio: {:?}", histogrm_vs_u32);
    println!("expected value of compression ratio: {}", hist_to_expected_value(&histogrm_vs_u32)/interval_divisor);
    plot_histogram_u32(
        "encode_factors_compression.png",
        "encode_factors compression ratio",
        "compression ratio * 10 (< 10 is compression)",
        "frequency",
        &histogrm_vs_u32)?;

    println!("histogram of factor array lengths: {:?}", histogrm_fct_len);
    println!("expected value of factor array length: {}", hist_to_expected_value(&histogrm_fct_len));
    plot_histogram_u32(
        "factor_array_len.png",
        "prime factor array length",
        "array length",
        "frequency",
        &histogrm_fct_len)?;

    println!("histogram of prime power array lengths: {:?}", histogrm_prmpwr_len);
    println!("expected value of prime power array length: {}", hist_to_expected_value(&histogrm_prmpwr_len));
    plot_histogram_u32(
        "prime_power_array_length.png",
        "prime power array length",
        "array length",
        "frequency",
        &histogrm_fct_len)?;

    println!("histogram of exponents: {:?}", histogrm_exponent);
    println!("expected value of exponent: {}", hist_to_expected_value(&histogrm_exponent));
    plot_histogram_u32(
        "exponent.png",
        "distribution of exponent sizes",
        "exponent size",
        "frequency",
        &histogrm_fct_len)?;

    println!("histogram of log2 of prime index values: {:?}", histogrm_log2_prime_index);
    println!("expected value of log2 prime index: {}", hist_to_expected_value(&histogrm_log2_prime_index));
    plot_histogram_u32(
        "log2_prime_index_value.png",
        "distribution of prime indexes",
        "log2(prime_index)",
        "frequency",
        &histogrm_fct_len)?;

    Ok(())
}
