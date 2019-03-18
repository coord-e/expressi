use super::error::LLVMError;

use failure::Error;

use inkwell::execution_engine::JitFunction;
use inkwell::targets::{FileType, TargetMachine};
use inkwell::{module, OptimizationLevel};

pub struct CompilationResult {
    module: module::Module,
}

impl CompilationResult {
    pub fn new(module: module::Module) -> Self {
        CompilationResult { module }
    }

    pub fn module(&self) -> &module::Module {
        &self.module
    }

    pub fn llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn verify(&self) -> Result<(), Error> {
        if let Err(message) = self.module.verify() {
            return Err(LLVMError::ModuleVerificationError {
                message: message.to_string(),
            }
            .into());
        }
        Ok(())
    }

    pub fn emit_assembly(&self, target_machine: &TargetMachine) -> Result<String, Error> {
        target_machine
            .write_to_memory_buffer(self.module(), FileType::Assembly)
            .map_err(|message| {
                LLVMError::MemoryBufferError {
                    message: message.to_string(),
                }
                .into()
            })
            .and_then(|buffer| String::from_utf8(buffer.as_slice().to_vec()).map_err(Into::into))
    }

    pub fn emit_object(&self, target_machine: &TargetMachine) -> Result<Vec<u8>, Error> {
        target_machine
            .write_to_memory_buffer(self.module(), FileType::Object)
            .map_err(|message| {
                LLVMError::MemoryBufferError {
                    message: message.to_string(),
                }
                .into()
            })
            .map(|buffer| buffer.as_slice().to_vec())
    }

    pub fn emit_function(
        &self,
        opt: OptimizationLevel,
    ) -> Result<JitFunction<unsafe extern "C" fn() -> u64>, Error> {
        let execution_engine = self
            .module()
            .create_jit_execution_engine(opt)
            .map_err(|_| LLVMError::FailedToCreateJIT)?;

        unsafe { execution_engine.get_function(self.module().get_name().to_str()?) }
            .map_err(Into::into)
    }
}
