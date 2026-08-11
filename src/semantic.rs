use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Program, Stmt, Type, TypeSpec};

pub type Bindings = HashMap<String, Type>;

pub fn check(program: &Program) -> Result<Bindings, String> {
    let mut bindings = HashMap::new();

    for statement in &program.statements {
        match statement {
            Stmt::Binding {
                name,
                type_spec,
                value,
            } => {
                if bindings.contains_key(name) {
                    return Err(format!("duplicate binding `{name}`"));
                }

                let value_type = match type_spec {
                    TypeSpec::Explicit(expected) => {
                        // 左辺の明示型を右辺へ渡す
                        let actual = type_of_expr_expected(value, &bindings, Some(*expected))?;

                        if actual != *expected {
                            return Err(format!(
                                "type mismatch for `{name}`: expected {}, found {}",
                                type_name(*expected),
                                type_name(actual),
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

            Stmt::Print { value } => {
                type_of_expr_expected(value, &bindings, None)?;
            }
        }
    }

    Ok(bindings)
}

pub fn type_of_expr(expr: &Expr, bindings: &Bindings) -> Result<Type, String> {
    type_of_expr_expected(expr, bindings, None)
}

fn type_of_expr_expected(
    expr: &Expr,
    bindings: &Bindings,
    expected: Option<Type>,
) -> Result<Type, String> {
    match expr {
        Expr::Integer(_) => Ok(Type::I64),

        Expr::Float { explicit_type, .. } => {
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

        Expr::Variable(name) => bindings
            .get(name)
            .copied()
            .ok_or_else(|| format!("unknown binding `{name}`")),

        Expr::Unary { value, .. } => type_of_expr_expected(value, bindings, expected),

        Expr::Binary { op, left, right } => {
            // 親から来た期待型を左右両方へ伝える
            let left_type = type_of_expr_expected(left, bindings, expected)?;

            let right_type = type_of_expr_expected(right, bindings, expected)?;

            if left_type != right_type {
                return Err(format!(
                    "cannot apply `{}` to {} and {}",
                    operator_name(*op),
                    type_name(left_type),
                    type_name(right_type),
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
    use crate::{ast::Type, lexer::lex, parser::parse};

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

        assert_eq!(
            check(&program).unwrap_err(),
            "cannot apply `+` to i64 and f64"
        );
    }
}
