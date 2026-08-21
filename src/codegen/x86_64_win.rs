mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
pub use lower::lower;

use crate::ir as primer_ir;

pub fn emit_x86_64_win_asm(program: &primer_ir::Program) -> String {
    let module = lower(program);

    emit(&module)
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

        let asm = emit_x86_64_win_asm(&program);

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

        let asm = emit_x86_64_win_asm(&program);

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

        let asm = emit_x86_64_win_asm(&program);

        assert!(asm.contains("addsd %xmm1, %xmm0"));

        assert!(asm.contains("movq %xmm1, %rdx"));
    }
}
