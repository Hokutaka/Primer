use std::fmt::Write;

use super::{BinaryOp, Expr, ExprKind, Program, Statement, Type, UnaryOp};

pub fn emit(program: &Program) -> String {
    let mut output = String::new();
    writeln!(output, "; Primer IR v0.1").unwrap();

    if !program.statements.is_empty() {
        writeln!(output).unwrap();
    }

    for statement in &program.statements {
        match statement {
            Statement::Binding { name, ty, value } => {
                write!(output, "%{name}: {} = ", type_name(*ty)).unwrap();
                emit_expr(value, &mut output);
                writeln!(output).unwrap();
            }
            Statement::Print { value } => {
                write!(output, "print.{} ", type_name(value.ty)).unwrap();
                emit_expr(value, &mut output);
                writeln!(output).unwrap();
            }
        }
    }

    output
}

fn emit_expr(expr: &Expr, output: &mut String) {
    match &expr.kind {
        ExprKind::Integer(value) => {
            write!(output, "{value}i64").unwrap();
        }
        ExprKind::Float { text } => {
            write!(output, "{text}{}", type_name(expr.ty)).unwrap();
        }
        ExprKind::Variable(name) => {
            write!(output, "%{name}:{}", type_name(expr.ty)).unwrap();
        }
        ExprKind::Unary { op, value } => {
            write!(output, "{}.{}(", unary_name(*op), type_name(expr.ty)).unwrap();
            emit_expr(value, output);
            output.push(')');
        }
        ExprKind::Binary { op, left, right } => {
            write!(output, "{}.{}(", binary_name(*op), type_name(expr.ty)).unwrap();
            emit_expr(left, output);
            output.push_str(", ");
            emit_expr(right, output);
            output.push(')');
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

fn unary_name(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Negate => "neg",
    }
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Subtract => "sub",
        BinaryOp::Multiply => "mul",
        BinaryOp::Divide => "div",
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::{
        BinaryOp as AstBinaryOp, Expr as AstExpr, Program as AstProgram, Stmt, Type as AstType,
        TypeSpec,
    };
    use crate::ir::builder::build;

    use super::emit;

    #[test]
    fn emits_resolved_types() {
        let ast = AstProgram {
            statements: vec![
                Stmt::Binding {
                    name: "x".into(),
                    type_spec: TypeSpec::Explicit(AstType::F32),
                    value: AstExpr::Binary {
                        op: AstBinaryOp::Add,
                        left: Box::new(AstExpr::Float {
                            text: "0.1".into(),
                            explicit_type: None,
                        }),
                        right: Box::new(AstExpr::Float {
                            text: "0.2".into(),
                            explicit_type: None,
                        }),
                    },
                },
                Stmt::Print {
                    value: AstExpr::Variable("x".into()),
                },
            ],
        };

        let text = emit(&build(&ast).unwrap());
        assert_eq!(
            text,
            "; Primer IR v0.1\n\n%x: f32 = add.f32(0.1f32, 0.2f32)\nprint.f32 %x:f32\n"
        );
    }
}
