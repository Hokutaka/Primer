mod conversion;
mod emit;
mod integer;
pub mod ir;
mod lower;
mod string;

pub use emit::emit;
use lower::lower;

use crate::{diagnostic::Diagnostic, ir as primer_ir};

/// QBEの文字列出力で検証する実行環境です。ホストOSからは選びません。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    X86_64UnknownLinuxGnu,
}

impl Target {
    pub const fn triple(self) -> &'static str {
        "x86_64-unknown-linux-gnu"
    }
    pub fn parse(value: &str) -> Option<Self> {
        (value == "x86_64-unknown-linux-gnu").then_some(Self::X86_64UnknownLinuxGnu)
    }
}

pub fn emit_qbe(program: &primer_ir::Program) -> Result<String, Diagnostic> {
    emit_qbe_with_target(program, None)
}

pub fn emit_qbe_with_target(
    program: &primer_ir::Program,
    target: Option<Target>,
) -> Result<String, Diagnostic> {
    if let Some(span) = super::support::first_string_span(program)
        && target.is_none()
    {
        return Err(Diagnostic::new(
            "QBE string values require an explicit --target: x86_64-unknown-linux-gnu",
            span,
        ));
    }
    let mut module = lower(program);
    module.target = target;

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
                    op: super::ir::BinaryOp::CheckedI64Add,
                    ..
                }
            )
        }));
    }

    #[test]
    fn emits_i64_add() {
        let program = compile_to_ir("x: i64 = 1 + 2; print(x);").unwrap();
        let qbe = emit_qbe(&program).unwrap();

        assert!(qbe.contains("function l $primer_i64_add"));
        assert!(qbe.contains("=l call $primer_i64_add(l 1, l 2)"));
        assert!(qbe.contains("jnz %overflow, @trap, @ok"));
        assert!(qbe.contains("storel %tmp0, %slot_x"));
        assert!(qbe.contains("call $printf(l $fmt_i64"));
    }

    #[test]
    fn emits_checked_i64_arithmetic_helpers() {
        let program = compile_to_ir(
            "value: i64 = 8;
             print(value + 1);
             print(value - 1);
             print(value * 2);
             print(value / 2);
             print(-value);",
        )
        .unwrap();
        let qbe = emit_qbe(&program).unwrap();

        for helper in [
            "$primer_i64_add",
            "$primer_i64_sub",
            "$primer_i64_mul",
            "$primer_i64_div",
            "$primer_i64_neg",
        ] {
            assert!(qbe.contains(helper));
        }
        assert!(qbe.contains("call $abort()"));
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

    #[test]
    fn lowers_product_values_to_stack_memory() {
        let program = compile_to_ir(
            "type Point { x: f64 = 0.0, y: f64, }
             point: Point = Point { y: 2.0, };
             print(point.x);",
        )
        .unwrap();
        let qbe = emit_qbe(&program).unwrap();

        assert!(qbe.contains("%slot_point =l alloc8 16"));
        assert!(qbe.contains("%slot_aggregate_tmp0 =l alloc8 16"));
        assert!(qbe.contains("blit %slot_aggregate_tmp0, %slot_point, 16"));
        assert!(qbe.contains("loadd %slot_point"));
    }

    #[test]
    fn emits_typed_functions_and_calls() {
        let program = compile_to_ir(
            "fn add(left: i64, right: i64) -> i64 { return left + right; }
             answer: i64 = add(20, 22);
             print(answer);",
        )
        .unwrap();
        let qbe = emit_qbe(&program).unwrap();

        assert!(qbe.contains("function l $primer_fn_add_0(l %arg0, l %arg1)"));
        assert!(qbe.contains("storel %arg0, %slot_left"));
        assert!(qbe.contains("call $primer_fn_add_0(l 20, l 22)"));
        assert!(qbe.contains("ret %tmp"));
    }
}
