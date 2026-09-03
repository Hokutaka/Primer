mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
use lower::lower;

use crate::{diagnostic::Diagnostic, ir as primer_ir};

pub fn emit_wat(program: &primer_ir::Program) -> Result<String, Diagnostic> {
    if let Some(diagnostic) = program.unsupported_functions("emit-wat") {
        return Err(diagnostic);
    }
    let module = lower(program);

    Ok(emit(&module))
}

#[cfg(test)]
mod tests {
    use crate::compile_to_ir;

    use super::{
        emit_wat,
        ir::{Instruction, Type},
        lower,
    };

    #[test]
    fn lowers_i64_negation_to_stack_instructions() {
        let program = compile_to_ir("x: i64 = -1;").unwrap();
        let module = lower(&program);

        assert!(module.instructions.windows(3).any(|instructions| {
            matches!(
                instructions,
                [
                    Instruction::I64Const(0),
                    Instruction::I64Const(1),
                    Instruction::I64Sub
                ]
            )
        }));
    }

    #[test]
    fn lowers_print_type() {
        let program = compile_to_ir("print(1);").unwrap();
        let module = lower(&program);

        assert!(matches!(
            module.instructions.last(),
            Some(Instruction::CallPrint(Type::I64))
        ));
    }

    #[test]
    fn emits_i64_add() {
        let program = compile_to_ir(
            "x: i64 = 1 + 2;
             print(x);",
        )
        .unwrap();

        let wat = emit_wat(&program).unwrap();

        assert!(wat.contains("i64.add"));

        assert!(wat.contains("(local $primer_x i64)"));

        assert!(wat.contains("call $print_i64"));
    }

    #[test]
    fn emits_f32_add() {
        let program = compile_to_ir(
            "x: f32 = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let wat = emit_wat(&program).unwrap();

        assert!(wat.contains("f32.const 0.1"));

        assert!(wat.contains("f32.const 0.2"));

        assert!(wat.contains("f32.add"));

        assert!(wat.contains("call $print_f32"));
    }

    #[test]
    fn emits_f64_add() {
        let program = compile_to_ir(
            "x: f64 = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let wat = emit_wat(&program).unwrap();

        assert!(wat.contains("f64.add"));

        assert!(wat.contains("call $print_f64"));
    }

    #[test]
    fn inferred_f32_uses_f32() {
        let program = compile_to_ir(
            "a: f32 = 0.1 + 0.2;
             b: infer = a + a;",
        )
        .unwrap();

        let wat = emit_wat(&program).unwrap();

        assert!(wat.contains("(local $primer_b f32)"));

        assert!(wat.contains("f32.add"));
    }

    #[test]
    fn emits_all_integer_comparisons() {
        let program = compile_to_ir(
            "a: bool = 1 == 1; b: bool = 1 != 2; c: bool = 1 < 2;
             d: bool = 1 <= 2; e: bool = 2 > 1; f: bool = 2 >= 1;",
        )
        .unwrap();
        let wat = emit_wat(&program).unwrap();

        for instruction in [
            "i64.eq", "i64.ne", "i64.lt_s", "i64.le_s", "i64.gt_s", "i64.ge_s",
        ] {
            assert!(wat.contains(instruction));
        }
    }

    #[test]
    fn lowers_product_values_to_linear_memory() {
        let program = compile_to_ir(
            "type Point { x: f64 = 0.0, y: f64, }
             point: Point = Point { y: 2.0, };
             print(point.x);",
        )
        .unwrap();
        let wat = emit_wat(&program).unwrap();

        assert!(wat.contains("(memory 1)"));
        assert!(wat.contains("f64.store"));
        assert!(wat.contains("f64.load"));
        assert!(wat.contains("call $print_f64"));
    }
}
