use bytes::Bytes;
use failure::Error;

use super::error::CLIError;
use super::opts::{BuildOpt, OutputType};
use crate::codegen::{compile, initialization, target_machine};
use crate::parser;
use crate::transform::TransformManager;
use crate::translator::translate_ast;

use std::fs::File;
use std::io::{Read, Write};

pub fn build(opt: &BuildOpt) -> Result<(), Error> {
    let BuildOpt {
        input,
        output,
        output_type,
        codegen_opt,
    } = opt;

    initialization::initialize_all()?;
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
            let result = compile::compile_string(contents, &codegen_opt.emit_func_name)?;
            result.llvm_ir().into()
        }
        OutputType::Assembly | OutputType::Object => {
            let result = compile::compile_string(contents, &codegen_opt.emit_func_name)?;

            let target_machine = target_machine::create_target_machine(
                codegen_opt.target_triple.as_ref(),
                codegen_opt.target_cpu.as_ref(),
                codegen_opt.target_cpu_features.as_ref(),
                codegen_opt.optimization_level.into(),
                codegen_opt.reloc_mode.into(),
            )?;

            match output_type {
                OutputType::Assembly => result.emit_assembly(&target_machine)?.into(),
                OutputType::Object => result.emit_object(&target_machine)?.into(),
                _ => unreachable!(),
            }
        }
    };
    let mut f = File::create(output)?;
    f.write_all(&buffer)?;
    Ok(())
}
