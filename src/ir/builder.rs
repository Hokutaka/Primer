use crate::{
    ast,
    diagnostic::Diagnostic,
    semantic::{self, Bindings},
};

use super::{BinaryOp, Expr, ExprKind, Program, Statement, StatementKind, Type, UnaryOp};

pub fn build(program: &ast::Program) -> Result<Program, Diagnostic> {
    let bindings = semantic::check(program)?;
    let mut statements = Vec::with_capacity(program.statements.len());

    for statement in &program.statements {
        statements.push(build_statement(statement, &bindings)?);
    }

    Ok(Program { statements })
}

fn build_statement(statement: &ast::Stmt, bindings: &Bindings) -> Result<Statement, Diagnostic> {
    let kind = match &statement.kind {
        ast::StmtKind::Binding { name, value, .. } => {
            let ty = bindings.get(name).copied().ok_or_else(|| {
                Diagnostic::without_span(format!("missing resolved type for binding `{name}`"))
            })?;

            StatementKind::Binding {
                name: name.clone(),
                ty: ty.into(),
                value: build_expr(value, Some(ty), bindings)?,
            }
        }
        ast::StmtKind::Print { value } => {
            let ty = semantic::type_of_expr(value, bindings)?;

            StatementKind::Print {
                value: build_expr(value, Some(ty), bindings)?,
            }
        }
    };

    Ok(Statement {
        kind,
        span: statement.span,
    })
}

fn build_expr(
    expr: &ast::Expr,
    expected: Option<ast::Type>,
    bindings: &Bindings,
) -> Result<Expr, Diagnostic> {
    let ty = semantic::type_of_expr_expected(expr, bindings, expected)?;

    let kind = match &expr.kind {
        ast::ExprKind::Integer(value) => ExprKind::Integer(*value),

        ast::ExprKind::Float { text, .. } => ExprKind::Float { text: text.clone() },

        ast::ExprKind::Variable(name) => ExprKind::Variable(name.clone()),

        ast::ExprKind::Unary { op, value } => ExprKind::Unary {
            op: (*op).into(),
            value: Box::new(build_expr(value, Some(ty), bindings)?),
        },

        ast::ExprKind::Binary { op, left, right } => ExprKind::Binary {
            op: (*op).into(),
            left: Box::new(build_expr(left, Some(ty), bindings)?),
            right: Box::new(build_expr(right, Some(ty), bindings)?),
        },
    };

    Ok(Expr {
        ty: ty.into(),
        kind,
        span: expr.span,
    })
}

impl From<ast::Type> for Type {
    fn from(value: ast::Type) -> Self {
        match value {
            ast::Type::I64 => Self::I64,
            ast::Type::F32 => Self::F32,
            ast::Type::F64 => Self::F64,
        }
    }
}

impl From<ast::UnaryOp> for UnaryOp {
    fn from(value: ast::UnaryOp) -> Self {
        match value {
            ast::UnaryOp::Negate => Self::Negate,
        }
    }
}

impl From<ast::BinaryOp> for BinaryOp {
    fn from(value: ast::BinaryOp) -> Self {
        match value {
            ast::BinaryOp::Add => Self::Add,
            ast::BinaryOp::Subtract => Self::Subtract,
            ast::BinaryOp::Multiply => Self::Multiply,
            ast::BinaryOp::Divide => Self::Divide,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        BinaryOp as AstBinaryOp, Expr as AstExpr, ExprKind as AstExprKind, Program as AstProgram,
        Stmt, StmtKind as AstStmtKind, Type as AstType, TypeSpec,
    };
    use crate::source::Span;

    use super::*;

    fn ast_expr(kind: AstExprKind) -> AstExpr {
        AstExpr {
            kind,
            span: Span::empty(0),
        }
    }

    fn ast_stmt(kind: AstStmtKind) -> Stmt {
        Stmt {
            kind,
            span: Span::empty(0),
        }
    }

    #[test]
    fn resolves_contextual_f32_literals() {
        let ast = AstProgram {
            statements: vec![ast_stmt(AstStmtKind::Binding {
                name: "x".into(),
                type_spec: TypeSpec::Explicit(AstType::F32),
                value: ast_expr(AstExprKind::Binary {
                    op: AstBinaryOp::Add,
                    left: Box::new(ast_expr(AstExprKind::Float {
                        text: "0.1".into(),
                        explicit_type: None,
                    })),
                    right: Box::new(ast_expr(AstExprKind::Float {
                        text: "0.2".into(),
                        explicit_type: None,
                    })),
                }),
            })],
        };

        let ir = build(&ast).unwrap();
        let StatementKind::Binding { ty, value, .. } = &ir.statements[0].kind else {
            panic!("expected binding")
        };
        assert_eq!(*ty, Type::F32);
        assert_eq!(value.ty, Type::F32);

        let ExprKind::Binary { left, right, .. } = &value.kind else {
            panic!("expected binary")
        };
        assert_eq!(left.ty, Type::F32);
        assert_eq!(right.ty, Type::F32);
    }

    #[test]
    fn infer_defaults_unsuffixed_float_to_f64() {
        let ast = AstProgram {
            statements: vec![ast_stmt(AstStmtKind::Binding {
                name: "x".into(),
                type_spec: TypeSpec::Infer,
                value: ast_expr(AstExprKind::Float {
                    text: "0.1".into(),
                    explicit_type: None,
                }),
            })],
        };

        let ir = build(&ast).unwrap();
        let StatementKind::Binding { ty, value, .. } = &ir.statements[0].kind else {
            panic!("expected binding")
        };
        assert_eq!(*ty, Type::F64);
        assert_eq!(value.ty, Type::F64);
    }

    #[test]
    fn preserves_expression_spans() {
        let ast = AstProgram {
            statements: vec![ast_stmt(AstStmtKind::Print {
                value: AstExpr {
                    kind: AstExprKind::Binary {
                        op: AstBinaryOp::Add,
                        left: Box::new(AstExpr {
                            kind: AstExprKind::Integer(1),
                            span: Span::new(0, 1),
                        }),
                        right: Box::new(AstExpr {
                            kind: AstExprKind::Integer(2),
                            span: Span::new(4, 5),
                        }),
                    },
                    span: Span::new(0, 5),
                },
            })],
        };

        let ir = build(&ast).unwrap();
        let StatementKind::Print { value } = &ir.statements[0].kind else {
            panic!("expected print");
        };

        assert_eq!(value.span, Span::new(0, 5));

        let ExprKind::Binary { left, right, .. } = &value.kind else {
            panic!("expected binary expression");
        };

        assert_eq!(left.span, Span::new(0, 1));
        assert_eq!(right.span, Span::new(4, 5));
    }

    #[test]
    fn preserves_statement_spans() {
        let statement_span = Span::new(0, 9);
        let ast = AstProgram {
            statements: vec![Stmt {
                kind: AstStmtKind::Print {
                    value: ast_expr(AstExprKind::Integer(1)),
                },
                span: statement_span,
            }],
        };

        let ir = build(&ast).unwrap();

        assert_eq!(ir.statements[0].span, statement_span);
    }
}
