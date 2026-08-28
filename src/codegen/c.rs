mod emit;
pub mod ir;
mod lower;

pub use emit::emit;
pub use lower::lower;

use crate::ir as primer_ir;

pub fn emit_c(program: &primer_ir::Program) -> String {
    let module = lower(program);

    emit(&module)
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

        let c = emit_c(&program);

        assert!(c.contains("float primer_x = (0.1f + 0.2f);"));
    }

    #[test]
    fn emits_contextual_f64_literals() {
        let program = compile_to_ir(
            "x: f64 = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let c = emit_c(&program);

        assert!(c.contains("double primer_x = (0.1 + 0.2);"));
    }

    #[test]
    fn inferred_float_defaults_to_f64() {
        let program = compile_to_ir(
            "x: infer = 0.1 + 0.2;
             print(x);",
        )
        .unwrap();

        let c = emit_c(&program);

        assert!(c.contains("double primer_x = (0.1 + 0.2);"));
    }
}
