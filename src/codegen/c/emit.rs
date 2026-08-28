use super::ir::{BinaryOp, Expr, ExprKind, Module, PrintFormat, Statement, Type, UnaryOp};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdio.h>\n\n");
    output.push_str("int main(void) {\n");

    for statement in &module.statements {
        emit_statement(statement, &mut output);
    }

    output.push_str("    return 0;\n");
    output.push_str("}\n");

    output
}

fn emit_statement(statement: &Statement, output: &mut String) {
    match statement {
        Statement::Binding { name, ty, value } => {
            output.push_str("    ");
            output.push_str(c_type(*ty));
            output.push_str(" primer_");
            output.push_str(name);
            output.push_str(" = ");

            emit_expr(value, output);

            output.push_str(";\n");
        }

        Statement::Print { format, value } => {
            emit_print(*format, value, output);
        }
    }
}

fn c_type(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "int64_t",
        Type::Float => "float",
        Type::Double => "double",
    }
}

fn emit_print(format: PrintFormat, expr: &Expr, output: &mut String) {
    match format {
        PrintFormat::I64 => {
            output.push_str("    printf(\"%lld\\n\", (long long)(");

            emit_expr(expr, output);

            output.push_str("));\n");
        }

        PrintFormat::F32 => {
            output.push_str("    printf(\"%.9g\\n\", (double)(");

            emit_expr(expr, output);

            output.push_str("));\n");
        }

        PrintFormat::F64 => {
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

        ExprKind::Float { text, suffix_f32 } => {
            output.push_str(text);

            if *suffix_f32 {
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
