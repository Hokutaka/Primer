pub mod ast;
pub mod bytecode;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod vm;

use ast::Program;

pub fn compile(source: &str) -> Result<Program, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    semantic::check(&program)?;

    Ok(program)
}

// C コンパイラ
pub fn compile_to_c(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    let bindings = semantic::check(&program)?;

    Ok(codegen::emit_c(&program, &bindings))
}

// LLVM コンパイラ
pub fn compile_to_llvm(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    let bindings = semantic::check(&program)?;

    Ok(codegen::emit_llvm(&program, &bindings))
}

// Wasm コンパイラ
pub fn compile_to_wat(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    let bindings = semantic::check(&program)?;

    Ok(codegen::emit_wat(&program, &bindings))
}

// QBE コンパイラ
pub fn compile_to_qbe(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source)?;

    let program = parser::parse(tokens)?;

    let bindings = semantic::check(&program)?;

    Ok(codegen::emit_qbe(&program, &bindings))
}

// Windows x86-64 Direct Assembly コンパイラ
pub fn compile_to_x86_64_win_asm(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source)?;

    let program = parser::parse(tokens)?;

    let bindings = semantic::check(&program)?;

    Ok(codegen::emit_x86_64_win_asm(&program, &bindings))
}

pub fn compile_to_bytecode(source: &str) -> Result<bytecode::BytecodeProgram, String> {
    let tokens = lexer::lex(source)?;

    let program = parser::parse(tokens)?;

    let bindings = semantic::check(&program)?;

    Ok(bytecode::compile(&program, &bindings))
}

pub fn compile_to_bytecode_text(source: &str) -> Result<String, String> {
    let bytecode = compile_to_bytecode(source)?;

    Ok(bytecode::format_program(&bytecode))
}

pub fn run_vm(source: &str) -> Result<String, String> {
    let bytecode = compile_to_bytecode(source)?;

    vm::run(&bytecode)
}
