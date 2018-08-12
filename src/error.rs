use cranelift::prelude::types;
use value::Type;

#[derive(Fail, Debug)]
#[fail(display = "Use of undeclared variable")]
pub struct UndeclaredVariableError;

#[derive(Fail, Debug)]
#[fail(display = "Invalid Type")]
pub struct TypeError;

#[derive(Fail, Debug)]
#[fail(display = "Invalid Cast")]
pub struct InvalidCastError;

#[derive(Fail, Debug)]
#[fail(
    display = "Attempt to convert incompatible cranelift type {} to expressi's type representation",
    from
)]
pub struct CraneliftTypeConversionError {
    pub from: types::Type,
}

#[derive(Fail, Debug)]
#[fail(
    display = "Attempt to convert incompatible cranelift type {} to expressi's type representation",
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
