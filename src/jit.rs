use builder::Builder;
use error::{
    FailedToCreateJITError, FunctionVerificationError, ModuleVerificationError, ParseError,
    TargetInitializationError,
};
use expression::Expression;
use parser;
use translator::{EIRTranslator, ASTTranslator};
use value::TypeID;

use std::rc::Rc;

use failure::Error;

use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;
use inkwell::{builder, context, execution_engine, module};

type CompiledFunc = unsafe extern "C" fn() -> u64;

pub struct JIT {
    context: context::ContextRef,
    builder: builder::Builder,
}

impl JIT {
    pub fn new() -> Result<Self, Error> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|message| TargetInitializationError { message })?;

        let context = context::Context::get_global();
        let builder = context.create_builder();

        Ok(Self { context, builder })
    }

    /// Compile a string in the toy language into machine code.
    pub fn compile(
        &mut self,
        name: &str,
        input: &str,
    ) -> Result<execution_engine::Symbol<CompiledFunc>, Error> {
        let module = Rc::new(self.context.create_module(name));
        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|_| FailedToCreateJITError)?;

        // Parse the string, producing AST nodes.
        let ast = parser::parse(&input).map_err(|e| ParseError {
            message: e.to_string(),
        })?;

        // Translate the AST nodes into Cranelift IR.
        self.translate(module.clone(), ast)?;

        unsafe { execution_engine.get_function(name) }.map_err(|e| e.into())
    }

    // Translate from toy-language AST nodes into Cranelift IR.
    fn translate(&mut self, module: Rc<module::Module>, expr: Expression) -> Result<(), Error> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);

        let function = module.add_function(module.get_name().to_str()?, fn_type, None);
        let basic_block = self.context.append_basic_block(&function, "entry");

        self.builder.position_at_end(&basic_block);

        let builder = Builder::new(&mut self.builder, module.clone());

        let mut trans = FunctionTranslator { builder };

        let evaluated_value = trans.translate_expr(expr)?.expect_value()?;
        trans.builder.ret_int(evaluated_value)?;

        if !function.verify(true) {
            eprintln!(""); // function.verify print results to stderr directory but it doesn't contain \n on the end
            return Err(FunctionVerificationError {
                name: function.get_name().to_str()?.to_string(),
            }.into());
        }

        if let Err(message) = module.verify() {
            return Err(ModuleVerificationError {
                message: message.to_string(),
            }.into());
        }

        Ok(())
    }
}
