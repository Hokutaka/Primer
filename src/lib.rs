pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod semantic;

use ast::Program;

pub fn compile(source: &str) -> Result<Program, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    semantic::check(&program)?;

    Ok(program)
}

pub fn compile_to_c(source: &str) -> Result<String, String> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

    let bindings = semantic::check(&program)?;

    Ok(codegen::emit_c(&program, &bindings))
}
