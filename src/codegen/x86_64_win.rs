mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
use lower::lower;

use crate::{diagnostic::Diagnostic, ir as primer_ir};

pub fn emit_x86_64_win_asm(program: &primer_ir::Program) -> Result<String, Diagnostic> {
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

    #[test]
    fn lowers_product_values_to_stack_fields() {
        let program = compile_to_ir(
            "type Point { x: f64 = 0.0, y: f64, }
             point: Point = Point { y: 2.0, };
             print(point.x);",
        )
        .unwrap();
        let asm = emit_x86_64_win_asm(&program).unwrap();

        assert!(asm.contains("movsd %xmm0, -8(%rbp)"));
        assert!(asm.contains("movsd %xmm0, -16(%rbp)"));
        assert!(asm.contains("movsd -8(%rbp), %xmm0"));
        assert!(asm.contains("callq printf"));
    }

    #[test]
    fn emits_typed_functions_with_windows_register_arguments() {
        let program = compile_to_ir(
            "fn add(left: i64, right: i64) -> i64 { return left + right; }
             answer: i64 = add(20, 22);
             print(answer);",
        )
        .unwrap();
        let asm = emit_x86_64_win_asm(&program).unwrap();

        assert!(asm.contains("primer_fn_add_0:"));
        assert!(asm.contains("movq %rcx, -8(%rbp)"));
        assert!(asm.contains("movq %rdx, -16(%rbp)"));
        assert!(asm.contains("callq primer_fn_add_0"));
    }

    #[test]
    fn preserves_nested_calls_and_mixed_argument_registers() {
        let program = compile_to_ir(
            "fn twice(value: i64) -> i64 { return value * 2; }
             fn select(a: i64, b: f64, c: i64, d: f32) -> f64 { return b; }
             result: f64 = select(twice(1), 2.5, twice(3), 4.5f32);
             print(result);",
        )
        .unwrap();
        let asm = emit_x86_64_win_asm(&program).unwrap();

        assert_eq!(asm.matches("callq primer_fn_twice_0").count(), 2);
        assert!(asm.contains("%rcx"));
        assert!(asm.contains("%xmm1"));
        assert!(asm.contains("%r8"));
        assert!(asm.contains("%xmm3"));
    }
}
