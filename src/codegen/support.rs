use crate::{
    diagnostic::Diagnostic,
    ir::{
        AssignmentProjection, Expr, ExprKind, Program, ReturnType, Statement, StatementKind, Type,
    },
    source::Span,
};

/// 未対応の出力先へ文字列を渡さず、lowering前にソース位置付きで報告します。
pub(super) fn reject_strings(program: &Program, route: &str) -> Result<(), Diagnostic> {
    let check = |span| {
        Diagnostic::new(
            format!("string values are not supported by `{route}` yet"),
            span,
        )
    };
    for definition in &program.type_definitions {
        for field in &definition.fields {
            if contains_string(&field.ty) {
                return Err(check(field.span));
            }
            if let Some(span) = field.default.as_ref().and_then(string_expr) {
                return Err(check(span));
            }
        }
    }
    for function in &program.function_definitions {
        for parameter in &function.parameters {
            if contains_string(&parameter.ty) {
                return Err(check(parameter.span));
            }
        }
        if matches!(&function.return_type, ReturnType::Value(ty) if contains_string(ty)) {
            return Err(check(function.span));
        }
        if let Some(span) = string_statements(&function.body) {
            return Err(check(span));
        }
    }
    if let Some(span) = string_statements(&program.statements) {
        return Err(check(span));
    }
    Ok(())
}

fn contains_string(ty: &Type) -> bool {
    match ty {
        Type::String => true,
        Type::Array { element, .. } => contains_string(element),
        // 名前付き型のフィールドは、未使用の定義も含めて入口で検査します。
        Type::Named(_) | Type::Bool | Type::Integer(_) | Type::F32 | Type::F64 => false,
    }
}

fn string_statements(statements: &[Statement]) -> Option<Span> {
    statements.iter().find_map(string_statement)
}

fn string_statement(statement: &Statement) -> Option<Span> {
    match &statement.kind {
        StatementKind::Binding { ty, value, .. } => {
            string_expr(value).or_else(|| contains_string(ty).then_some(statement.span))
        }
        StatementKind::Assignment { target, value } => string_expr(value)
            .or_else(|| {
                target
                    .projections
                    .iter()
                    .find_map(|projection| match projection {
                        AssignmentProjection::Index {
                            index,
                            element,
                            span,
                            ..
                        } => {
                            string_expr(index).or_else(|| contains_string(element).then_some(*span))
                        }
                    })
            })
            .or_else(|| {
                (contains_string(&target.ty) || contains_string(&target.root_ty))
                    .then_some(statement.span)
            }),
        StatementKind::Print { value } => string_expr(value),
        StatementKind::Call { arguments, .. } => arguments.iter().find_map(string_expr),
        StatementKind::Return { value } => value.as_ref().and_then(string_expr),
        StatementKind::If {
            condition,
            then_body,
            else_body,
        } => string_expr(condition)
            .or_else(|| string_statements(then_body))
            .or_else(|| string_statements(else_body)),
        StatementKind::While { condition, body } => {
            string_expr(condition).or_else(|| string_statements(body))
        }
        StatementKind::For {
            initializer,
            condition,
            update,
            body,
        } => string_statement(initializer)
            .or_else(|| string_expr(condition))
            .or_else(|| string_statement(update))
            .or_else(|| string_statements(body)),
        StatementKind::Break | StatementKind::Continue => None,
    }
}

fn string_expr(expr: &Expr) -> Option<Span> {
    if contains_string(&expr.ty) || matches!(expr.kind, ExprKind::String(_)) {
        return Some(expr.span);
    }
    match &expr.kind {
        ExprKind::ConvertNumeric { value, .. }
        | ExprKind::ConvertInteger { value, .. }
        | ExprKind::Unary { value, .. }
        | ExprKind::FieldAccess { base: value, .. } => string_expr(value),
        ExprKind::Logical { left, right, .. }
        | ExprKind::Binary { left, right, .. }
        | ExprKind::Index {
            base: left,
            index: right,
        } => string_expr(left).or_else(|| string_expr(right)),
        ExprKind::Construct { fields, .. } => {
            fields.iter().find_map(|field| string_expr(&field.value))
        }
        ExprKind::Array(values)
        | ExprKind::Call {
            arguments: values, ..
        } => values.iter().find_map(string_expr),
        ExprKind::Boolean(_)
        | ExprKind::Integer(_)
        | ExprKind::Float { .. }
        | ExprKind::Variable { .. }
        | ExprKind::String(_) => None,
    }
}
