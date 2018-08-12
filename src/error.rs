#[derive(Fail, Debug)]
#[fail(display = "Use of undeclared variable")]
pub struct UndeclaredVariableError;
