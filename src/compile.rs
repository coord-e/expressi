use crate::error::LLVMError;
use crate::ir;
use crate::translator::eir_translator::Builder;
use crate::translator::translate_eir;

use failure::Error;

use inkwell::{context, module};

pub struct CompilationResult {
    module: module::Module,
}

impl CompilationResult {
    pub fn new(module: module::Module) -> Self {
        CompilationResult { module }
    }

    pub fn module(&self) -> &module::Module {
        &self.module
    }

    pub fn llvm_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    pub fn verify(&self) -> Result<(), Error> {
        if let Err(message) = self.module.verify() {
            return Err(LLVMError::ModuleVerificationError {
                message: message.to_string(),
            }
            .into());
        }
        Ok(())
    }
}

pub fn compile_eir(eir: ir::Node, module_name: &str) -> Result<CompilationResult, Error> {
    let context = context::Context::get_global();
    let inst_builder = context.create_builder();

    let module = context.create_module(module_name);

    let i64_type = context.i64_type();
    let fn_type = i64_type.fn_type(&[], false);

    let function = module.add_function(module_name, fn_type, None);
    let basic_block = context.append_basic_block(&function, "entry");

    inst_builder.position_at_end(&basic_block);

    let mut builder = Builder::new(inst_builder, module);

    let evaluated_value = translate_eir(&mut builder, eir)?.expect_value()?;
    builder.ret_int(evaluated_value)?;

    return Ok(CompilationResult::new(builder.take_module()));
}
