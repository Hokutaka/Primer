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

    #[test]
    fn emits_all_integer_comparisons() {
        let program = compile_to_ir(
            "a: bool = 1 == 1; b: bool = 1 != 2; c: bool = 1 < 2;
             d: bool = 1 <= 2; e: bool = 2 > 1; f: bool = 2 >= 1;",
        )
        .unwrap();
        let llvm = emit_llvm(&program);

        for instruction in [
            "icmp eq i64",
            "icmp ne i64",
            "icmp slt i64",
            "icmp sle i64",
            "icmp sgt i64",
            "icmp sge i64",
        ] {
            assert!(llvm.contains(instruction));
        }
    }

    #[test]
    fn allocates_loop_bindings_once_in_entry() {
        let program = compile_to_ir(
            "mut count: i64 = 0;
             while count < 2 {
                 marker: bool = true;
                 print(marker);
                 count = count + 1;
             }",
        )
        .unwrap();
        let llvm = emit_llvm(&program);

        assert_eq!(llvm.matches("%primer_marker = alloca i1").count(), 1);
        assert!(
            llvm.find("%primer_marker = alloca i1").unwrap()
                < llvm.find("while_condition").unwrap()
        );
    }
}
