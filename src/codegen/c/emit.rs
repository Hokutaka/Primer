use super::ir::{BinaryOp, Expr, ExprKind, Module, PrintFormat, Statement, Type, UnaryOp};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    if module_uses_bool(module) {
        output.push_str("#include <stdbool.h>\n");
    }

    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdio.h>\n\n");
    output.push_str("int main(void) {\n");

    for statement in &module.statements {
        emit_statement(statement, 1, &mut output);
    }

    output.push_str("    return 0;\n");
    output.push_str("}\n");

    output
}

fn emit_statement(statement: &Statement, indent: usize, output: &mut String) {
    let prefix = "    ".repeat(indent);

    match statement {
        Statement::Binding { name, ty, value } => {
            output.push_str(&prefix);
            output.push_str(c_type(*ty));
            output.push_str(" primer_");
            output.push_str(name);
            output.push_str(" = ");

            emit_expr(value, output);

            output.push_str(";\n");
        }

        Statement::Assignment { name, value } => {
            output.push_str(&prefix);
            output.push_str("primer_");
            output.push_str(name);
            output.push_str(" = ");

            emit_expr(value, output);

            output.push_str(";\n");
        }

        Statement::Print { format, value } => {
            emit_print(*format, value, &prefix, output);
        }

        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            output.push_str(&prefix);
            output.push_str("if ");
            emit_condition(condition, output);
            output.push_str(" {\n");

            for statement in then_body {
                emit_statement(statement, indent + 1, output);
            }

            output.push_str(&prefix);
            output.push('}');

            if else_body.is_empty() {
                output.push('\n');
            } else {
                output.push_str(" else {\n");
                for statement in else_body {
                    emit_statement(statement, indent + 1, output);
                }
                output.push_str(&prefix);
                output.push_str("}\n");
            }
        }

        Statement::While { condition, body } => {
            output.push_str(&prefix);
            output.push_str("while ");
            emit_condition(condition, output);
            output.push_str(" {\n");

            for statement in body {
                emit_statement(statement, indent + 1, output);
            }

            output.push_str(&prefix);
            output.push_str("}\n");
        }
    }
}

fn c_type(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "bool",
        Type::I64 => "int64_t",
        Type::Float => "float",
        Type::Double => "double",
    }
}

fn emit_print(format: PrintFormat, expr: &Expr, prefix: &str, output: &mut String) {
    match format {
        PrintFormat::Bool => {
            output.push_str(prefix);
            output.push_str("printf(\"%s\\n\", (");

            emit_expr(expr, output);

            output.push_str(") ? \"true\" : \"false\");\n");
        }

        PrintFormat::I64 => {
            output.push_str(prefix);
            output.push_str("printf(\"%lld\\n\", (long long)(");

            emit_expr(expr, output);

            output.push_str("));\n");
        }

        PrintFormat::F32 => {
            output.push_str(prefix);
            output.push_str("printf(\"%.9g\\n\", (double)(");

            emit_expr(expr, output);

            output.push_str("));\n");
        }

        PrintFormat::F64 => {
            output.push_str(prefix);
            output.push_str("printf(\"%.17g\\n\", (double)(");

            emit_expr(expr, output);

            output.push_str("));\n");
        }
    }
}

fn emit_expr(expr: &Expr, output: &mut String) {
    match &expr.kind {
        ExprKind::Boolean(value) => {
            output.push_str(if *value { "true" } else { "false" });
        }

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
                UnaryOp::Not => {
                    output.push('!');
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
                BinaryOp::Equal => "==",
                BinaryOp::NotEqual => "!=",
                BinaryOp::Less => "<",
                BinaryOp::LessEqual => "<=",
                BinaryOp::Greater => ">",
                BinaryOp::GreaterEqual => ">=",
            });

            output.push(' ');

            emit_expr(right, output);

            output.push(')');
        }
    }
}

fn emit_condition(expr: &Expr, output: &mut String) {
    if matches!(&expr.kind, ExprKind::Unary { .. } | ExprKind::Binary { .. }) {
        emit_expr(expr, output);
    } else {
        output.push('(');
        emit_expr(expr, output);
        output.push(')');
    }
}

fn module_uses_bool(module: &Module) -> bool {
    module.statements.iter().any(statement_uses_bool)
}

fn statement_uses_bool(statement: &Statement) -> bool {
    match statement {
        Statement::Binding { ty, value, .. } => *ty == Type::Bool || value.ty == Type::Bool,
        Statement::Assignment { value, .. } => value.ty == Type::Bool,
        Statement::Print { format, value } => {
            *format == PrintFormat::Bool || value.ty == Type::Bool
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            condition.ty == Type::Bool
                || then_body.iter().any(statement_uses_bool)
                || else_body.iter().any(statement_uses_bool)
        }
        Statement::While { condition, body } => {
            condition.ty == Type::Bool || body.iter().any(statement_uses_bool)
        }
    }
}
