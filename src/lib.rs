pub mod ast;
pub mod bytecode;
pub mod codegen;
pub mod diagnostic;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod source;
pub mod vm;

use ast::Program;
use diagnostic::Diagnostic;

pub fn compile(source: &str) -> Result<Program, Diagnostic> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    semantic::check(&program)?;

    Ok(program)
}

pub fn compile_to_ir(source: &str) -> Result<ir::Program, Diagnostic> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    ir::builder::build(&program)
}

pub fn compile_to_ir_text(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    Ok(ir::text::emit(&program))
}

// C コンパイラ
pub fn compile_to_c(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    Ok(codegen::emit_c(&program))
}

// LLVM コンパイラ
pub fn compile_to_llvm(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    Ok(codegen::emit_llvm(&program))
}

// Wasm コンパイラ
pub fn compile_to_wat(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    Ok(codegen::emit_wat(&program))
}

// QBE コンパイラ
pub fn compile_to_qbe(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    Ok(codegen::emit_qbe(&program))
}

// Windows x86-64 Direct Assembly コンパイラ
pub fn compile_to_x86_64_win_asm(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    Ok(codegen::emit_x86_64_win_asm(&program))
}

pub fn compile_to_bytecode(source: &str) -> Result<bytecode::BytecodeProgram, Diagnostic> {
    let program = compile_to_ir(source)?;

    Ok(bytecode::lower(&program))
}

pub fn compile_to_bytecode_text(source: &str) -> Result<String, Diagnostic> {
    let bytecode = compile_to_bytecode(source)?;

    Ok(bytecode::format_program(&bytecode))
}

pub fn run_vm(source: &str) -> Result<String, String> {
    let bytecode = compile_to_bytecode(source).map_err(format_legacy_diagnostic)?;

    vm::run(&bytecode)
}

// VM実行時エラーを構造化するまでは、コンパイル診断を既存形式へ変換する
fn format_legacy_diagnostic(diagnostic: Diagnostic) -> String {
    match diagnostic.primary_span() {
        Some(span) => format!("{} at byte {}", diagnostic.message(), span.start()),
        None => diagnostic.message().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::compile_to_ir_text;

    #[test]
    fn emits_primer_ir_with_resolved_types() {
        let ir = compile_to_ir_text("x: f32 = 0.1 + 0.2; print(x);").unwrap();

        assert_eq!(
            ir,
            "; Primer IR v0.1\n\n%x: f32 = add.f32(0.1f32, 0.2f32)\nprint.f32 %x:f32\n"
        );
    }
}
