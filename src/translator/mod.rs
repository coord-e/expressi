pub mod ast_translator;
pub mod eir_translator;

pub use translator::ast_translator::ASTTranslator;
pub use translator::eir_translator::translate_eir;
