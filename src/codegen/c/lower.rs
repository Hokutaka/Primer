use crate::ir as primer_ir;

use super::ir::{BinaryOp, Expr, ExprKind, Module, PrintFormat, Statement, Type, UnaryOp};

pub fn lower(program: &primer_ir::Program) -> Module {
    Module {
        statements: program.statements.iter().map(lower_statement).collect(),
    }
}

fn lower_statement(statement: &primer_ir::Statement) -> Statement {
    match &statement.kind {
        primer_ir::StatementKind::Binding { name, ty, value } => Statement::Binding {
            name: name.clone(),
            ty: (*ty).into(),
            value: lower_expr(value),
        },

        primer_ir::StatementKind::Print { value } => Statement::Print {
            format: print_format(value.ty),
            value: lower_expr(value),
        },
    }
}

fn lower_expr(expr: &primer_ir::Expr) -> Expr {
    let kind = match &expr.kind {
        primer_ir::ExprKind::Integer(value) => ExprKind::Integer(*value),

        primer_ir::ExprKind::Float { text } => ExprKind::Float {
            text: text.clone(),
            suffix_f32: expr.ty == primer_ir::Type::F32,
        },

        primer_ir::ExprKind::Variable(name) => ExprKind::Variable(name.clone()),

        primer_ir::ExprKind::Unary { op, value } => ExprKind::Unary {
            op: (*op).into(),
            value: Box::new(lower_expr(value)),
        },

        primer_ir::ExprKind::Binary { op, left, right } => ExprKind::Binary {
            op: (*op).into(),
            left: Box::new(lower_expr(left)),
            right: Box::new(lower_expr(right)),
        },
    };

    Expr {
        ty: expr.ty.into(),
        kind,
    }
}

fn print_format(ty: primer_ir::Type) -> PrintFormat {
    match ty {
        primer_ir::Type::I64 => PrintFormat::I64,
        primer_ir::Type::F32 => PrintFormat::F32,
        primer_ir::Type::F64 => PrintFormat::F64,
    }
}

impl From<primer_ir::Type> for Type {
    fn from(value: primer_ir::Type) -> Self {
        match value {
            primer_ir::Type::I64 => Self::I64,
            primer_ir::Type::F32 => Self::Float,
            primer_ir::Type::F64 => Self::Double,
        }
    }
}

impl From<primer_ir::UnaryOp> for UnaryOp {
    fn from(value: primer_ir::UnaryOp) -> Self {
        match value {
            primer_ir::UnaryOp::Negate => Self::Negate,
        }
    }
}

impl From<primer_ir::BinaryOp> for BinaryOp {
    fn from(value: primer_ir::BinaryOp) -> Self {
        match value {
            primer_ir::BinaryOp::Add => Self::Add,
            primer_ir::BinaryOp::Subtract => Self::Subtract,
            primer_ir::BinaryOp::Multiply => Self::Multiply,
            primer_ir::BinaryOp::Divide => Self::Divide,
        }
    }
}
