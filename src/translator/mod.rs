pub mod ast_translator;
pub mod eir_translator;

pub use crate::translator::ast_translator::ASTTranslator;
pub use crate::translator::eir_translator::translate_eir;
