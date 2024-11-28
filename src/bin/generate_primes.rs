use compress_wi_primes::primes;
use std::time::SystemTime;

use compress_wi_primes::get_env_var::EnvVarFailure::VarNotFound;
use compress_wi_primes::get_env_var::{get_env_var_u32_with_default,get_env_var_bool_with_default,env_var_usage};
use compress_wi_primes::plot::plot_histogram_f64;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {

    // read in parameters from environment variables and syscalls

    let num_cores = num_cpus::get();
    let nthreads : usize = get_env_var_u32_with_default(
        "NTHREADS",
        num_cores as u32).unwrap() as usize;
    println!("number of cores to use: {}", nthreads);

    let largest_uint32: u32 = get_env_var_u32_with_default(
        "LARGEST_UINT",
        u32::MAX).unwrap();
    println!("largest prime number candidate: {}", largest_uint32);

    let time_before_primes = SystemTime::now();

    let mut prms : Vec<u32> = Vec::new();
    let read_result = primes::read_primes(largest_uint32);
    if let Ok(read_prms) = read_result {
        let time_after_read = SystemTime::now();
        let duration_read_primes = time_after_read.duration_since(time_before_primes)?;
        println!("time to read {} primes: {:?}", read_prms.len(), duration_read_primes);
        prms = read_prms;
        println!("primes up through {} were already generated!", largest_uint32);
    }
    if prms.is_empty() {  // if primes were not read from file
        prms = primes::parallel_calc_primes(nthreads, largest_uint32);

        let time_after_threads = SystemTime::now();
        let duration_after_threads = time_after_threads.duration_since(time_before_primes)?;
        println!("time to compute primes: {:?}", duration_after_threads);

        if let Err(e) = primes::write_primes(&prms, largest_uint32) {
            panic!("failed to write {} primes : {:?}", prms.len(), e);
        }

        let time_after_file = SystemTime::now();
        let duration_file_write = time_after_file.duration_since(time_after_threads)?;
        println!("time to write file: {:?}", duration_file_write);
    }

    let pics_env_var_name = "PRIME_INDEX_COMPRESSION_STATS".to_string();
    match get_env_var_bool_with_default(
        pics_env_var_name.as_str(),
        false) {
        Err(e) => { if e != VarNotFound { env_var_usage(e, &pics_env_var_name); }},
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

    Ok(())
}
