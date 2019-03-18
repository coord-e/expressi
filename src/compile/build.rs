use bytes::Bytes;
use failure::Error;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, Target, TargetMachine};

use super::llvm;
use super::opts::{BuildOpt, OutputType};
use crate::error::{CLIError, LLVMError};
use crate::parser;
use crate::transform::TransformManager;
use crate::translator::translate_ast;

use std::fs::File;
use std::io::{Read, Write};

pub fn build(opt: BuildOpt) -> Result<(), Error> {
    let BuildOpt {
        input,
        output,
        output_type,
        codegen_opt,
    } = opt;

    Target::initialize_all(&InitializationConfig::default());
    let mut f = File::open(&input).map_err(|_| CLIError::NotFound {
        path: input.clone(),
    })?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .map_err(|error| CLIError::IOError { error })?;
    let contents = contents.trim();

    let buffer: Bytes = match output_type {
        OutputType::AST => format!("{:#?}", parser::parse(contents)?).into(),
        OutputType::EIR => {
            let ast = parser::parse(contents)?;
            let eir = TransformManager::default().apply(translate_ast(ast)?)?;
            format!("{}", eir).into()
        }
        OutputType::IR => {
            let result = llvm::compile_string(contents, &codegen_opt.emit_func_name)?;
            result.llvm_ir().into()
        }
        OutputType::Assembly | OutputType::Object => {
            let result = llvm::compile_string(contents, &codegen_opt.emit_func_name)?;

            let (triple, default_cpu, default_features) =
                if let Some(triple) = codegen_opt.target_triple {
                    (triple, String::new(), String::new())
                } else {
                    (
                        TargetMachine::get_default_triple().to_string(),
                        TargetMachine::get_host_cpu_name().to_string(),
                        TargetMachine::get_host_cpu_features().to_string(),
                    )
                };

            let cpu = codegen_opt.target_cpu.unwrap_or(default_cpu);
            let cpu_features = codegen_opt.target_cpu_features.unwrap_or(default_features);
            let target = Target::from_triple(&triple).map_err(|message| {
                LLVMError::TargetInitializationFailed {
                    message: message.to_string(),
                }
            })?;
            let target_machine = target
                .create_target_machine(
                    &triple,
                    &cpu,
                    &cpu_features,
                    codegen_opt.optimization_level.into(),
                    codegen_opt.reloc_mode.into(),
                    CodeModel::Default,
                )
                .ok_or_else(|| LLVMError::TargetInitializationFailed {
                    message: "Failed to create TargetMachine'".to_string(),
                })?;
            let filetype = match output_type {
                OutputType::Assembly => FileType::Assembly,
                OutputType::Object => FileType::Object,
                _ => unreachable!(),
            };
            let memory_buffer = target_machine
                .write_to_memory_buffer(result.module(), filetype)
                .unwrap();
            memory_buffer.as_slice().into()
        }
    };
    let mut f = File::create(output)?;
    f.write_all(&buffer)?;
    Ok(())
}
