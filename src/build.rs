use failure::Error;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;
use structopt::StructOpt;

use crate::compile;
use crate::error::{CLIError, LLVMError};

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(StructOpt)]
pub struct BuildOpt {
    #[structopt(name = "FILE", parse(from_os_str))]
    input: PathBuf,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,
}

pub fn build(opt: BuildOpt) -> Result<(), Error> {
    let BuildOpt { input, output } = opt;

    Target::initialize_all(&InitializationConfig::default());
    let mut f = File::open(&input).map_err(|_| CLIError::NotFound {
        path: input.clone(),
    })?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|error| CLIError::IOError { error })?;

    let result = compile::compile_string(contents.trim(), "main")?;
    let triple = TargetMachine::get_default_triple().to_string();
    let target =
        Target::from_triple(&triple).map_err(|message| LLVMError::TargetInitializationFailed {
            message: message.to_string(),
        })?;
    let target_machine = target
        .create_target_machine(
            &triple,
            &TargetMachine::get_host_cpu_name().to_string(),
            &TargetMachine::get_host_cpu_features().to_string(),
            OptimizationLevel::None,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| LLVMError::TargetInitializationFailed {
            message: "Failed to create TargetMachine'".to_string(),
        })?;
    let memory_buffer = target_machine
        .write_to_memory_buffer(result.module(), FileType::Object)
        .unwrap();
    let mut f = File::create(output)?;
    f.write_all(memory_buffer.as_slice())?;
    Ok(())
}
