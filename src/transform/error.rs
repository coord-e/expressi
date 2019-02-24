use transform::type_infer::type_::TypeVarID;
use transform::type_infer::Type;

#[derive(Debug, Fail)]
pub enum TypeInferError {
    #[fail(
        display = "Mismatched types. expected: {}, found: {}",
        expected,
        found
    )]
    MismatchedTypes { expected: Type, found: Type },

    #[fail(display = "Recursive type detected: {} vs {}", t1, t2)]
    RecursiveType { t1: TypeVarID, t2: Type },

    #[fail(display = "Undeclared identifier \"{}\"", ident)]
    UndeclaredIdentifier { ident: String },

    #[fail(display = "Unexpected not-typed value")]
    NotTyped,
}

#[derive(Debug, Fail)]
pub enum CheckCaptureError {
    #[fail(display = "Unexpected not-typed value")]
    NotTyped,

    #[fail(display = "Double typed")]
    DoubleTyped,
}
