use std::collections::HashMap;

use crate::{
    ast::{BinaryOp, Expr, ExprKind, Program, Stmt, StmtKind, Type, TypeSpec},
    diagnostic::Diagnostic,
};

pub type Bindings = HashMap<String, BindingInfo>;
type SemanticResult<T> = Result<T, Diagnostic>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingInfo {
    pub ty: Type,
    pub mutable: bool,
}

pub fn check(program: &Program) -> SemanticResult<Bindings> {
    let mut scopes = vec![HashMap::new()];
    check_statements(&program.statements, &mut scopes, 0)?;
    Ok(scopes.pop().expect("top-level scope must exist"))
}

fn check_statements(
    statements: &[Stmt],
    scopes: &mut Vec<Bindings>,
    loop_depth: usize,
) -> SemanticResult<()> {
    for statement in statements {
        let bindings = visible_bindings(scopes);

        match &statement.kind {
            StmtKind::Binding {
                mutable,
                name,
                type_spec,
                value,
            } => {
                if scopes
                    .last()
                    .expect("current scope must exist")
                    .contains_key(name)
                {
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

                scopes.last_mut().expect("current scope must exist").insert(
                    name.clone(),
                    BindingInfo {
                        ty: value_type,
                        mutable: *mutable,
                    },
                );
            }

            StmtKind::Assignment {
                name,
                name_span,
                value,
            } => {
                let binding = bindings.get(name).copied().ok_or_else(|| {
                    Diagnostic::new(format!("unknown binding `{name}`"), *name_span)
                })?;

                if !binding.mutable {
                    return Err(Diagnostic::new(
                        format!("cannot assign to immutable binding `{name}`"),
                        *name_span,
                    ));
                }

                let actual = type_of_expr_expected(value, &bindings, Some(binding.ty))?;

                if actual != binding.ty {
                    return Err(Diagnostic::new(
                        format!(
                            "type mismatch for assignment to `{name}`: expected {}, found {}",
                            type_name(binding.ty),
                            type_name(actual),
                        ),
                        value.span,
                    ));
                }
            }

            StmtKind::Print { value } => {
                type_of_expr_expected(value, &bindings, None)?;
            }

            StmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition_ty = type_of_expr_expected(condition, &bindings, Some(Type::Bool))?;

                if condition_ty != Type::Bool {
                    return Err(Diagnostic::new(
                        format!(
                            "if condition must be bool, found {}",
                            type_name(condition_ty)
                        ),
                        condition.span,
                    ));
                }

                scopes.push(HashMap::new());
                check_statements(then_body, scopes, loop_depth)?;
                scopes.pop();

                scopes.push(HashMap::new());
                check_statements(else_body, scopes, loop_depth)?;
                scopes.pop();
            }

            StmtKind::While { condition, body } => {
                let condition_ty = type_of_expr_expected(condition, &bindings, Some(Type::Bool))?;

                if condition_ty != Type::Bool {
                    return Err(Diagnostic::new(
                        format!(
                            "while condition must be bool, found {}",
                            type_name(condition_ty)
                        ),
                        condition.span,
                    ));
                }

                scopes.push(HashMap::new());
                check_statements(body, scopes, loop_depth + 1)?;
                scopes.pop();
            }

            StmtKind::Break => {
                if loop_depth == 0 {
                    return Err(Diagnostic::new(
                        "break can only be used inside a loop",
                        statement.span,
                    ));
                }
            }

            StmtKind::Continue => {
                if loop_depth == 0 {
                    return Err(Diagnostic::new(
                        "continue can only be used inside a loop",
                        statement.span,
                    ));
                }
            }
        }
    }

    Ok(())
}

