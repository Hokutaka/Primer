use std::fmt::Write;

use super::{BinaryOp, Expr, ExprKind, Program, Statement, StatementKind, Type, UnaryOp};

pub fn emit(program: &Program) -> String {
    let mut output = String::new();
    writeln!(output, "; Primer IR v0.1").unwrap();

    if !program.statements.is_empty() {
        writeln!(output).unwrap();
    }

    for statement in &program.statements {
        emit_statement(statement, 0, &mut output);
    }

    output
}

fn emit_statement(statement: &Statement, indent: usize, output: &mut String) {
    let prefix = "  ".repeat(indent);

    match &statement.kind {
        StatementKind::Binding {
            id,
            mutable,
            name,
            ty,
            value,
        } => {
            if *mutable {
                output.push_str(&prefix);
                output.push_str("mut ");
            } else {
                output.push_str(&prefix);
            }
            write!(output, "%{name}@{}: {} = ", id.0, type_name(*ty)).unwrap();
            emit_expr(value, output);
            writeln!(output).unwrap();
        }
        StatementKind::Assignment {
            id,
            name,
            ty,
            value,
        } => {
            write!(output, "{prefix}set %{name}@{}:{} = ", id.0, type_name(*ty)).unwrap();
            emit_expr(value, output);
            writeln!(output).unwrap();
        }
        StatementKind::Print { value } => {
            write!(output, "{prefix}print.{} ", type_name(value.ty)).unwrap();
            emit_expr(value, output);
            writeln!(output).unwrap();
        }
        StatementKind::If {
            condition,
            then_body,
            else_body,
        } => {
            write!(output, "{prefix}if.bool ").unwrap();
            emit_expr(condition, output);
            writeln!(output, " {{").unwrap();
            for statement in then_body {
                emit_statement(statement, indent + 1, output);
            }
            write!(output, "{prefix}}}").unwrap();

            if else_body.is_empty() {
                writeln!(output).unwrap();
            } else {
                writeln!(output, " else {{").unwrap();
                for statement in else_body {
                    emit_statement(statement, indent + 1, output);
                }
                writeln!(output, "{prefix}}}").unwrap();
            }
        }
        StatementKind::While { condition, body } => {
            write!(output, "{prefix}while.bool ").unwrap();
            emit_expr(condition, output);
            writeln!(output, " {{").unwrap();
            for statement in body {
                emit_statement(statement, indent + 1, output);
            }
            writeln!(output, "{prefix}}}").unwrap();
        }
        StatementKind::For {
            initializer,
            condition,
            update,
            body,
        } => {
            writeln!(output, "{prefix}for.loop {{").unwrap();
            writeln!(output, "{prefix}  start {{").unwrap();
            emit_statement(initializer, indent + 2, output);
            writeln!(output, "{prefix}  }}").unwrap();
            write!(output, "{prefix}  condition.bool ").unwrap();
            emit_expr(condition, output);
            writeln!(output).unwrap();
            writeln!(output, "{prefix}  body {{").unwrap();
            for statement in body {
                emit_statement(statement, indent + 2, output);
            }
            writeln!(output, "{prefix}  }}").unwrap();
            writeln!(output, "{prefix}  update {{").unwrap();
            emit_statement(update, indent + 2, output);
            writeln!(output, "{prefix}  }}").unwrap();
            writeln!(output, "{prefix}}}").unwrap();
        }
        StatementKind::Break => {
            writeln!(output, "{prefix}break").unwrap();
        }
        StatementKind::Continue => {
            writeln!(output, "{prefix}continue").unwrap();
        }
    }
}

fn emit_expr(expr: &Expr, output: &mut String) {
    match &expr.kind {
        ExprKind::Boolean(value) => {
            write!(output, "{value}:bool").unwrap();
        }
        ExprKind::Integer(value) => {
            write!(output, "{value}i64").unwrap();
        }
        ExprKind::Float { text } => {
            write!(output, "{text}{}", type_name(expr.ty)).unwrap();
        }
        ExprKind::Variable { id, name } => {
            write!(output, "%{name}@{}:{}", id.0, type_name(expr.ty)).unwrap();
        }
        ExprKind::Unary { op, value } => {
            write!(output, "{}.{}(", unary_name(*op), type_name(expr.ty)).unwrap();
            emit_expr(value, output);
            output.push(')');
        }
        ExprKind::Binary { op, left, right } => {
            let operation_type = if is_comparison(*op) { left.ty } else { expr.ty };

            write!(
                output,
                "{}.{}(",
                binary_name(*op),
                type_name(operation_type)
            )
            .unwrap();
            emit_expr(left, output);
            output.push_str(", ");
            emit_expr(right, output);
            output.push(')');
        }
    }
}

fn type_name(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "bool",
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
    }
}

fn unary_name(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Negate => "neg",
        UnaryOp::Not => "not",
    }
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Subtract => "sub",
        BinaryOp::Multiply => "mul",
        BinaryOp::Divide => "div",
        BinaryOp::Equal => "eq",
        BinaryOp::NotEqual => "ne",
        BinaryOp::Less => "lt",
        BinaryOp::LessEqual => "le",
        BinaryOp::Greater => "gt",
        BinaryOp::GreaterEqual => "ge",
    }
}

const fn is_comparison(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::LessEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEqual
    )
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        BinaryOp as AstBinaryOp, Expr as AstExpr, ExprKind as AstExprKind, Item as AstItem,
        Program as AstProgram, Stmt, StmtKind as AstStmtKind, Type as AstType, TypeSpec,
    };
    use crate::ir::builder::build;
    use crate::source::Span;

    use super::emit;

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
    fn emits_resolved_types() {
        let ast = AstProgram {
            items: vec![
                AstItem::Statement(ast_stmt(AstStmtKind::Binding {
                    mutable: false,
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
                })),
                AstItem::Statement(ast_stmt(AstStmtKind::Print {
                    value: ast_expr(AstExprKind::Variable("x".into())),
                })),
            ],
        };

        let text = emit(&build(&ast).unwrap());
        assert_eq!(
            text,
            "; Primer IR v0.1\n\n%x@0: f32 = add.f32(0.1f32, 0.2f32)\nprint.f32 %x@0:f32\n"
        );
    }
}
