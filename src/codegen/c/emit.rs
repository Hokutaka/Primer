use super::ir::{BinaryOp, Expr, ExprKind, Module, PrintFormat, Statement, Type, UnaryOp};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();
    let i64_operations = i64_operations(module);

    if module_uses_bool(module) {
        output.push_str("#include <stdbool.h>\n");
    }

    output.push_str("#include <stdint.h>\n");
    output.push_str("#include <stdio.h>\n");
    if !module.array_types.is_empty() || i64_operations.any() {
        output.push_str("#include <stdlib.h>\n");
    }
    output.push('\n');

    emit_i64_operation_support(i64_operations, &mut output);

    let mut emitted_array_types = Vec::new();
    for ty in &module.array_types {
        let Type::Array { element, .. } = ty else {
            unreachable!("array type collection only contains arrays")
        };
        if !type_uses_named(element) {
            emit_array_support_recursive(ty, module, &mut emitted_array_types, &mut output);
        }
    }

    for definition in &module.type_definitions {
        for field in &definition.fields {
            emit_array_support_recursive(&field.ty, module, &mut emitted_array_types, &mut output);
        }
        output.push_str("typedef struct primer_type_");
        output.push_str(&definition.name);
        output.push('_');
        output.push_str(&definition.id.to_string());
        output.push_str(" {\n");
        for field in &definition.fields {
            output.push_str("    ");
            output.push_str(&c_type(&field.ty, module));
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

    for ty in &module.array_types {
        emit_array_support_recursive(ty, module, &mut emitted_array_types, &mut output);
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
            .as_ref()
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
            output.push_str(&c_type(&parameter.ty, module));
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
            output.push_str(&c_type(ty, module));
            output.push_str(" primer_");
            output.push_str(name);
            output.push_str(" = ");

            emit_expr(value, module, output);

            output.push_str(";\n");
        }

        Statement::Assignment { target, value } => {
            if target.projections.is_empty() {
                output.push_str(&prefix);
                output.push_str("primer_");
                output.push_str(&target.name);
                output.push_str(" = ");
                emit_expr(value, module, output);
                output.push_str(";\n");
            } else {
                output.push_str(&prefix);
                output.push_str("{\n");
                for (index, projection) in target.projections.iter().enumerate() {
                    output.push_str(&prefix);
                    output.push_str("    ");
                    output.push_str(&c_type(&projection.element, module));
                    output.push_str(" *primer_assignment_target_");
                    output.push_str(&index.to_string());
                    output.push_str(" = ");
                    output.push_str(&array_at_name(
                        &projection.element,
                        projection.length,
                        module,
                    ));
                    output.push('(');
                    if index == 0 {
                        output.push_str("&primer_");
                        output.push_str(&target.name);
                    } else {
                        output.push_str("primer_assignment_target_");
                        output.push_str(&(index - 1).to_string());
                    }
                    output.push_str(", ");
                    emit_expr(&projection.index, module, output);
                    output.push(')');
                    output.push_str(";\n");
                }
                output.push_str(&prefix);
                output.push_str("    ");
                output.push_str(&c_type(&target.ty, module));
                output.push_str(" primer_assignment_value = ");
                emit_expr(value, module, output);
                output.push_str(";\n");
                output.push_str(&prefix);
                output.push_str("    *primer_assignment_target_");
                output.push_str(&(target.projections.len() - 1).to_string());
                output.push_str(" = primer_assignment_value;\n");
                output.push_str(&prefix);
                output.push_str("}\n");
            }
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
            output.push_str(&c_type(ty, module));
            output.push_str(" primer_");
            output.push_str(name);
            output.push_str(" = ");
            emit_expr(value, module, output);
        }
        Statement::Assignment { target, value } => {
            debug_assert!(target.projections.is_empty());
            output.push_str("primer_");
            output.push_str(&target.name);
            output.push_str(" = ");
            emit_expr(value, module, output);
        }
        _ => unreachable!("for clauses are validated by the parser"),
    }
}

