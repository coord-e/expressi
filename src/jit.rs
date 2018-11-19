use error::{LLVMError, ParseError};
use expression::Expression;
use parser;
use transform::{Transform, TypeInfer};
use translator::eir_translator::Builder;
use translator::{ASTTranslator, EIRTranslator};
use type_::{TypeID, TypeStore};

use std::cell::RefCell;
use std::rc::Rc;

use failure::Error;

use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;
use inkwell::{builder, context, execution_engine, module};

type CompiledFunc = unsafe extern "C" fn() -> u64;

pub struct JIT {
    context: context::ContextRef,
    builder: builder::Builder,
    print_ast: bool,
    print_eir: bool,
    print_ir: bool,
}

impl JIT {
    pub fn new(print_ast: bool, print_eir: bool, print_ir: bool) -> Result<Self, Error> {
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|message| LLVMError::TargetInitializationFailed { message })?;

        let context = context::Context::get_global();
        let builder = context.create_builder();

        Ok(Self {
            context,
            builder,
            print_ast,
            print_eir,
            print_ir,
        })
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
            .map_err(|_| LLVMError::FailedToCreateJIT)?;

        // Parse the string, producing AST nodes.
        let ast = parser::parse(&input).map_err(|e| ParseError {
            message: e.to_string(),
        })?;
        if self.print_ast {
            eprintln!("AST:\n{:#?}", ast);
        }

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

        let mut type_store = TypeStore::new();

        let eir = {
            let mut a_trans = ASTTranslator {
                type_store: &mut type_store,
            };
            a_trans.translate_expr(expr)?
        };
        if self.print_eir {
            eprintln!("EIR:\n{:#?}", eir);
        }

        let transformed = {
            let mut ti = TypeInfer::new(&mut type_store);
            eir.apply(ti)?
        };

        if self.print_eir {
            eprintln!("Transformed EIR:\n{:#?}", transformed);
        }

        let builder = Builder::new(&mut type_store, &mut self.builder, module.clone());
        let mut trans = EIRTranslator { builder };

        let evaluated_value = trans.translate_expr(transformed)?.expect_value()?;
        trans.builder.ret_int(evaluated_value)?;

        if !function.verify(true) {
            eprintln!(""); // function.verify print results to stderr directory but it doesn't contain \n on the end
            return Err(LLVMError::FunctionVerificationError {
                name: function.get_name().to_str()?.to_string(),
            }.into());
        }

        if let Err(message) = module.verify() {
            return Err(LLVMError::ModuleVerificationError {
                message: message.to_string(),
            }.into());
        }

        if self.print_ir {
            eprintln!("LLVM IR:");
            module.print_to_stderr();
        }

        Ok(())
    }
}
