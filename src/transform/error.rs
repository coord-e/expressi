use value::TypeID;

#[derive(Debug, Fail)]
pub enum TypeInferError {
    #[fail(
        display = "Mismatched types. expected: {:?}, found: {:?}",
        expected,
        found
    )]
    MismatchedTypes { expected: TypeID, found: TypeID },

    #[fail(display = "Undeclared identifier \"{}\"", ident)]
    UndeclaredIdentifier { ident: String },

    #[fail(display = "Unexpected not-typed value")]
    NotTyped,
}
