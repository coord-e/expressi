#[derive(Fail, Debug)]
#[fail(display = "Use of undeclared variable")]
pub struct UndeclaredVariableError;

#[derive(Fail, Debug)]
#[fail(display = "Failed to parse: {}", message)]
pub struct ParseError {
    pub message: String
}

#[derive(Fail, Debug)]
#[fail(display = "Failed to finalize the function: {}", message)]
pub struct FinalizationError {
    pub message: String
}
