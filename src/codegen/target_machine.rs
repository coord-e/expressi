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

#[cfg(test)]
mod tests {
    use super::super::initialization::{initialize_all, initialize_native};
    use super::create_target_machine;
    use inkwell::targets::{RelocMode, TargetMachine};
    use inkwell::OptimizationLevel;

    #[test]
    fn default_target_machine() {
        initialize_native().unwrap();

        let m = create_target_machine(
            None,
            None,
            None,
            OptimizationLevel::None,
            RelocMode::Default,
        )
        .unwrap();
        assert_eq!(m.get_triple(), TargetMachine::get_default_triple());
        assert_eq!(m.get_cpu(), TargetMachine::get_host_cpu_name());
        assert_eq!(
            m.get_feature_string().to_str().unwrap(),
            &TargetMachine::get_host_cpu_features().to_string()
        );
    }

    #[test]
    fn specify_target_machine() {
        initialize_all().unwrap();

        let triple = "armv7-pc-linux-gnu".to_string();
        let m = create_target_machine(
            Some(&triple),
            None,
            None,
            OptimizationLevel::None,
            RelocMode::Default,
        )
        .unwrap();
        assert_eq!(m.get_triple().to_string(), triple);
        assert_eq!(m.get_cpu().to_string(), "".to_string());
        assert_eq!(m.get_feature_string().to_str().unwrap(), "");
    }
}