fn visible_bindings(scopes: &[Bindings]) -> Bindings {
    let mut visible = HashMap::new();

    for scope in scopes {
        visible.extend(scope.iter().map(|(name, binding)| (name.clone(), *binding)));
    }

    visible
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
        ExprKind::Boolean(_) => Ok(Type::Bool),

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
            .map(|binding| binding.ty)
            .ok_or_else(|| Diagnostic::new(format!("unknown binding `{name}`"), expr.span)),

        ExprKind::Unary { op, value } => match op {
            crate::ast::UnaryOp::Negate => {
                let ty = type_of_expr_expected(value, bindings, expected)?;

                if !is_numeric(ty) {
                    return Err(Diagnostic::new(
                        format!("cannot apply `-` to {}", type_name(ty)),
                        expr.span,
                    ));
                }

                Ok(ty)
            }

            crate::ast::UnaryOp::Not => {
                let ty = type_of_expr_expected(value, bindings, Some(Type::Bool))?;

                if ty != Type::Bool {
                    return Err(Diagnostic::new(
                        format!("cannot apply `!` to {}", type_name(ty)),
                        expr.span,
                    ));
                }

                Ok(Type::Bool)
            }
        },

        ExprKind::Binary { op, left, right } => {
            let (left_type, right_type) = if is_comparison(*op) {
                comparison_operand_types(left, right, bindings)?
            } else {
                // 親から来た期待型を左右両方へ伝える
                (
                    type_of_expr_expected(left, bindings, expected)?,
                    type_of_expr_expected(right, bindings, expected)?,
                )
            };

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

            if is_arithmetic(*op) {
                if !is_numeric(left_type) {
                    return Err(Diagnostic::new(
                        format!(
                            "cannot apply `{}` to {}",
                            operator_name(*op),
                            type_name(left_type),
                        ),
                        expr.span,
                    ));
                }

                Ok(left_type)
            } else if is_ordering(*op) {
                if !is_numeric(left_type) {
                    return Err(Diagnostic::new(
                        format!(
                            "cannot apply `{}` to {}",
                            operator_name(*op),
                            type_name(left_type),
                        ),
                        expr.span,
                    ));
                }

                Ok(Type::Bool)
            } else {
                Ok(Type::Bool)
            }
        }
    }
}

pub(crate) fn comparison_operand_types(
    left: &Expr,
    right: &Expr,
    bindings: &Bindings,
) -> SemanticResult<(Type, Type)> {
    let left_type = type_of_expr_expected(left, bindings, None)?;
    let right_type = type_of_expr_expected(right, bindings, None)?;

    if left_type == right_type {
        return Ok((left_type, right_type));
    }

    let contextual_left = type_of_expr_expected(left, bindings, Some(right_type))?;

    if contextual_left == right_type {
        return Ok((contextual_left, right_type));
    }

    let contextual_right = type_of_expr_expected(right, bindings, Some(left_type))?;

    Ok((left_type, contextual_right))
}

fn type_name(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "bool",
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
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::Greater => ">",
        BinaryOp::GreaterEqual => ">=",
    }
}

const fn is_numeric(ty: Type) -> bool {
    matches!(ty, Type::I64 | Type::F32 | Type::F64)
}

const fn is_arithmetic(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide
    )
}

const fn is_ordering(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual
    )
}

const fn is_comparison(op: BinaryOp) -> bool {
    !is_arithmetic(op)
}

#[cfg(test)]
mod tests {
    use crate::{ast::Type, lexer::lex, parser::parse, source::Span};

    use super::check;

