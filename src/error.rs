use value::TypeID;

#[derive(Debug, Fail)]
pub enum TranslationError {
    #[fail(display = "Use of undeclared variable")]
    UndeclaredVariable,

    #[fail(display = "Cannot assign twice to immutable variable")]
    ImmutableAssign,

    #[fail(display = "Use of undeclared type identifier")]
    UndeclaredType,

    #[fail(display = "Value is expected but found Type")]
    ValueExpected,

    #[fail(display = "Type is expected but found Value")]
    TypeExpected,

    #[fail(display = "LLVM Value is not available for this value")]
    LLVMValueNotAvailable,

    #[fail(display = "Invalid Type")]
    InvalidType,

    #[fail(display = "Can't pop the scope stack anymore")]
    UnexpectedScopePop,

    #[fail(display = "Attempt to create a new branch in an invalid context")]
    InvalidContextBranch,

    #[fail(display = "Failed to create JIT execution engine")]
    FailedToCreateJIT,

    #[fail(display = "Invalid Cast from {:?} to {:?}", from, to)]
    InvalidCast {
        from: TypeID,
        to: TypeID,
    },

    #[fail(
        display = "Attempt to convert incompatible llvm type {} to expressi's type representation",
        from
    )]
    LLVMTypeConversion {
        from: String,
    },

    #[fail(
        display = "Attempt to convert incompatible llvm type {} to expressi's type representation",
        type_description
    )]
    InternalTypeConversion {
        // TODO: Use better representation than string
        type_description: String,
    },
}

#[derive(Debug, Fail)]
pub enum InternalError {
    #ifail(display = "Use of invalid value ID")]
    InvalidValueID,

    #[fail(display = "Use of invalid type ID")]
    InvalidTypeID,
}

#[derive(Debug, Fail)]
enum LLVMError {
    #[fail(display = "Failed to initialize the target: {}", message)]
    TargetIntializationFailed {
        message: String,
    },

    #[fail(display = "The function '{}' is invaild", name)]
    FunctionVerificationError {
        name: String,
    },

    #[fail(display = "Invaild module was generated: {}", message)]
    ModuleVerificationError {
        message: String,
    },
}

#[derive(Fail, Debug)]
pub enum CLIError {
    #[fail(display = "IO Error: {}", message)]
    IOError {
        message: String,
    },

    #[fail(display = "File not found: {}", path)]
    NotFound {
        path: String,
    },
}

#[fail(display = "Failed to parse: {}", message)]
pub struct ParseError {
    pub message: String,
}

