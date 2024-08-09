// use environment variables as command-line interface

use crate::get_env_var::EnvVarFailure::{CouldNotParseVar, VarNotFound};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum EnvVarFailure {
    VarNotFound,
    CouldNotParseVar
}

pub fn get_env_var_u32(str_var_name : &str) -> Result<u32, EnvVarFailure> {
    use std::env;
    use std::str::FromStr;

    let getenv_result = env::var(str_var_name);
    match getenv_result {
        Err(_) => {  // FIXME: assumes that only error ever returned is variable not found
            Err(VarNotFound)
        }
        Ok(env_var_val) => {
            let result = u32::from_str(env_var_val.as_str());
            match result {
                Err(_) => {
                    if let Some(stripped_hex) = env_var_val.strip_prefix("0x") {
                        let result_hex = u32::from_str_radix(stripped_hex, 16);
                        match result_hex {
                            Err(_) => {
                                return Err(CouldNotParseVar);
                            },
                            Ok(u32valhex) => {
                                println!("env.var. {} = {}", str_var_name, u32valhex);
                                return Ok(u32valhex);
                            }
                        }
                    }
                    Err(CouldNotParseVar)
                }
                Ok(u32val) => {
                    println!("env.var. {} = {}", str_var_name, u32val);
                    Ok(u32val)
                }
            }
        },
    }
}

pub fn get_env_var_u32_with_default(str_var_name : &str, default_value : u32) -> Result<u32, EnvVarFailure> {
    match get_env_var_u32(str_var_name) {
        Ok(u32val) => Ok(u32val),
        Err(e) => {
            if e == VarNotFound { Ok(default_value) }
            else { Err(e) }
        }
    }
}

pub fn get_env_var_bool(str_var_name : &str) -> Result<bool, EnvVarFailure> {
    use std::env;
    use std::str::FromStr;

    let getenv_result = env::var(str_var_name);
    match getenv_result {
        Err(_) => {  // FIXME: assumes that only error ever returned is variable not found
            Err(VarNotFound)
        }
        Ok(env_var_val) => {
            let result = bool::from_str(env_var_val.as_str());
            match result {
                Err(_) => Err(CouldNotParseVar),
                Ok(bval) => Ok(bval)
            }
        }
    }
}

pub fn get_env_var_bool_with_default(str_var_name : &str, default_value : bool) -> Result<bool, EnvVarFailure> {
    match get_env_var_bool(str_var_name) {
        Ok(boolval) => Ok(boolval),
        Err(e) => {
            if e == VarNotFound { Ok(default_value) }
            else { Err(e) }
        }
    }

}

pub fn env_var_usage( e : EnvVarFailure, var : &String ) {
    let s = match e {
        VarNotFound =>  "environment variable not found" ,
        CouldNotParseVar => "could not parse environment variable"
    };
    println!("ERROR: {} : {}", var, s);
    std::process::exit(1);
}
