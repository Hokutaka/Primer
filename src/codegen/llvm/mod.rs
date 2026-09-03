mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
use lower::lower;

use crate::{diagnostic::Diagnostic, ir as primer_ir};

pub fn emit_llvm(program: &primer_ir::Program) -> Result<String, Diagnostic> {
    let module = lower(program);

    Ok(emit(&module))
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
        let llvm = emit_llvm(&program).unwrap();

        assert!(llvm.contains("add i64 1, 2"));
    }

    #[test]
    fn emits_f32_add() {
        let program = compile_to_ir("x: f32 = 0.1 + 0.2; print(x);").unwrap();
        let llvm = emit_llvm(&program).unwrap();

        assert!(llvm.contains("fadd float"));
        assert!(llvm.contains("fpext float"));
    }

    #[test]
    fn emits_f64_add() {
        let program = compile_to_ir("x: f64 = 0.1 + 0.2; print(x);").unwrap();
        let llvm = emit_llvm(&program).unwrap();

        assert!(llvm.contains("fadd double"));
    }

    #[test]
    fn emits_llvm_22_compatible_float_literals() {
        let program = compile_to_ir("x: f32 = 0.1 + 0.2; print(x);").unwrap();
        let llvm = emit_llvm(&program).unwrap();

        assert!(llvm.contains("fadd float 0x3FB99999A0000000, 0x3FC99999A0000000"));
    }

    #[test]
    fn emits_all_integer_comparisons() {
        let program = compile_to_ir(
            "a: bool = 1 == 1; b: bool = 1 != 2; c: bool = 1 < 2;
             d: bool = 1 <= 2; e: bool = 2 > 1; f: bool = 2 >= 1;",
        )
        .unwrap();
        let llvm = emit_llvm(&program).unwrap();

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
    fn emits_product_construction_and_field_access() {
        let program = compile_to_ir(
            "type Point { x: f64 = 0.0, y: f64, }
             point: Point = Point { y: 2.0, };
             print(point.x);",
        )
        .unwrap();
        let llvm = emit_llvm(&program).unwrap();

        assert!(llvm.contains("%primer.type.Point.0 = type { double, double }"));
        assert!(llvm.contains("insertvalue %primer.type.Point.0 poison, double"));
        assert!(llvm.contains("extractvalue %primer.type.Point.0"));
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
        let llvm = emit_llvm(&program).unwrap();

        assert_eq!(llvm.matches("%primer_marker = alloca i1").count(), 1);
        assert!(
            llvm.find("%primer_marker = alloca i1").unwrap()
                < llvm.find("while_condition").unwrap()
        );
    }

    #[test]
    fn emits_typed_functions_and_calls() {
        let program = compile_to_ir(
            "fn add(left: i64, right: i64) -> i64 { return left + right; }
             answer: i64 = add(20, 22);
             print(answer);",
        )
        .unwrap();
        let llvm = emit_llvm(&program).unwrap();

        assert!(llvm.contains("define i64 @primer.fn.add.0(i64 %arg0, i64 %arg1)"));
        assert!(llvm.contains("store i64 %arg0, ptr %primer_left"));
        assert!(llvm.contains("call i64 @primer.fn.add.0(i64 20, i64 22)"));
        assert!(llvm.contains("ret i64"));
    }
}
