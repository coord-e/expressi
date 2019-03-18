use failure::Error;
use inkwell::targets::{CodeModel, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;

use super::error::LLVMError;

pub fn create_target_machine(
    target_triple: Option<&String>,
    target_cpu: Option<&String>,
    target_cpu_features: Option<&String>,
    optimization_level: OptimizationLevel,
    reloc_mode: RelocMode,
) -> Result<TargetMachine, Error> {
    let (triple, default_cpu, default_features) = if let Some(triple) = target_triple {
        (triple.clone(), String::new(), String::new())
    } else {
        (
            TargetMachine::get_default_triple().to_string(),
            TargetMachine::get_host_cpu_name().to_string(),
            TargetMachine::get_host_cpu_features().to_string(),
        )
    };

    let cpu = target_cpu.unwrap_or(&default_cpu);
    let cpu_features = target_cpu_features.unwrap_or(&default_features);
    let target =
        Target::from_triple(&triple).map_err(|message| LLVMError::TargetInitializationFailed {
            message: message.to_string(),
        })?;
    target
        .create_target_machine(
            &triple,
            &cpu,
            &cpu_features,
            optimization_level,
            reloc_mode,
            CodeModel::Default,
        )
        .ok_or_else(|| {
            LLVMError::TargetInitializationFailed {
                message: "Failed to create TargetMachine'".to_string(),
            }
            .into()
        })
}
