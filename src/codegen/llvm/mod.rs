mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
pub use lower::lower;

use crate::ir as primer_ir;

pub fn emit_llvm(program: &primer_ir::Program) -> String {
    let module = lower(program);

    emit(&module)
}

#[cfg(test)]
mod tests {
    use crate::compile_to_ir;

    use super::{emit_llvm, ir::Instruction, lower};

    #[test]
    fn lowers_i64_add() {
        let program = compile_to_ir("x: i64 = 1 + 2; print(x);").unwrap();
        let module = lower(&program);

        assert!(module.instructions.iter().any(|instruction| {
            matches!(
                instruction,
                Instruction::Binary {
                    op: super::ir::BinaryOp::Add,
                    ..
                }
            )
        }));
    }

    #[test]
    fn emits_i64_add() {
        let program = compile_to_ir("x: i64 = 1 + 2; print(x);").unwrap();
        let llvm = emit_llvm(&program);

        assert!(llvm.contains("add i64 1, 2"));
    }

    #[test]
    fn emits_f32_add() {
        let program = compile_to_ir("x: f32 = 0.1 + 0.2; print(x);").unwrap();
        let llvm = emit_llvm(&program);

        assert!(llvm.contains("fadd float"));
        assert!(llvm.contains("fpext float"));
    }

    #[test]
    fn emits_f64_add() {
        let program = compile_to_ir("x: f64 = 0.1 + 0.2; print(x);").unwrap();
        let llvm = emit_llvm(&program);

        assert!(llvm.contains("fadd double"));
    }

    #[test]
    fn emits_llvm_22_compatible_float_literals() {
        let program = compile_to_ir("x: f32 = 0.1 + 0.2; print(x);").unwrap();
        let llvm = emit_llvm(&program);

        assert!(llvm.contains("fadd float 0x3FB99999A0000000, 0x3FC99999A0000000"));
    }
}
