use crate::ir::{BinaryOp, Expr, ExprKind, Program, Statement, Type, UnaryOp};

pub fn emit_c(program: &Program) -> String {
    let mut output = String::new();

    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdio.h>\n\n");
    output.push_str("int main(void) {\n");

    for statement in &program.statements {
        match statement {
            Statement::Binding { name, ty, value } => {
                output.push_str("    ");
                output.push_str(c_type(*ty));
                output.push_str(" primer_");
                output.push_str(name);
                output.push_str(" = ");

                emit_expr(value, &mut output);

                output.push_str(";\n");
            }

            Statement::Print { value } => {
                emit_print(value, &mut output);
            }
        }
    }

    output.push_str("    return 0;\n");
    output.push_str("}\n");

    output
}

fn c_type(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "int64_t",
        Type::F32 => "float",
        Type::F64 => "double",
    }
}

fn emit_print(expr: &Expr, output: &mut String) {
    match expr.ty {
        Type::I64 => {
            output.push_str("    printf(\"%lld\\n\", (long long)(");

            emit_expr(expr, output);

            output.push_str("));\n");
        }

        Type::F32 => {
            output.push_str("    printf(\"%.9g\\n\", (double)(");

            emit_expr(expr, output);

            output.push_str("));\n");
        }

        Type::F64 => {
            output.push_str("    printf(\"%.17g\\n\", (double)(");

            emit_expr(expr, output);

            output.push_str("));\n");
        }
    }
}

fn emit_expr(expr: &Expr, output: &mut String) {
    match &expr.kind {
        ExprKind::Integer(value) => {
            output.push_str(&value.to_string());
        }

        ExprKind::Float { text } => {
            output.push_str(text);

            if expr.ty == Type::F32 {
                output.push('f');
            }
        }

        ExprKind::Variable(name) => {
            output.push_str("primer_");
            output.push_str(name);
        }

        ExprKind::Unary { op, value } => {
            output.push('(');

            match op {
                UnaryOp::Negate => {
                    output.push('-');
                }
            }

            emit_expr(value, output);

            output.push(')');
        }

        ExprKind::Binary { op, left, right } => {
            output.push('(');

            emit_expr(left, output);

            output.push(' ');

            output.push_str(match op {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
            });

            output.push(' ');

            emit_expr(right, output);

            output.push(')');
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compile_to_ir;

    use super::emit_c;

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
