use bytes::Bytes;
use failure::Error;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;
use structopt::clap::{_clap_count_exprs, arg_enum};
use structopt::StructOpt;

use crate::compile;
use crate::error::{CLIError, LLVMError};
use crate::parser;
use crate::transform::TransformManager;
use crate::translator::translate_ast;

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

arg_enum! {
    enum OutputType {
        Object,
        Assembly,
        IR,
        EIR,
        AST,
    }
}

arg_enum! {
    enum OptimizationLevelOpt {
        None,
        Less,
        Default,
        Aggressive,
    }
}

impl Into<OptimizationLevel> for OptimizationLevelOpt {
    fn into(self) -> OptimizationLevel {
        match self {
            OptimizationLevelOpt::None => OptimizationLevel::None,
            OptimizationLevelOpt::Less => OptimizationLevel::Less,
            OptimizationLevelOpt::Default => OptimizationLevel::Default,
            OptimizationLevelOpt::Aggressive => OptimizationLevel::Aggressive,
        }
    }
}

arg_enum! {
    enum RelocModeOpt {
        Default,
        Static,
        PIC,
        DynamicNoPIC,
    }
}

impl Into<RelocMode> for RelocModeOpt {
    fn into(self) -> RelocMode {
        match self {
            RelocModeOpt::Default => RelocMode::Default,
            RelocModeOpt::Static => RelocMode::Static,
            RelocModeOpt::PIC => RelocMode::PIC,
            RelocModeOpt::DynamicNoPIC => RelocMode::DynamicNoPic,
        }
    }
}

#[derive(StructOpt)]
pub struct CodegenOpt {
    #[structopt(long = "triple")]
    target_triple: Option<String>,

    #[structopt(long = "cpu")]
    target_cpu: Option<String>,

    #[structopt(long = "cpu-features")]
    target_cpu_features: Option<String>,

    #[structopt(short = "O", long = "optimize", default_value = "default")]
    #[structopt(raw(
        possible_values = "&OptimizationLevelOpt::variants()",
        case_insensitive = "true"
    ))]
    optimization_level: OptimizationLevelOpt,

    #[structopt(long = "reloc", default_value = "default")]
    #[structopt(raw(
        possible_values = "&RelocModeOpt::variants()",
        case_insensitive = "true"
    ))]
    reloc_mode: RelocModeOpt,
}

#[derive(StructOpt)]
pub struct BuildOpt {
    #[structopt(name = "FILE", parse(from_os_str))]
    input: PathBuf,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,

    #[structopt(short = "t", long = "output-type", default_value = "object")]
    #[structopt(raw(possible_values = "&OutputType::variants()", case_insensitive = "true"))]
    output_type: OutputType,

    #[structopt(flatten)]
    codegen_opt: CodegenOpt,
}

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
            let result = compile::compile_string(contents, "main")?;
            result.llvm_ir().into()
        }
        OutputType::Assembly | OutputType::Object => {
            let result = compile::compile_string(contents, "main")?;

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