fn c_type(ty: &Type, module: &Module) -> String {
    match ty {
        Type::Bool => "bool".into(),
        Type::I64 => "int64_t".into(),
        Type::Float => "float".into(),
        Type::Double => "double".into(),
        Type::Named(id) => {
            let definition = module
                .type_definitions
                .iter()
                .find(|definition| definition.id == *id)
                .expect("named C type must have a definition");
            format!("primer_type_{}_{}", definition.name, id)
        }
        Type::Array { element, length } => array_type_name(element, *length, module),
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
        ExprKind::CheckIntegerRange { value, ty } => {
            output.push_str(&format!("primer_check_{}(", ty.name()));
            emit_expr(value, module, output);
            output.push(')');
        }
        ExprKind::Boolean(value) => {
            output.push_str(if *value { "true" } else { "false" });
        }

        ExprKind::Integer(value) => {
            if *value == i64::MIN {
                output.push_str("INT64_MIN");
            } else {
                output.push_str(&value.to_string());
            }
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
            output.push_str(&c_type(&Type::Named(*type_id), module));
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
            output.push_str(&c_type(&expr.ty, module));
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
            let Type::Array { element, length } = &base.ty else {
                unreachable!("indexed expression must have an array base")
            };
            output.push_str(&array_get_name(element, *length, module));
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
            if *op == UnaryOp::CheckedI64Negate {
                output.push_str("primer_i64_neg(");
                emit_expr(value, module, output);
                output.push(')');
            } else {
                output.push('(');
                output.push_str(match op {
                    UnaryOp::CheckedI64Negate => unreachable!(),
                    UnaryOp::Negate => "-",
                    UnaryOp::Not => "!",
                });
                emit_expr(value, module, output);
                output.push(')');
            }
        }

        ExprKind::Logical { op, left, right } => {
            output.push('(');
            emit_expr(left, module, output);
            output.push_str(match op {
                super::ir::LogicalOp::And => " && ",
                super::ir::LogicalOp::Or => " || ",
            });
            emit_expr(right, module, output);
            output.push(')');
        }
        ExprKind::Binary { op, left, right } => {
            let helper = match op {
                BinaryOp::CheckedI64Add => Some("primer_i64_add"),
                BinaryOp::CheckedI64Subtract => Some("primer_i64_sub"),
                BinaryOp::CheckedI64Multiply => Some("primer_i64_mul"),
                BinaryOp::CheckedI64Divide => Some("primer_i64_div"),
                BinaryOp::Add
                | BinaryOp::Subtract
                | BinaryOp::Multiply
                | BinaryOp::Divide
                | BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual => None,
            };
            if let Some(helper) = helper {
                output.push_str(helper);
                output.push('(');
                emit_expr(left, module, output);
                output.push_str(", ");
                emit_expr(right, module, output);
                output.push(')');
                return;
            }

            output.push('(');

            emit_expr(left, module, output);

            output.push(' ');

            output.push_str(match op {
                BinaryOp::CheckedI64Add
                | BinaryOp::CheckedI64Subtract
                | BinaryOp::CheckedI64Multiply
                | BinaryOp::CheckedI64Divide => unreachable!(),
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

#[derive(Clone, Default)]
struct I64Operations {
    range_checks: std::collections::BTreeSet<crate::types::IntegerType>,
    add: bool,
    subtract: bool,
    multiply: bool,
    divide: bool,
    negate: bool,
}

impl I64Operations {
    fn any(&self) -> bool {
        !self.range_checks.is_empty()
            || self.add
            || self.subtract
            || self.multiply
            || self.divide
            || self.negate
    }

    fn include_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::CheckIntegerRange { value, ty } => {
                self.include_expr(value);
                if *ty != crate::types::IntegerType::I64 {
                    self.range_checks.insert(*ty);
                }
            }
            ExprKind::Unary { op, value } => {
                self.include_expr(value);
                if *op == UnaryOp::CheckedI64Negate {
                    self.negate = true;
                }
            }
            ExprKind::Binary { op, left, right } => {
                self.include_expr(left);
                self.include_expr(right);
                match op {
                    BinaryOp::CheckedI64Add => self.add = true,
                    BinaryOp::CheckedI64Subtract => self.subtract = true,
                    BinaryOp::CheckedI64Multiply => self.multiply = true,
                    BinaryOp::CheckedI64Divide => self.divide = true,
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::LessEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterEqual => {}
                }
            }
            ExprKind::Construct { fields, .. } => {
                for field in fields {
                    self.include_expr(&field.value);
                }
            }
            ExprKind::FieldAccess { base, .. } => self.include_expr(base),
            ExprKind::Array(values) => {
                for value in values {
                    self.include_expr(value);
                }
            }
            ExprKind::Logical {
                left: base,
                right: index,
                ..
            }
            | ExprKind::Index { base, index } => {
                self.include_expr(base);
                self.include_expr(index);
            }
            ExprKind::Call { arguments, .. } => {
                for argument in arguments {
                    self.include_expr(argument);
                }
            }
            ExprKind::Boolean(_)
            | ExprKind::Integer(_)
            | ExprKind::Float { .. }
            | ExprKind::Variable(_) => {}
        }
    }

    fn include_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Binding { value, .. } | Statement::Print { value, .. } => {
                self.include_expr(value)
            }
            Statement::Assignment { target, value } => {
                self.include_expr(value);
                for projection in &target.projections {
                    self.include_expr(&projection.index);
                }
            }
            Statement::Call { arguments, .. } => {
                for argument in arguments {
                    self.include_expr(argument);
                }
            }
            Statement::Return(value) => {
                if let Some(value) = value {
                    self.include_expr(value);
                }
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.include_expr(condition);
                for statement in then_body.iter().chain(else_body) {
                    self.include_statement(statement);
                }
            }
            Statement::While { condition, body } => {
                self.include_expr(condition);
                for statement in body {
                    self.include_statement(statement);
                }
            }
            Statement::For {
                initializer,
                condition,
                update,
                body,
            } => {
                self.include_statement(initializer);
                self.include_expr(condition);
                self.include_statement(update);
                for statement in body {
                    self.include_statement(statement);
                }
            }
            Statement::Break | Statement::Continue => {}
        }
    }
}

fn i64_operations(module: &Module) -> I64Operations {
    let mut operations = I64Operations::default();
    for statement in &module.statements {
        operations.include_statement(statement);
    }
    for function in &module.functions {
        for statement in &function.body {
            operations.include_statement(statement);
        }
    }
    operations
}

fn emit_i64_operation_support(operations: I64Operations, output: &mut String) {
    for ty in &operations.range_checks {
        output.push_str(&format!("static int64_t primer_check_{}(int64_t value) {{\n    if (value < {}LL || value > {}LL) abort();\n    return value;\n}}\n\n", ty.name(), ty.minimum(), ty.maximum()));
    }

    if !operations.any() {
        return;
    }

    output.push_str("static void primer_integer_overflow(void) {\n");
    output.push_str(
        "    fputs(\"primer: integer operation produced a value outside the supported range\\n\", stderr);\n",
    );
    output.push_str("    abort();\n}\n\n");

    if operations.add {
        output.push_str("static int64_t primer_i64_add(int64_t left, int64_t right) {\n");
        output.push_str("    if ((right > 0 && left > INT64_MAX - right) ||\n");
        output.push_str("        (right < 0 && left < INT64_MIN - right)) {\n");
        output.push_str("        primer_integer_overflow();\n    }\n");
        output.push_str("    return left + right;\n}\n\n");
    }

    if operations.subtract {
        output.push_str("static int64_t primer_i64_sub(int64_t left, int64_t right) {\n");
        output.push_str("    if ((right < 0 && left > INT64_MAX + right) ||\n");
        output.push_str("        (right > 0 && left < INT64_MIN + right)) {\n");
        output.push_str("        primer_integer_overflow();\n    }\n");
        output.push_str("    return left - right;\n}\n\n");
    }

    if operations.multiply {
        output.push_str("static int64_t primer_i64_mul(int64_t left, int64_t right) {\n");
        output.push_str("    if ((left > 0 && right > 0 && left > INT64_MAX / right) ||\n");
        output.push_str("        (left > 0 && right < 0 && right < INT64_MIN / left) ||\n");
        output.push_str("        (left < 0 && right > 0 && left < INT64_MIN / right) ||\n");
        output.push_str("        (left < 0 && right < 0 && left < INT64_MAX / right)) {\n");
        output.push_str("        primer_integer_overflow();\n    }\n");
        output.push_str("    return left * right;\n}\n\n");
    }

    if operations.divide {
        output.push_str("static int64_t primer_i64_div(int64_t left, int64_t right) {\n");
        output.push_str("    if (right == 0) {\n");
        output
            .push_str("        fputs(\"primer: cannot divide an integer by zero\\n\", stderr);\n");
        output.push_str("        abort();\n    }\n");
        output.push_str("    if (left == INT64_MIN && right == -1) {\n");
        output.push_str("        primer_integer_overflow();\n    }\n");
        output.push_str("    return left / right;\n}\n\n");
    }

    if operations.negate {
        output.push_str("static int64_t primer_i64_neg(int64_t value) {\n");
        output.push_str("    if (value == INT64_MIN) {\n");
        output.push_str("        primer_integer_overflow();\n    }\n");
        output.push_str("    return -value;\n}\n\n");
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
        .any(|field| type_uses_bool(&field.ty))
        || module.array_types.iter().any(type_uses_bool)
        || module.statements.iter().any(statement_uses_bool)
        || module.functions.iter().any(|function| {
            function.return_type.as_ref().is_some_and(type_uses_bool)
                || function
                    .parameters
                    .iter()
                    .any(|parameter| type_uses_bool(&parameter.ty))
                || function.body.iter().any(statement_uses_bool)
        })
}

fn type_uses_bool(ty: &Type) -> bool {
    match ty {
        Type::Bool => true,
        Type::Array { element, .. } => type_uses_bool(element),
        Type::I64 | Type::Float | Type::Double | Type::Named(_) => false,
    }
}

fn type_uses_named(ty: &Type) -> bool {
    match ty {
        Type::Named(_) => true,
        Type::Array { element, .. } => type_uses_named(element),
        Type::Bool | Type::I64 | Type::Float | Type::Double => false,
    }
}

fn emit_array_support_recursive(
    ty: &Type,
    module: &Module,
    emitted: &mut Vec<Type>,
    output: &mut String,
) {
    let Type::Array { element, .. } = ty else {
        return;
    };

    emit_array_support_recursive(element, module, emitted, output);
    if !emitted.contains(ty) {
        emit_array_support(ty, module, output);
        emitted.push(ty.clone());
    }
}

fn emit_array_support(ty: &Type, module: &Module, output: &mut String) {
    let Type::Array { element, length } = ty else {
        unreachable!("array support requires an array type")
    };
    let name = array_type_name(element, *length, module);
    output.push_str("typedef struct ");
    output.push_str(&name);
    output.push_str(" {\n    ");
    output.push_str(&c_type(element, module));
    output.push_str(" items[");
    output.push_str(&length.to_string());
    output.push_str("];\n} ");
    output.push_str(&name);
    output.push_str(";\n\n");

    output.push_str("static ");
    output.push_str(&c_type(element, module));
    output.push(' ');
    output.push_str(&array_get_name(element, *length, module));
    output.push('(');
    output.push_str(&name);
    output.push_str(" value, int64_t index) {\n");
    output.push_str("    if (index < 0 || index >= ");
    output.push_str(&length.to_string());
    output.push_str(") {\n");
    output.push_str("        fputs(\"primer: array index out of bounds\\n\", stderr);\n");
    output.push_str("        abort();\n    }\n");
    output.push_str("    return value.items[index];\n}\n\n");

    if module.array_assignment_types.contains(ty) {
        output.push_str("static ");
        output.push_str(&c_type(element, module));
        output.push_str(" *");
        output.push_str(&array_at_name(element, *length, module));
        output.push('(');
        output.push_str(&name);
        output.push_str(" *value, int64_t index) {\n");
        output.push_str("    if (index < 0 || index >= ");
        output.push_str(&length.to_string());
        output.push_str(") {\n");
        output.push_str("        fputs(\"primer: array index out of bounds\\n\", stderr);\n");
        output.push_str("        abort();\n    }\n");
        output.push_str("    return &value->items[index];\n}\n\n");
    }
}

fn array_type_name(element: &Type, length: usize, module: &Module) -> String {
    format!(
        "primer_array_{}_{}",
        array_element_name(element, module),
        length
    )
}

fn array_get_name(element: &Type, length: usize, module: &Module) -> String {
    format!(
        "primer_array_get_{}_{}",
        array_element_name(element, module),
        length
    )
}

fn array_at_name(element: &Type, length: usize, module: &Module) -> String {
    format!(
        "primer_array_at_{}_{}",
        array_element_name(element, module),
        length
    )
}

fn array_element_name(element: &Type, module: &Module) -> String {
    match element {
        Type::Bool => "bool".into(),
        Type::I64 => "i64".into(),
        Type::Float => "f32".into(),
        Type::Double => "f64".into(),
        Type::Named(id) => {
            let definition = module
                .type_definitions
                .iter()
                .find(|definition| definition.id == *id)
                .expect("C IR should contain the referenced product type");
            format!("type_{}_{}", definition.name, id)
        }
        Type::Array { element, length } => {
            format!("array_{}_{}", array_element_name(element, module), length)
        }
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
