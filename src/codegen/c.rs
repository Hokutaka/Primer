use crate::ast::{BinaryOp, Expr, Program, Stmt, Type, UnaryOp};
use crate::semantic::{Bindings, type_of_expr};

pub fn emit_c(program: &Program, bindings: &Bindings) -> String {
    let mut output = String::new();

    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdio.h>\n\n");
    output.push_str("int main(void) {\n");

    for statement in &program.statements {
        match statement {
            Stmt::Binding { name, value, .. } => {
                let ty = bindings
                    .get(name)
                    .copied()
                    .expect("binding must have been resolved by type checker");

                output.push_str("    ");
                output.push_str(c_type(ty));
                output.push_str(" primer_");
                output.push_str(name);
                output.push_str(" = ");

                // 左辺の型を右辺へ渡す
                emit_expr(value, Some(ty), &mut output);

                output.push_str(";\n");
            }

            Stmt::Print { value } => {
                let ty = type_of_expr(value, bindings).expect("expression must have been checked");

                emit_print(value, ty, &mut output);
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

fn emit_print(expr: &Expr, ty: Type, output: &mut String) {
    match ty {
        Type::I64 => {
            output.push_str("    printf(\"%lld\\n\", (long long)(");

            emit_expr(expr, Some(Type::I64), output);

            output.push_str("));\n");
        }

        Type::F32 => {
            output.push_str("    printf(\"%.9g\\n\", (double)(");

            emit_expr(expr, Some(Type::F32), output);

            output.push_str("));\n");
        }

        Type::F64 => {
            output.push_str("    printf(\"%.17g\\n\", (double)(");

            emit_expr(expr, Some(Type::F64), output);

            output.push_str("));\n");
        }
    }
}

fn emit_expr(expr: &Expr, expected: Option<Type>, output: &mut String) {
    match expr {
        Expr::Integer(value) => {
            output.push_str(&value.to_string());
        }

        Expr::Float {
            text,
            explicit_type,
        } => {
            output.push_str(text);

            let ty = match explicit_type {
                Some(ty) => *ty,

                None => match expected {
                    Some(Type::F32) => Type::F32,
                    _ => Type::F64,
                },
            };

            if ty == Type::F32 {
                output.push('f');
            }
        }

        Expr::Variable(name) => {
            output.push_str("primer_");
            output.push_str(name);
        }

        Expr::Unary { op, value } => {
            output.push('(');

            match op {
                UnaryOp::Negate => {
                    output.push('-');
                }
            }

            emit_expr(value, expected, output);

            output.push(')');
        }

        Expr::Binary { op, left, right } => {
            output.push('(');

            emit_expr(left, expected, output);

            output.push(' ');

            output.push_str(match op {
                BinaryOp::Add => "+",
                BinaryOp::Subtract => "-",
                BinaryOp::Multiply => "*",
                BinaryOp::Divide => "/",
            });

            output.push(' ');

            emit_expr(right, expected, output);

            output.push(')');
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::lex, parser::parse, semantic::check};

    use super::emit_c;

    #[test]
    fn emits_contextual_f32_literals() {
        let program = parse(
            lex("x: f32 = 0.1 + 0.2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let c = emit_c(&program, &bindings);

        assert!(c.contains("float primer_x = (0.1f + 0.2f);"));
    }

    #[test]
    fn emits_contextual_f64_literals() {
        let program = parse(
            lex("x: f64 = 0.1 + 0.2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let c = emit_c(&program, &bindings);

        assert!(c.contains("double primer_x = (0.1 + 0.2);"));
    }

    #[test]
    fn inferred_float_defaults_to_f64() {
        let program = parse(
            lex("x: infer = 0.1 + 0.2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let c = emit_c(&program, &bindings);

        assert!(c.contains("double primer_x = (0.1 + 0.2);"));
    }
}
