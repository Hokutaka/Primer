use super::ir::{
    ArrayElementType, BinaryOp, Expr, ExprKind, Module, PrintFormat, Statement, Type, UnaryOp,
};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    if module_uses_bool(module) {
        output.push_str("#include <stdbool.h>\n");
    }

    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdio.h>\n");
    if !module.array_types.is_empty() {
        output.push_str("#include <stdlib.h>\n");
    }
    output.push('\n');

    for ty in &module.array_types {
        let Type::Array { element, length } = *ty else {
            unreachable!("array type collection only contains arrays")
        };
        let name = array_type_name(element, length);
        output.push_str("typedef struct ");
        output.push_str(&name);
        output.push_str(" {\n    ");
        output.push_str(array_element_c_type(element));
        output.push_str(" items[");
        output.push_str(&length.to_string());
        output.push_str("];\n} ");
        output.push_str(&name);
        output.push_str(";\n\n");

        output.push_str("static ");
        output.push_str(array_element_c_type(element));
        output.push(' ');
        output.push_str(&array_get_name(element, length));
        output.push('(');
        output.push_str(&name);
        output.push_str(" value, int64_t index) {\n");
        output.push_str("    if (index < 0 || index >= ");
        output.push_str(&length.to_string());
        output.push_str(") {\n");
        output.push_str("        fputs(\"primer: array index out of bounds\\n\", stderr);\n");
        output.push_str("        abort();\n    }\n");
        output.push_str("    return value.items[index];\n}\n\n");
    }

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

    for function in &module.functions {
        emit_function_signature(function, module, &mut output);
        output.push_str(";\n");
    }
    if !module.functions.is_empty() {
        output.push('\n');
    }

    for function in &module.functions {
        emit_function_signature(function, module, &mut output);
        output.push_str(" {\n");
        for statement in &function.body {
            emit_statement(statement, 1, module, &mut output);
        }
        output.push_str("}\n\n");
    }

    output.push_str("int main(void) {\n");

    for statement in &module.statements {
        emit_statement(statement, 1, module, &mut output);
    }

    if let Some(function_id) = module.explicit_main {
        let function = &module.functions[function_id];
        output.push_str("    ");
        output.push_str(&function_name(function.id, &function.name));
        output.push_str("();\n");
    }

    output.push_str("    return 0;\n");
    output.push_str("}\n");

    output
}

fn emit_function_signature(function: &super::ir::Function, module: &Module, output: &mut String) {
    output.push_str(
        &function
            .return_type
            .map_or_else(|| "void".into(), |ty| c_type(ty, module)),
    );
    output.push(' ');
    output.push_str(&function_name(function.id, &function.name));
    output.push('(');
    if function.parameters.is_empty() {
        output.push_str("void");
    } else {
        for (index, parameter) in function.parameters.iter().enumerate() {
            if index > 0 {
                output.push_str(", ");
            }
            output.push_str(&c_type(parameter.ty, module));
            output.push_str(" primer_");
            output.push_str(&parameter.name);
        }
    }
    output.push(')');
}

fn function_name(id: usize, name: &str) -> String {
    format!("primer_fn_{name}_{id}")
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

        Statement::Call {
            function_id,
            function_name,
            arguments,
        } => {
            output.push_str(&prefix);
            emit_call(*function_id, function_name, arguments, module, output);
            output.push_str(";\n");
        }

        Statement::Return(value) => {
            output.push_str(&prefix);
            output.push_str("return");
            if let Some(value) = value {
                output.push(' ');
                emit_expr(value, module, output);
            }
            output.push_str(";\n");
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
        Type::Array { element, length } => array_type_name(element, length),
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

        ExprKind::Array(values) => {
            output.push('(');
            output.push_str(&c_type(expr.ty, module));
            output.push_str("){ .items = { ");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                emit_expr(value, module, output);
            }
            output.push_str(" } }");
        }

        ExprKind::Index { base, index } => {
            let Type::Array { element, length } = base.ty else {
                unreachable!("indexed expression must have an array base")
            };
            output.push_str(&array_get_name(element, length));
            output.push('(');
            emit_expr(base, module, output);
            output.push_str(", ");
            emit_expr(index, module, output);
            output.push(')');
        }

        ExprKind::Call {
            function_id,
            function_name,
            arguments,
        } => emit_call(*function_id, function_name, arguments, module, output),

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

fn emit_call(
    function_id: usize,
    name: &str,
    arguments: &[Expr],
    module: &Module,
    output: &mut String,
) {
    output.push_str(&function_name(function_id, name));
    output.push('(');
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        emit_expr(argument, module, output);
    }
    output.push(')');
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
        .any(|field| type_uses_bool(field.ty))
        || module.array_types.iter().copied().any(type_uses_bool)
        || module.statements.iter().any(statement_uses_bool)
        || module.functions.iter().any(|function| {
            function.return_type.is_some_and(type_uses_bool)
                || function
                    .parameters
                    .iter()
                    .any(|parameter| type_uses_bool(parameter.ty))
                || function.body.iter().any(statement_uses_bool)
        })
}

fn type_uses_bool(ty: Type) -> bool {
    ty == Type::Bool
        || matches!(
            ty,
            Type::Array {
                element: ArrayElementType::Bool,
                ..
            }
        )
}

const fn array_element_c_type(element: ArrayElementType) -> &'static str {
    match element {
        ArrayElementType::Bool => "bool",
        ArrayElementType::I64 => "int64_t",
        ArrayElementType::Float => "float",
        ArrayElementType::Double => "double",
    }
}

fn array_type_name(element: ArrayElementType, length: usize) -> String {
    format!("primer_array_{}_{}", array_element_name(element), length)
}

fn array_get_name(element: ArrayElementType, length: usize) -> String {
    format!(
        "primer_array_get_{}_{}",
        array_element_name(element),
        length
    )
}

const fn array_element_name(element: ArrayElementType) -> &'static str {
    match element {
        ArrayElementType::Bool => "bool",
        ArrayElementType::I64 => "i64",
        ArrayElementType::Float => "f32",
        ArrayElementType::Double => "f64",
    }
}

fn statement_uses_bool(statement: &Statement) -> bool {
    match statement {
        Statement::Binding { ty, value, .. } => *ty == Type::Bool || value.ty == Type::Bool,
        Statement::Assignment { value, .. } => value.ty == Type::Bool,
        Statement::Print { format, value } => {
            *format == PrintFormat::Bool || value.ty == Type::Bool
        }
        Statement::Call { arguments, .. } => arguments.iter().any(|value| value.ty == Type::Bool),
        Statement::Return(value) => value.as_ref().is_some_and(|value| value.ty == Type::Bool),
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
