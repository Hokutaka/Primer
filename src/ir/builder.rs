use std::collections::HashMap;

use crate::{
    ast,
    diagnostic::Diagnostic,
    semantic::{self, BindingInfo, Bindings},
};

use super::{
    BinaryOp, BindingId, Expr, ExprKind, Program, Statement, StatementKind, Type, UnaryOp,
};

pub fn build(program: &ast::Program) -> Result<Program, Diagnostic> {
    semantic::check(program)?;

    let mut builder = Builder {
        scopes: vec![HashMap::new()],
        next_binding_id: 0,
    };

    let statements = program
        .items
        .iter()
        .map(|item| match item {
            ast::Item::Statement(statement) => builder.build_statement(statement),
        })
        .collect::<Result<_, _>>()?;

    Ok(Program { statements })
}

#[derive(Debug, Clone, Copy)]
struct ResolvedBinding {
    id: BindingId,
    info: BindingInfo,
}

struct Builder {
    scopes: Vec<HashMap<String, ResolvedBinding>>,
    next_binding_id: usize,
}

impl Builder {
    fn build_statements(&mut self, statements: &[ast::Stmt]) -> Result<Vec<Statement>, Diagnostic> {
        statements
            .iter()
            .map(|statement| self.build_statement(statement))
            .collect()
    }

    fn build_statement(&mut self, statement: &ast::Stmt) -> Result<Statement, Diagnostic> {
        let bindings = self.visible_bindings();

        let kind = match &statement.kind {
            ast::StmtKind::Binding {
                mutable,
                name,
                type_spec,
                value,
            } => {
                let ty = match type_spec {
                    ast::TypeSpec::Explicit(ty) => *ty,
                    ast::TypeSpec::Infer => semantic::type_of_expr(value, &bindings)?,
                };
                let value = self.build_expr(value, Some(ty), &bindings)?;
                let id = BindingId(self.next_binding_id);
                self.next_binding_id += 1;

                self.scopes
                    .last_mut()
                    .expect("current scope must exist")
                    .insert(
                        name.clone(),
                        ResolvedBinding {
                            id,
                            info: BindingInfo {
                                ty,
                                mutable: *mutable,
                            },
                        },
                    );

                StatementKind::Binding {
                    id,
                    mutable: *mutable,
                    name: name.clone(),
                    ty: ty.into(),
                    value,
                }
            }
            ast::StmtKind::Assignment { name, value, .. } => {
                let binding = self.resolve(name).ok_or_else(|| {
                    Diagnostic::without_span(format!("missing resolved binding `{name}`"))
                })?;

                StatementKind::Assignment {
                    id: binding.id,
                    name: name.clone(),
                    ty: binding.info.ty.into(),
                    value: self.build_expr(value, Some(binding.info.ty), &bindings)?,
                }
            }
            ast::StmtKind::Print { value } => {
                let ty = semantic::type_of_expr(value, &bindings)?;
                StatementKind::Print {
                    value: self.build_expr(value, Some(ty), &bindings)?,
                }
            }
            ast::StmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition = self.build_expr(condition, Some(ast::Type::Bool), &bindings)?;
                let then_body = self.with_scope(|builder| builder.build_statements(then_body))?;
                let else_body = self.with_scope(|builder| builder.build_statements(else_body))?;

                StatementKind::If {
                    condition,
                    then_body,
                    else_body,
                }
            }
            ast::StmtKind::While { condition, body } => {
                let condition = self.build_expr(condition, Some(ast::Type::Bool), &bindings)?;
                let body = self.with_scope(|builder| builder.build_statements(body))?;

                StatementKind::While { condition, body }
            }
            ast::StmtKind::For {
                initializer,
                condition,
                update,
                body,
            } => self.with_scope(|builder| {
                let initializer = Box::new(builder.build_statement(initializer)?);
                let bindings = builder.visible_bindings();
                let condition = builder.build_expr(condition, Some(ast::Type::Bool), &bindings)?;
                let update = Box::new(builder.build_statement(update)?);
                let body = builder.with_scope(|builder| builder.build_statements(body))?;

                Ok(StatementKind::For {
                    initializer,
                    condition,
                    update,
                    body,
                })
            })?,
            ast::StmtKind::Break => StatementKind::Break,
            ast::StmtKind::Continue => StatementKind::Continue,
        };

