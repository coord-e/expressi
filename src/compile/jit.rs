

pub fn compile_jit(
    source: &str,
    module_name: &str,
) -> Result<execution_engine::JitFunction<CompiledFunc>, Error> {
    let ast = parser::parse(&source)?;

    if opt.print_ast {
        eprintln!("AST:\n{:#?}", ast);
    }

    let eir = translate_ast(ast)?;

    if opt.print_eir {
        eprintln!("EIR:\n{}\n", eir);
    }

    let transformed = TransformManager::default().apply(eir)?;

    if opt.print_eir {
        eprintln!("Transformed EIR:\n{}\n", transformed);
    }

    let result = llvm::compile_eir(transformed, module_name)?;

    if opt.print_ir {
        eprintln!("LLVM IR: \n{}", result.llvm_ir());
    }

    result.verify()?;

    let execution_engine = result
        .module()
        .create_jit_execution_engine(opt.optimization_level.into())
        .map_err(|_| LLVMError::FailedToCreateJIT)?;

    unsafe { execution_engine.get_function(module_name) }.map_err(Into::into)
}
