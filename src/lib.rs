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
use bytecode::InstructionOrigin;
use diagnostic::Diagnostic;

/// VM実行エラーと、失敗したbytecode命令の出自を保持します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionError {
    vm_error: vm::VmError,
    origin: Option<InstructionOrigin>,
}

impl ExecutionError {
    /// VMが報告した構造化エラーを返します。
    pub const fn vm_error(self) -> vm::VmError {
        self.vm_error
    }

    /// 失敗した命令の出自を返します。
    ///
    /// 命令番号がbytecodeの範囲外で、対応する命令自体が存在しない場合は`None`です。
    pub const fn origin(self) -> Option<InstructionOrigin> {
        self.origin
    }
}

/// `run_vm`で発生したコンパイルエラーまたはVM実行エラーを表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// Primerソースからbytecodeを生成するまでに見つかった問題です。
    Compilation(Diagnostic),

    /// 生成されたbytecodeをPrimer VMで実行中に見つかった問題です。
    Execution(ExecutionError),
}

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

    codegen::emit_c(&program)
}

// LLVM コンパイラ
pub fn compile_to_llvm(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_llvm(&program)
}

// Wasm コンパイラ
pub fn compile_to_wat(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_wat(&program)
}

// QBE コンパイラ
pub fn compile_to_qbe(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_qbe(&program)
}

// Windows x86-64 Direct Assembly コンパイラ
pub fn compile_to_x86_64_win_asm(source: &str) -> Result<String, Diagnostic> {
    let program = compile_to_ir(source)?;

    codegen::emit_x86_64_win_asm(&program)
}

pub fn compile_to_bytecode(source: &str) -> Result<bytecode::BytecodeProgram, Diagnostic> {
    let program = compile_to_ir(source)?;

    bytecode::lower(&program)
}

pub fn compile_to_bytecode_text(source: &str) -> Result<String, Diagnostic> {
    let bytecode = compile_to_bytecode(source)?;

    Ok(bytecode::format_program(&bytecode))
}

pub fn run_vm(source: &str) -> Result<String, RunError> {
    let bytecode = compile_to_bytecode(source).map_err(RunError::Compilation)?;

    vm::run(&bytecode).map_err(|vm_error| {
        let origin = bytecode
            .instructions
            .get(vm_error.instruction_index())
            .map(|instruction| instruction.origin);

        RunError::Execution(ExecutionError { vm_error, origin })
    })
}

#[cfg(test)]
mod tests {
    use super::{RunError, compile_to_c, compile_to_ir_text, run_vm};
    use crate::{bytecode::InstructionOrigin, source::Span, vm::VmErrorKind};

    #[test]
    fn emits_primer_ir_with_resolved_types() {
        let ir = compile_to_ir_text("x: f32 = 0.1 + 0.2; print(x);").unwrap();

        assert_eq!(
            ir,
            "; Primer IR v0.1\n\n%x@0: f32 = add.f32(0.1f32, 0.2f32)\nprint.f32 %x@0:f32\n"
        );
    }

    #[test]
    fn distinguishes_compilation_and_execution_errors() {
        let compilation_error = run_vm("print(missing);").unwrap_err();
        let execution_error = run_vm("print(1 / 0);").unwrap_err();

        assert!(matches!(compilation_error, RunError::Compilation(_)));

        match execution_error {
            RunError::Execution(error) => {
                assert_eq!(error.vm_error().kind(), VmErrorKind::DivisionByZero);
                assert_eq!(error.vm_error().instruction_index(), 2);
                assert_eq!(
                    error.origin(),
                    Some(InstructionOrigin::Source(Span::new(6, 11)))
                );
            }
            RunError::Compilation(diagnostic) => {
                panic!("expected execution error, found {diagnostic:?}");
            }
        }
    }

    #[test]
    fn emits_product_types_and_field_origins_in_primer_ir() {
        let source = "
            type Point { x: f64 = 0.0, y: f64, }
            point: Point = Point { y: 2.0, };
            print(point.x);
        ";
        let ir = compile_to_ir_text(source).unwrap();

        assert_eq!(
            ir,
            concat!(
                "; Primer IR v0.1\n\n",
                "type %Point@0 {\n",
                "  field %x@0: f64 = 0.0f64\n",
                "  field %y@1: f64\n",
                "}\n\n",
                "%point@0: %Point@0 = construct %Point@0 { ",
                "field %y@1 = 2.0f64 [explicit]; ",
                "field %x@0 = 0.0f64 [default]; }\n",
                "print.f64 field(%point@0:%Point@0, %x@0):f64\n",
            )
        );
    }

    #[test]
    fn unsupported_backend_names_its_output_route() {
        let source = "type Point { x: f64, } point: Point = Point { x: 1.0, };";
        let error = compile_to_c(source).unwrap_err();

        assert_eq!(
            error.message(),
            "output route `emit-c` does not support product types yet"
        );
        assert_eq!(error.primary_span(), Some(Span::new(0, 22)));
    }
}