        Ok(Statement {
            kind,
            span: statement.span,
        })
    }

    fn build_expr(
        &self,
        expr: &ast::Expr,
        expected: Option<ast::Type>,
        bindings: &Bindings,
    ) -> Result<Expr, Diagnostic> {
        let ty = semantic::type_of_expr_expected(expr, bindings, expected)?;

        let kind = match &expr.kind {
            ast::ExprKind::Boolean(value) => ExprKind::Boolean(*value),
            ast::ExprKind::Integer(value) => ExprKind::Integer(*value),
            ast::ExprKind::Float { text, .. } => ExprKind::Float { text: text.clone() },
            ast::ExprKind::Variable(name) => {
                let binding = self.resolve(name).ok_or_else(|| {
                    Diagnostic::without_span(format!("missing resolved binding `{name}`"))
                })?;
                ExprKind::Variable {
                    id: binding.id,
                    name: name.clone(),
                }
            }
            ast::ExprKind::Unary { op, value } => ExprKind::Unary {
                op: (*op).into(),
                value: Box::new(self.build_expr(value, Some(ty), bindings)?),
            },
            ast::ExprKind::Binary { op, left, right } => {
                let (left_expected, right_expected) = if is_comparison(*op) {
                    semantic::comparison_operand_types(left, right, bindings)?
                } else {
                    (ty, ty)
                };

                ExprKind::Binary {
                    op: (*op).into(),
                    left: Box::new(self.build_expr(left, Some(left_expected), bindings)?),
                    right: Box::new(self.build_expr(right, Some(right_expected), bindings)?),
                }
            }
        };

        Ok(Expr {
            ty: ty.into(),
            kind,
            span: expr.span,
        })
    }

    fn resolve(&self, name: &str) -> Option<ResolvedBinding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn visible_bindings(&self) -> Bindings {
        let mut bindings = HashMap::new();
        for scope in &self.scopes {
            bindings.extend(
                scope
                    .iter()
                    .map(|(name, binding)| (name.clone(), binding.info)),
            );
        }
        bindings
    }

    fn with_scope<T>(
        &mut self,
        build: impl FnOnce(&mut Self) -> Result<T, Diagnostic>,
    ) -> Result<T, Diagnostic> {
        self.scopes.push(HashMap::new());
        let result = build(self);
        self.scopes.pop();
        result
    }
}

impl From<ast::Type> for Type {
    fn from(value: ast::Type) -> Self {
        match value {
            ast::Type::Bool => Self::Bool,
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
            ast::UnaryOp::Not => Self::Not,
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
            ast::BinaryOp::Equal => Self::Equal,
            ast::BinaryOp::NotEqual => Self::NotEqual,
            ast::BinaryOp::Less => Self::Less,
            ast::BinaryOp::LessEqual => Self::LessEqual,
            ast::BinaryOp::Greater => Self::Greater,
            ast::BinaryOp::GreaterEqual => Self::GreaterEqual,
        }
    }
}

const fn is_comparison(op: ast::BinaryOp) -> bool {
    matches!(
        op,
        ast::BinaryOp::Equal
            | ast::BinaryOp::NotEqual
            | ast::BinaryOp::Less
            | ast::BinaryOp::LessEqual
            | ast::BinaryOp::Greater
            | ast::BinaryOp::GreaterEqual
    )
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        BinaryOp as AstBinaryOp, Expr as AstExpr, ExprKind as AstExprKind, Item as AstItem,
        Program as AstProgram, Stmt, StmtKind as AstStmtKind, Type as AstType, TypeSpec,
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

    fn ast_item(kind: AstStmtKind) -> AstItem {
        AstItem::Statement(ast_stmt(kind))
    }

    #[test]
    fn resolves_contextual_f32_literals() {
        let ast = AstProgram {
            items: vec![ast_item(AstStmtKind::Binding {
                name: "x".into(),
                mutable: false,
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
            items: vec![ast_item(AstStmtKind::Binding {
                name: "x".into(),
                mutable: false,
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
            items: vec![ast_item(AstStmtKind::Print {
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
            items: vec![AstItem::Statement(Stmt {
                kind: AstStmtKind::Print {
                    value: ast_expr(AstExprKind::Integer(1)),
                },
                span: statement_span,
            })],
        };

        let ir = build(&ast).unwrap();

        assert_eq!(ir.statements[0].span, statement_span);
    }
}
