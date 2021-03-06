use super::error::CLIError;
use super::opts::RunOpt;
use super::shell::Shell;
use crate::codegen::{compile, initialization};
use crate::parser;
use crate::transform::TransformManager;
use crate::translator::translate_ast;

use failure::Error;

use ansi_term::Colour::{Blue, Red};
use inkwell::execution_engine;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process;

pub fn run(opt: &RunOpt) -> Result<!, Error> {
    initialization::initialize_native()?;

    if let Some(path) = &opt.input {
        let mut f = File::open(path).map_err(|_| CLIError::NotFound { path: path.clone() })?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)
            .map_err(|error| CLIError::IOError { error })?;

        let func = compile_jit(&contents.trim(), "file_input", opt)?;
        process::exit(unsafe { func.call() } as i32)
    } else {
        let home = dirs::home_dir().unwrap_or_else(|| env::current_dir().unwrap());
        let mut shell = Shell::new(home.join(".expressi_history"));
        loop {
            let line = shell.get_next_line()?;
            match compile_jit(line.trim(), "repl", opt) {
                Ok(func) => {
                    println!(
                        "{}{}",
                        Blue.paint("-> "),
                        Blue.paint(unsafe { func.call() }.to_string())
                    );
                }
                Err(e) => {
                    eprintln!("{}: {}", Red.paint("Error"), e);
                }
            }
        }
    }
}

type CompiledFunc = unsafe extern "C" fn() -> u64;

pub fn compile_jit(
    source: &str,
    module_name: &str,
    opt: &RunOpt,
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

    let result = compile::compile_eir(transformed, module_name)?;

    if opt.print_ir {
        eprintln!("LLVM IR: \n{}", result.llvm_ir());
    }

    result.verify()?;

    result.emit_function(opt.optimization_level.into())
}
