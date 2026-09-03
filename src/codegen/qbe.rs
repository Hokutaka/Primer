mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
use lower::lower;

use crate::{diagnostic::Diagnostic, ir as primer_ir};

pub fn emit_qbe(program: &primer_ir::Program) -> Result<String, Diagnostic> {
    if let Some(diagnostic) = program.unsupported_product_type("emit-qbe") {
        return Err(diagnostic);
    }
    let module = lower(program);

    Ok(emit(&module))
}

#[cfg(test)]
mod tests {
    use crate::compile_to_ir;

    use super::{emit_qbe, ir::Instruction, lower};

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
        let qbe = emit_qbe(&program).unwrap();

        assert!(qbe.contains("=l add 1, 2"));
        assert!(qbe.contains("storel %tmp0, %slot_x"));
        assert!(qbe.contains("call $printf(l $fmt_i64"));
    }

    #[test]
    fn emits_f32_add() {
        let program = compile_to_ir("x: f32 = 0.1 + 0.2; print(x);").unwrap();
        let qbe = emit_qbe(&program).unwrap();

        assert!(qbe.contains("=s add s_0.1, s_0.2"));
        assert!(qbe.contains("=d exts %tmp"));
        assert!(qbe.contains("call $printf(l $fmt_f32"));
    }

    #[test]
    fn emits_f64_add() {
        let program = compile_to_ir("x: f64 = 0.1 + 0.2; print(x);").unwrap();
        let qbe = emit_qbe(&program).unwrap();

        assert!(qbe.contains("=d add d_0.1, d_0.2"));
        assert!(qbe.contains("call $printf(l $fmt_f64"));
    }

    #[test]
    fn inferred_f32_uses_single() {
        let program = compile_to_ir("a: f32 = 0.1 + 0.2; b: infer = a + a;").unwrap();
        let qbe = emit_qbe(&program).unwrap();

        assert!(qbe.contains("stores %tmp"));
    }

    #[test]
    fn emits_all_integer_comparisons() {
        let program = compile_to_ir(
            "a: bool = 1 == 1; b: bool = 1 != 2; c: bool = 1 < 2;
             d: bool = 1 <= 2; e: bool = 2 > 1; f: bool = 2 >= 1;",
        )
        .unwrap();
        let qbe = emit_qbe(&program).unwrap();

        for instruction in ["ceql", "cnel", "csltl", "cslel", "csgtl", "csgel"] {
            assert!(qbe.contains(instruction));
        }
    }
}
