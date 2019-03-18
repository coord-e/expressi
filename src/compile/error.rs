use failure::Fail;

#[derive(Debug, Fail)]
pub enum LLVMError {
    #[fail(display = "Failed to initialize the target: {}", message)]
    TargetInitializationFailed { message: String },

    #[fail(display = "The function '{}' is invaild", name)]
    FunctionVerificationError { name: String },

    #[fail(display = "Invaild module was generated: {}", message)]
    ModuleVerificationError { message: String },

    #[fail(display = "Failed to create JIT execution engine")]
    FailedToCreateJIT,

    #[fail(display = "Failed to write the module to a buffer: {}", message)]
    MemoryBufferError { message: String },
}
