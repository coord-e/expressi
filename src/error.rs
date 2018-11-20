use std::io;

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

    // TODO: Hold BasicTypeEnum
    #[fail(display = "Invalid Cast from {:?} to {:?}", from, to)]
    InvalidCast { from: String, to: String },

    #[fail(
        display = "Attempt to convert incompatible llvm type {} to expressi's type representation",
        from
    )]
    LLVMTypeConversion { from: String },

    #[fail(
        display = "Attempt to convert incompatible llvm type {} to expressi's type representation",
        type_description
    )]
    InternalTypeConversion {
        // TODO: Use better representation than string
        type_description: String,
    },

    #[fail(display = "Attempt to translate untyped value")]
    NotTyped
}

#[derive(Debug, Fail)]
pub enum InternalError {
    #[fail(display = "Use of invalid value ID")]
    InvalidValueID,

    #[fail(display = "Use of invalid type ID")]
    InvalidTypeID,

    #[fail(display = "Typed Typed Value detected")]
    DoubleTyped,
}

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
}

#[derive(Fail, Debug)]
pub enum CLIError {
    #[fail(display = "IO Error: {}", error)]
    IOError { error: io::Error },

    #[fail(display = "File not found: {}", path)]
    NotFound { path: String },
}

#[derive(Fail, Debug)]
#[fail(display = "Failed to parse: {}", message)]
pub struct ParseError {
    pub message: String,
}