    #[test]
    fn explicit_f32_guides_float_literals() {
        let program = parse(lex("x: f32 = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::F32));
    }

    #[test]
    fn explicit_f64_guides_float_literals() {
        let program = parse(lex("x: f64 = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::F64));
    }

    #[test]
    fn infer_defaults_float_to_f64() {
        let program = parse(lex("x: infer = 0.1 + 0.2;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::F64));
    }

    #[test]
    fn suffix_can_force_f32() {
        let program = parse(lex("x: infer = 0.1f32 + 0.2f32;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::F32));
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

    #[test]
    fn accepts_assignment_to_mutable_binding() {
        let program = parse(lex("mut x: i64 = 1; x = x + 1;").unwrap()).unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(bindings.get("x").map(|binding| binding.ty), Some(Type::I64));
        assert_eq!(bindings.get("x").map(|binding| binding.mutable), Some(true));
    }

    #[test]
    fn rejects_assignment_to_immutable_binding() {
        let program = parse(lex("x: i64 = 1; x = 2;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "cannot assign to immutable binding `x`");
        assert_eq!(error.primary_span(), Some(Span::new(12, 13)));
    }

    #[test]
    fn rejects_assignment_to_unknown_binding() {
        let program = parse(lex("missing = 1;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "unknown binding `missing`");
        assert_eq!(error.primary_span(), Some(Span::new(0, 7)));
    }

    #[test]
    fn rejects_assignment_with_different_type() {
        let program = parse(lex("mut x: i64 = 1; x = 0.5;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(
            error.message(),
            "type mismatch for assignment to `x`: expected i64, found f64"
        );
        assert_eq!(error.primary_span(), Some(Span::new(20, 23)));
    }

    #[test]
    fn checks_boolean_and_numeric_comparisons() {
        let program =
            parse(lex("a: bool = true == !false; b: bool = 1 < 2; c: bool = 0.1 < 0.2;").unwrap())
                .unwrap();

        let bindings = check(&program).unwrap();

        assert_eq!(
            bindings.get("a").map(|binding| binding.ty),
            Some(Type::Bool)
        );
        assert_eq!(
            bindings.get("b").map(|binding| binding.ty),
            Some(Type::Bool)
        );
        assert_eq!(
            bindings.get("c").map(|binding| binding.ty),
            Some(Type::Bool)
        );
    }

    #[test]
    fn comparison_uses_float_context_from_either_operand() {
        let program =
            parse(lex("value: f32 = 0.5; a: bool = value < 1.0; b: bool = 0.1 < value;").unwrap())
                .unwrap();

        check(&program).unwrap();
    }

    #[test]
    fn rejects_arithmetic_on_booleans() {
        let program = parse(lex("value: bool = true + false;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "cannot apply `+` to bool");
    }

    #[test]
    fn accepts_shadowing_in_nested_blocks() {
        let program =
            parse(lex("x: i64 = 1; if true { x: bool = false; print(x); } print(x);").unwrap())
                .unwrap();

        check(&program).unwrap();
    }

    #[test]
    fn rejects_block_binding_outside_its_scope() {
        let program = parse(lex("if true { local: i64 = 1; } print(local);").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "unknown binding `local`");
    }

    #[test]
    fn requires_boolean_if_condition() {
        let program = parse(lex("if 1 { print(1); }").unwrap()).unwrap();

        let error = check(&program).unwrap_err();

        assert_eq!(error.message(), "if condition must be bool, found i64");
    }

    #[test]
    fn requires_boolean_while_condition() {
        let program =
            crate::parser::parse(crate::lexer::lex("while 1 { print(1); }").unwrap()).unwrap();

        let error = check(&program).unwrap_err();
        assert_eq!(error.message(), "while condition must be bool, found i64");
    }

    #[test]
    fn rejects_break_outside_loop() {
        let program = crate::parser::parse(crate::lexer::lex("break;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();
        assert_eq!(error.message(), "break can only be used inside a loop");
    }

    #[test]
    fn rejects_continue_outside_loop() {
        let program = crate::parser::parse(crate::lexer::lex("continue;").unwrap()).unwrap();

        let error = check(&program).unwrap_err();
        assert_eq!(error.message(), "continue can only be used inside a loop");
    }

    #[test]
    fn accepts_loop_control_inside_nested_if() {
        let program = crate::parser::parse(
            crate::lexer::lex(
                "while true {
                    if true { continue; }
                    if false { break; }
                }",
            )
            .unwrap(),
        )
        .unwrap();

        check(&program).unwrap();
    }
}
