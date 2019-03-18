use super::compile;
use super::initialization;
use super::target_machine::create_target_machine;

use inkwell::targets::RelocMode;
use inkwell::OptimizationLevel;
use tempfile;

use std::io::Write;
use std::process::Command;

fn link_and_exec(bytes: &[u8], extension: &str) -> i32 {
    let mut f = tempfile::Builder::new()
        .suffix(extension)
        .tempfile()
        .unwrap();
    f.write_all(bytes).unwrap();
    let path = f.into_temp_path();
    let exe_path = path.parent().unwrap().join(path.file_stem().unwrap());
    let success = Command::new("gcc")
        .arg(&path)
        .arg("-o")
        .arg(&exe_path)
        .status()
        .unwrap()
        .success();
    assert!(success);

    Command::new(exe_path).status().unwrap().code().unwrap()
}

#[test]
fn emit_assembly() {
    initialization::initialize_native().unwrap();

    let result = compile::compile_string("2 * 3 * 7", "main").unwrap();
    let target =
        create_target_machine(None, None, None, OptimizationLevel::None, RelocMode::PIC).unwrap();
    let asm = result.emit_assembly(&target).unwrap();
    let code = link_and_exec(asm.as_bytes(), ".s");
    assert_eq!(code, 42);
}

#[test]
fn emit_object() {
    initialization::initialize_native().unwrap();

    let result = compile::compile_string("2 * 3 * 7", "main").unwrap();
    let target =
        create_target_machine(None, None, None, OptimizationLevel::None, RelocMode::PIC).unwrap();
    let obj = result.emit_object(&target).unwrap();
    let code = link_and_exec(&obj, ".o");
    assert_eq!(code, 42);
}

#[test]
fn emit_llvm() {
    let result = compile::compile_string("2 * 3 * 7", "main").unwrap();
    let mut f = tempfile::Builder::new().suffix(".ll").tempfile().unwrap();
    f.write_all(result.llvm_ir().as_bytes()).unwrap();
    let path = f.into_temp_path();
    let out = Command::new("llc")
        .arg(&path)
        .arg("-o")
        .arg("-")
        .output()
        .unwrap();
    assert!(out.status.success());

    let code = link_and_exec(&out.stdout, ".s");
    assert_eq!(code, 42);
}
