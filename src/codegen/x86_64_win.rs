mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
use lower::lower;

use crate::{diagnostic::Diagnostic, ir as primer_ir};

pub fn emit_x86_64_win_asm(program: &primer_ir::Program) -> Result<String, Diagnostic> {
    if let Some(diagnostic) = program.unsupported_product_type("emit-asm") {
        return Err(diagnostic);
    }
    let module = lower(program);

    Ok(emit(&module))
}

#[cfg(test)]
mod tests {
    use crate::compile_to_ir;

    use super::{
        emit_x86_64_win_asm,
        ir::{BinaryOp, FloatConstant, Instruction},
        lower,
    };

    #[test]
    fn lowers_i64_add_to_asm_ir() {
        let program = compile_to_ir("x: i64 = 1 + 2;").unwrap();
        let module = lower(&program);

        assert!(
            module
                .instructions
                .iter()
                .any(|instruction| matches!(instruction, Instruction::I64Binary(BinaryOp::Add)))
        );
    }

    #[test]
    fn lowers_f32_constant_to_asm_ir() {
        let program = compile_to_ir("x: f32 = 0.1;").unwrap();
        let module = lower(&program);

        assert!(matches!(
            module.float_constants.first(),
            Some(FloatConstant::F32 { .. })
        ));
    }

    #[test]
    fn emits_i64_arithmetic() {
        let program = compile_to_ir(
            "x: i64 = 1 + 2;
             print(x);",
        )
        .unwrap();

        let asm = emit_x86_64_win_asm(&program).unwrap();

        assert!(asm.contains("addq %rcx, %rax"));

        assert!(asm.contains("callq printf"));
    }

    #[test]
    fn emits_f32_arithmetic() {
        let program = compile_to_ir(
            "x: f32 = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let asm = emit_x86_64_win_asm(&program).unwrap();

        assert!(asm.contains("addss %xmm1, %xmm0"));

        assert!(asm.contains("cvtss2sd %xmm0, %xmm1"));
    }

    #[test]
    fn emits_f64_arithmetic() {
        let program = compile_to_ir(
            "x: f64 = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let asm = emit_x86_64_win_asm(&program).unwrap();

        assert!(asm.contains("addsd %xmm1, %xmm0"));

        assert!(asm.contains("movq %xmm1, %rdx"));
    }

    #[test]
    fn emits_all_integer_comparisons() {
        let program = compile_to_ir(
            "a: bool = 1 == 1; b: bool = 1 != 2; c: bool = 1 < 2;
             d: bool = 1 <= 2; e: bool = 2 > 1; f: bool = 2 >= 1;",
        )
        .unwrap();
        let asm = emit_x86_64_win_asm(&program).unwrap();

        for instruction in [
            "sete %al",
            "setne %al",
            "setl %al",
            "setle %al",
            "setg %al",
            "setge %al",
        ] {
            assert!(asm.contains(instruction));
        }
    }
}
