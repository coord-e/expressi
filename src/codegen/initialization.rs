use failure::Error;

use super::error::LLVMError;
use inkwell::targets::{InitializationConfig, Target};

pub fn initialize_native() -> Result<(), Error> {
    Target::initialize_native(&InitializationConfig::default())
        .map_err(|message| LLVMError::TargetInitializationFailed { message }.into())
}

pub fn initialize_all() -> Result<(), Error> {
    Target::initialize_all(&InitializationConfig::default());
    Ok(())
}
