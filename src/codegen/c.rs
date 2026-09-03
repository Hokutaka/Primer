mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
use lower::lower;

use crate::{diagnostic::Diagnostic, ir as primer_ir};

pub fn emit_c(program: &primer_ir::Program) -> Result<String, Diagnostic> {
    if let Some(diagnostic) = program.unsupported_product_type("emit-c") {
        return Err(diagnostic);
    }
    let module = lower(program);

    Ok(emit(&module))
}

#[cfg(test)]
mod tests {
    use crate::compile_to_ir;

    use super::{emit_c, ir::Statement, lower};

    #[test]
    fn lowers_binding_to_c_ir() {
        let program = compile_to_ir("x: f32 = 0.1 + 0.2;").unwrap();
        let module = lower(&program);

        assert!(matches!(
            &module.statements[0],
            Statement::Binding {
                ty: super::ir::Type::Float,
                ..
            }
        ));
    }

    #[test]
    fn emits_contextual_f32_literals() {
        let program = compile_to_ir(
            "x: f32 = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let c = emit_c(&program).unwrap();

        assert!(c.contains("float primer_x = (0.1f + 0.2f);"));
    }

    #[test]
    fn emits_contextual_f64_literals() {
        let program = compile_to_ir(
            "x: f64 = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let c = emit_c(&program).unwrap();

        assert!(c.contains("double primer_x = (0.1 + 0.2);"));
    }

    #[test]
    fn inferred_float_defaults_to_f64() {
        let program = compile_to_ir(
            "x: infer = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let c = emit_c(&program).unwrap();

        assert!(c.contains("double primer_x = (0.1 + 0.2);"));
    }

    #[test]
    fn emits_all_comparison_operators() {
        let program = compile_to_ir(
            "a: bool = 1 == 1; b: bool = 1 != 2; c: bool = 1 < 2;
             d: bool = 1 <= 2; e: bool = 2 > 1; f: bool = 2 >= 1;",
        )
        .unwrap();
        let c = emit_c(&program).unwrap();

        for operator in ["==", "!=", "<", "<=", ">", ">="] {
            assert!(c.contains(operator));
        }
    }
}
