use super::ir::{BinaryOp, Expr, ExprKind, Module, PrintFormat, Statement, Type, UnaryOp};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    if module_uses_bool(module) {
        output.push_str("#include <stdbool.h>\n");
    }

    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdio.h>\n\n");

    for definition in &module.type_definitions {
        output.push_str("typedef struct primer_type_");
        output.push_str(&definition.name);
        output.push('_');
        output.push_str(&definition.id.to_string());
        output.push_str(" {\n");
        for field in &definition.fields {
            output.push_str("    ");
            output.push_str(&c_type(field.ty, module));
            output.push(' ');
            output.push_str(&field.name);
            output.push_str(";\n");
        }
        output.push_str("} primer_type_");
        output.push_str(&definition.name);
        output.push('_');
        output.push_str(&definition.id.to_string());
        output.push_str(";\n\n");
    }

    output.push_str("int main(void) {\n");

    for statement in &module.statements {
        emit_statement(statement, 1, module, &mut output);
    }

    output.push_str("    return 0;\n");
    output.push_str("}\n");

    output
}

fn emit_statement(statement: &Statement, indent: usize, module: &Module, output: &mut String) {
    let prefix = "    ".repeat(indent);

    match statement {
        Statement::Binding { name, ty, value } => {
            output.push_str(&prefix);
            output.push_str(&c_type(*ty, module));
            output.push_str(" primer_");
            output.push_str(name);
            output.push_str(" = ");

            emit_expr(value, module, output);

            output.push_str(";\n");
        }

        Statement::Assignment { name, value } => {
            output.push_str(&prefix);
            output.push_str("primer_");
            output.push_str(name);
            output.push_str(" = ");

            emit_expr(value, module, output);

            output.push_str(";\n");
        }

        Statement::Print { format, value } => {
            emit_print(*format, value, &prefix, module, output);
        }

        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            output.push_str(&prefix);
            output.push_str("if ");
            emit_condition(condition, module, output);
            output.push_str(" {\n");

            for statement in then_body {
                emit_statement(statement, indent + 1, module, output);
            }

            output.push_str(&prefix);
            output.push('}');

            if else_body.is_empty() {
                output.push('\n');
            } else {
                output.push_str(" else {\n");
                for statement in else_body {
                    emit_statement(statement, indent + 1, module, output);
                }
                output.push_str(&prefix);
                output.push_str("}\n");
            }
        }

        Statement::While { condition, body } => {
            output.push_str(&prefix);
            output.push_str("while ");
            emit_condition(condition, module, output);
            output.push_str(" {\n");

            for statement in body {
                emit_statement(statement, indent + 1, module, output);
            }

            output.push_str(&prefix);
            output.push_str("}\n");
        }

        Statement::For {
            initializer,
            condition,
            update,
            body,
        } => {
            output.push_str(&prefix);
            output.push_str("for (");
            emit_for_clause(initializer, module, output);
            output.push_str("; ");
            emit_expr(condition, module, output);
            output.push_str("; ");
            emit_for_clause(update, module, output);
            output.push_str(") {\n");

            for statement in body {
                emit_statement(statement, indent + 1, module, output);
            }

            output.push_str(&prefix);
            output.push_str("}\n");
        }

        Statement::Break => {
            output.push_str(&prefix);
            output.push_str("break;\n");
        }

        Statement::Continue => {
            output.push_str(&prefix);
            output.push_str("continue;\n");
        }
    }
}

fn emit_for_clause(statement: &Statement, module: &Module, output: &mut String) {
    match statement {
        Statement::Binding { name, ty, value } => {
            output.push_str(&c_type(*ty, module));
            output.push_str(" primer_");
            output.push_str(name);
            output.push_str(" = ");
            emit_expr(value, module, output);
        }
        Statement::Assignment { name, value } => {
            output.push_str("primer_");
            output.push_str(name);
            output.push_str(" = ");
            emit_expr(value, module, output);
        }
        _ => unreachable!("for clauses are validated by the parser"),
    }
}

fn c_type(ty: Type, module: &Module) -> String {
    match ty {
        Type::Bool => "bool".into(),
        Type::I64 => "int64_t".into(),
        Type::Float => "float".into(),
        Type::Double => "double".into(),
        Type::Named(id) => {
            let definition = module
                .type_definitions
                .iter()
                .find(|definition| definition.id == id)
                .expect("named C type must have a definition");
            format!("primer_type_{}_{}", definition.name, id)
        }
    }
}

fn emit_print(
    format: PrintFormat,
    expr: &Expr,
    prefix: &str,
    module: &Module,
    output: &mut String,
) {
    match format {
        PrintFormat::Bool => {
            output.push_str(prefix);
            output.push_str("printf(\"%s\\n\", (");

            emit_expr(expr, module, output);

            output.push_str(") ? \"true\" : \"false\");\n");
        }

        PrintFormat::I64 => {
            output.push_str(prefix);
            output.push_str("printf(\"%lld\\n\", (long long)(");

            emit_expr(expr, module, output);

            output.push_str("));\n");
        }

        PrintFormat::F32 => {
            output.push_str(prefix);
            output.push_str("printf(\"%.9g\\n\", (double)(");

            emit_expr(expr, module, output);

            output.push_str("));\n");
        }

        PrintFormat::F64 => {
            output.push_str(prefix);
            output.push_str("printf(\"%.17g\\n\", (double)(");

            emit_expr(expr, module, output);

            output.push_str("));\n");
        }
    }
}

fn emit_expr(expr: &Expr, module: &Module, output: &mut String) {
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

        ExprKind::Construct { type_id, fields } => {
            output.push('(');
            output.push_str(&c_type(Type::Named(*type_id), module));
            output.push_str("){ ");
            for (index, field) in fields.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push('.');
                output.push_str(&field.name);
                output.push_str(" = ");
                emit_expr(&field.value, module, output);
            }
            output.push_str(" }");
        }

        ExprKind::FieldAccess { field_name, base } => {
            output.push('(');
            emit_expr(base, module, output);
            output.push_str(").");
            output.push_str(field_name);
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

            emit_expr(value, module, output);

            output.push(')');
        }

        ExprKind::Binary { op, left, right } => {
            output.push('(');

            emit_expr(left, module, output);

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

            emit_expr(right, module, output);

            output.push(')');
        }
    }
}

fn emit_condition(expr: &Expr, module: &Module, output: &mut String) {
    if matches!(&expr.kind, ExprKind::Unary { .. } | ExprKind::Binary { .. }) {
        emit_expr(expr, module, output);
    } else {
        output.push('(');
        emit_expr(expr, module, output);
        output.push(')');
    }
}

fn module_uses_bool(module: &Module) -> bool {
    module
        .type_definitions
        .iter()
        .flat_map(|definition| &definition.fields)
        .any(|field| field.ty == Type::Bool)
        || module.statements.iter().any(statement_uses_bool)
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
        Statement::For {
            initializer,
            condition,
            update,
            body,
        } => {
            statement_uses_bool(initializer)
                || condition.ty == Type::Bool
                || statement_uses_bool(update)
                || body.iter().any(statement_uses_bool)
        }
        Statement::Break | Statement::Continue => false,
    }
}
