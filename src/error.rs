use inkwell::types;
use value::Type;

#[derive(Fail, Debug)]
#[fail(display = "Use of undeclared variable")]
pub struct UndeclaredVariableError;

#[derive(Fail, Debug)]
#[fail(display = "LLVM Value is not available for this value")]
pub struct LLVMValueNotAvailableError;

#[derive(Fail, Debug)]
#[fail(display = "Invalid Type")]
pub struct TypeError;

#[derive(Fail, Debug)]
#[fail(display = "Can't pop the scope stack anymore")]
pub struct UnexpectedScopePopError;

#[derive(Fail, Debug)]
#[fail(display = "Internal Error; Use of released value")]
pub struct ReleasedValueError;

#[derive(Fail, Debug)]
#[fail(display = "Attempt to create a new branch in an invalid context")]
pub struct InvalidContextBranchError;

#[derive(Fail, Debug)]
#[fail(display = "Failed to create JIT execution engine")]
pub struct FailedToCreateJITError;

#[derive(Fail, Debug)]
#[fail(display = "Invalid Cast from {} to {}", from, to)]
pub struct InvalidCastError {
    pub from: Type,
    pub to: Type,
}

#[derive(Fail, Debug)]
#[fail(
    display = "Failed to initialize the target: {}",
    message
)]
pub struct TargetInitializationError {
    pub message: String,
}

#[derive(Fail, Debug)]
#[fail(
    display = "Attempt to convert incompatible llvm type {} to expressi's type representation",
    from
)]
pub struct LLVMTypeConversionError {
    pub from: String,
}

#[derive(Fail, Debug)]
#[fail(
    display = "Attempt to convert incompatible llvm type {} to expressi's type representation",
    from
)]
pub struct InternalTypeConversionError {
    pub from: Type,
}

#[derive(Fail, Debug)]
#[fail(display = "Failed to parse: {}", message)]
pub struct ParseError {
    pub message: String,
}

#[derive(Fail, Debug)]
#[fail(display = "Failed to finalize the function: {}", message)]
pub struct FinalizationError {
    pub message: String,
}

#[derive(Fail, Debug)]
#[fail(display = "IO Error: {}", message)]
pub struct IOError {
    pub message: String,
}

#[derive(Fail, Debug)]
#[fail(display = "File not found: {}", path)]
pub struct NotFoundError {
    pub path: String,
}
