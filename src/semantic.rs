use std::collections::HashMap;

use crate::{
    ast::{BinaryOp, Expr, ExprKind, Program, StmtKind, Type, TypeSpec},
    diagnostic::Diagnostic,
};

pub type Bindings = HashMap<String, Type>;
type SemanticResult<T> = Result<T, Diagnostic>;

pub fn check(program: &Program) -> SemanticResult<Bindings> {
    let mut bindings = HashMap::new();

    for statement in &program.statements {
        match &statement.kind {
            StmtKind::Binding {
                name,
                type_spec,
                value,
            } => {
                if bindings.contains_key(name) {
                    return Err(Diagnostic::new(
                        format!("duplicate binding `{name}`"),
                        statement.span,
                    ));
                }

                let value_type = match type_spec {
                    TypeSpec::Explicit(expected) => {
                        // 左辺の明示型を右辺へ渡す
                        let actual = type_of_expr_expected(value, &bindings, Some(*expected))?;

                        if actual != *expected {
                            return Err(Diagnostic::new(
                                format!(
                                    "type mismatch for `{name}`: expected {}, found {}",
                                    type_name(*expected),
                                    type_name(actual),
                                ),
                                value.span,
                            ));
                        }

                        *expected
                    }

                    TypeSpec::Infer => {
                        // inferの場合は文脈なしで推論
                        type_of_expr_expected(value, &bindings, None)?
                    }
                };

                bindings.insert(name.clone(), value_type);
            }

            StmtKind::Print { value } => {
                type_of_expr_expected(value, &bindings, None)?;
            }
        }
    }

    Ok(bindings)
}

pub fn type_of_expr(expr: &Expr, bindings: &Bindings) -> SemanticResult<Type> {
    type_of_expr_expected(expr, bindings, None)
}

pub(crate) fn type_of_expr_expected(
    expr: &Expr,
    bindings: &Bindings,
    expected: Option<Type>,
) -> SemanticResult<Type> {
    match &expr.kind {
        ExprKind::Integer(_) => Ok(Type::I64),

        ExprKind::Float { explicit_type, .. } => {
            // suffix付きなら絶対その型
            if let Some(ty) = explicit_type {
                return Ok(*ty);
            }

            // suffixなしなら文脈を見る
            match expected {
                Some(Type::F32) => Ok(Type::F32),
                Some(Type::F64) => Ok(Type::F64),

                // 文脈なしのfloatはf64
                _ => Ok(Type::F64),
            }
        }

        ExprKind::Variable(name) => bindings
            .get(name)
            .copied()
            .ok_or_else(|| Diagnostic::new(format!("unknown binding `{name}`"), expr.span)),

        ExprKind::Unary { value, .. } => type_of_expr_expected(value, bindings, expected),

        ExprKind::Binary { op, left, right } => {
            // 親から来た期待型を左右両方へ伝える
            let left_type = type_of_expr_expected(left, bindings, expected)?;

            let right_type = type_of_expr_expected(right, bindings, expected)?;

            if left_type != right_type {
                return Err(Diagnostic::new(
                    format!(
                        "cannot apply `{}` to {} and {}",
                        operator_name(*op),
                        type_name(left_type),
                        type_name(right_type),
                    ),
                    expr.span,
                ));
            }

            Ok(left_type)
        }
    }
}

fn type_name(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
    }
}

fn operator_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
    }
}

#[cfg(test)]
mod tests {
    use crate::{ast::Type, lexer::lex, parser::parse, source::Span};

    use super::check;

    #[test]
    fn explicit_f32_guides_float_literals() {
        let program = parse(lex("x: f32 = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x"), Some(&Type::F32));
    }

    #[test]
    fn explicit_f64_guides_float_literals() {
        let program = parse(lex("x: f64 = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x"), Some(&Type::F64));
    }

    #[test]
    fn infer_defaults_float_to_f64() {
        let program = parse(lex("x: infer = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x"), Some(&Type::F64));
    }

    #[test]
    fn suffix_can_force_f32() {
        let program = parse(lex("x: infer = 0.1f32 + 0.2f32;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x"), Some(&Type::F32));
    }

    #[test]
    fn rejects_integer_float_mix() {
        let program = parse(lex("x: infer = 1 + 0.1;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "cannot apply `+` to i64 and f64");
        assert_eq!(error.primary_span(), Some(Span::new(11, 18)));
    }

    #[test]
    fn reports_unknown_binding_at_variable() {
        let program = parse(lex("print(missing);").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "unknown binding `missing`");
        assert_eq!(error.primary_span(), Some(Span::new(6, 13)));
    }

    #[test]
    fn reports_duplicate_binding_at_statement() {
        let program = parse(lex("x: i64 = 1; x: i64 = 2;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "duplicate binding `x`");
        assert_eq!(error.primary_span(), Some(Span::new(12, 23)));
    }

    #[test]
    fn reports_binding_type_mismatch_at_value() {
        let program = parse(lex("x: f32 = 1;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "type mismatch for `x`: expected f32, found i64"
        );
        assert_eq!(error.primary_span(), Some(Span::new(9, 10)));
    }
}
