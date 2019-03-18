use inkwell::targets::RelocMode;
use inkwell::OptimizationLevel;
use structopt::clap::{_clap_count_exprs, arg_enum};
use structopt::StructOpt;

use std::path::PathBuf;

#[derive(StructOpt)]
pub struct RunOpt {
    #[structopt(name = "FILE", parse(from_os_str))]
    pub input: Option<PathBuf>,

    #[structopt(long = "print-ast")]
    pub print_ast: bool,

    #[structopt(long = "print-eir")]
    pub print_eir: bool,

    #[structopt(long = "print-ir")]
    pub print_ir: bool,
}

arg_enum! {
    #[derive(Copy, Clone)]
    pub enum OutputType {
        Object,
        Assembly,
        IR,
        EIR,
        AST,
    }
}

arg_enum! {
    #[derive(Copy, Clone)]
    pub enum OptimizationLevelOpt {
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
    #[derive(Copy, Clone)]
    pub enum RelocModeOpt {
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
    pub target_triple: Option<String>,

    #[structopt(long = "cpu")]
    pub target_cpu: Option<String>,

    #[structopt(long = "cpu-features")]
    pub target_cpu_features: Option<String>,

    #[structopt(short = "O", long = "optimize", default_value = "default")]
    #[structopt(raw(
        possible_values = "&OptimizationLevelOpt::variants()",
        case_insensitive = "true"
    ))]
    pub optimization_level: OptimizationLevelOpt,

    #[structopt(long = "reloc", default_value = "default")]
    #[structopt(raw(
        possible_values = "&RelocModeOpt::variants()",
        case_insensitive = "true"
    ))]
    pub reloc_mode: RelocModeOpt,

    #[structopt(long = "emit-func-name", default_value = "main")]
    pub emit_func_name: String,
}

#[derive(StructOpt)]
pub struct BuildOpt {
    #[structopt(name = "FILE", parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    pub output: PathBuf,

    #[structopt(short = "t", long = "output-type", default_value = "object")]
    #[structopt(raw(possible_values = "&OutputType::variants()", case_insensitive = "true"))]
    pub output_type: OutputType,

    #[structopt(flatten)]
    pub codegen_opt: CodegenOpt,
}
