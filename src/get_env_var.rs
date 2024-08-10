// use environment variables as command-line interface

#[derive(Debug)]
#[derive(PartialEq)]
pub enum EnvVarFailure {
    VarNotFound,
    CouldNotParseVar
}

use std::str::FromStr;
use std::env;

pub fn get_env_var_u32(str_var_name : &str) -> Result<u32, EnvVarFailure> {
    let getenv_result = env::var(str_var_name);
    match getenv_result {
        Err(_) => {  // FIXME: assumes that only error ever returned is variable not found
            Err(EnvVarFailure::VarNotFound)
        }
        Ok(env_var_val) => {
            let result = u32::from_str(env_var_val.as_str());
            match result {
                Err(_) => {
                    if let Some(stripped_hex) = env_var_val.strip_prefix("0x") {
                        let result_hex = u32::from_str_radix(stripped_hex, 16);
                        match result_hex {
                            Err(_) => {
                                return Err(EnvVarFailure::CouldNotParseVar);
                            },
                            Ok(u32valhex) => {
                                return Ok(u32valhex);
                            }
                        }
                    }
                    Err(EnvVarFailure::CouldNotParseVar)
                }
                Ok(u32val) => {
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
            if e == EnvVarFailure::VarNotFound { Ok(default_value) }
            else { Err(e) }
        }
    }
}

pub fn get_env_var_bool(str_var_name : &str) -> Result<bool, EnvVarFailure> {
    let getenv_result = env::var(str_var_name);
    match getenv_result {
        Err(_) => {  // FIXME: assumes that only error ever returned is variable not found
            Err(EnvVarFailure::VarNotFound)
        }
        Ok(env_var_val) => {
            let result = bool::from_str(env_var_val.as_str());
            match result {
                Err(_) => Err(EnvVarFailure::CouldNotParseVar),
                Ok(bval) => Ok(bval)
            }
        }
    }
}

pub fn get_env_var_bool_with_default(str_var_name : &str, default_value : bool) -> Result<bool, EnvVarFailure> {
    match get_env_var_bool(str_var_name) {
        Ok(boolval) => Ok(boolval),
        Err(e) => {
            if e == EnvVarFailure::VarNotFound { Ok(default_value) }
            else { Err(e) }
        }
    }

}

// FIXME: do we need a "mock" capability to test this (exit call)?
pub fn env_var_usage( e : EnvVarFailure, var : &String ) {
    let s = match e {
        EnvVarFailure::VarNotFound =>  "environment variable not found" ,
        EnvVarFailure::CouldNotParseVar => "could not parse environment variable"
    };
    println!("ERROR: {} : {}", var, s);
    std::process::exit(1);
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_get_env_var_u32() {
        assert_eq!(get_env_var_u32("U32_VAL"), Ok(55));
        assert_eq!(get_env_var_u32("U32_VAL_UNPARSEABLE"), Err(EnvVarFailure::CouldNotParseVar));
        assert_eq!(get_env_var_u32("U32_VAL_NOT_THERE"), Err(EnvVarFailure::VarNotFound));
    }
    #[test]
    pub fn test_get_env_var_u32_with_default() {
        assert_eq!(get_env_var_u32_with_default("U32_VAL", 87), Ok(55));
        assert_eq!(get_env_var_u32_with_default("U32_VAL_NOT_THERE", 88), Ok(88));
    }
    #[test]
    pub fn test_get_env_var_bool() {
        assert_eq!(get_env_var_bool("BOOL_VAL_TRUE"), Ok(true));
        assert_eq!(get_env_var_bool("BOOL_VAL_FALSE"), Ok(false));
        assert_eq!(get_env_var_bool("BOOL_VAL_UNDEFINED"), Err(EnvVarFailure::VarNotFound));
        assert_eq!(get_env_var_bool("BOOL_VAL_INVALID"), Err(EnvVarFailure::CouldNotParseVar));
    }
    #[test]
    pub fn test_get_env_var_bool_with_default() {
        assert_eq!(get_env_var_bool_with_default("BOOL_VAL_TRUE", false), Ok(true));
        assert_eq!(get_env_var_bool_with_default("BOOL_VAL_UNDEFINED", true), Ok(true));
        assert_eq!(get_env_var_bool_with_default("BOOL_VAL_UNDEFINED", false), Ok(false));
    }
}