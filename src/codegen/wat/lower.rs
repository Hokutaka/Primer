use crate::ir as primer_ir;

use super::ir::{Instruction, Local, Module, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let locals = program
        .statements
        .iter()
        .filter_map(|statement| match statement {
            primer_ir::Statement::Binding { name, ty, .. } => Some(Local {
                name: name.clone(),
                ty: (*ty).into(),
            }),
            primer_ir::Statement::Print { .. } => None,
        })
        .collect();

    let mut instructions = Vec::new();

    for statement in &program.statements {
        lower_statement(statement, &mut instructions);
    }

    Module {
        locals,
        instructions,
    }
}

fn lower_statement(statement: &primer_ir::Statement, instructions: &mut Vec<Instruction>) {
    match statement {
        primer_ir::Statement::Binding { name, value, .. } => {
            lower_expr(value, instructions);
            instructions.push(Instruction::LocalSet(name.clone()));
        }

        primer_ir::Statement::Print { value } => {
            lower_expr(value, instructions);
            instructions.push(Instruction::CallPrint(value.ty.into()));
        }
    }
}

fn lower_expr(expr: &primer_ir::Expr, instructions: &mut Vec<Instruction>) {
    match &expr.kind {
        primer_ir::ExprKind::Integer(value) => {
            instructions.push(Instruction::I64Const(*value));
        }

        primer_ir::ExprKind::Float { text } => match expr.ty {
            primer_ir::Type::F32 => {
                instructions.push(Instruction::F32Const(text.clone()));
            }

            primer_ir::Type::F64 => {
                instructions.push(Instruction::F64Const(text.clone()));
            }

            primer_ir::Type::I64 => {
                unreachable!("integer cannot be lowered as float");
            }
        },

        primer_ir::ExprKind::Variable(name) => {
            instructions.push(Instruction::LocalGet(name.clone()));
        }

        primer_ir::ExprKind::Unary { op, value } => match (*op, expr.ty) {
            (primer_ir::UnaryOp::Negate, primer_ir::Type::I64) => {
                instructions.push(Instruction::I64Const(0));
                lower_expr(value, instructions);
                instructions.push(Instruction::I64Sub);
            }

            (primer_ir::UnaryOp::Negate, primer_ir::Type::F32) => {
                lower_expr(value, instructions);
                instructions.push(Instruction::F32Neg);
            }

            (primer_ir::UnaryOp::Negate, primer_ir::Type::F64) => {
                lower_expr(value, instructions);
                instructions.push(Instruction::F64Neg);
            }
        },

        primer_ir::ExprKind::Binary { op, left, right } => {
            lower_expr(left, instructions);
            lower_expr(right, instructions);
            instructions.push(lower_binary(*op, expr.ty));
        }
    }
}

fn lower_binary(op: primer_ir::BinaryOp, ty: primer_ir::Type) -> Instruction {
    match (op, ty) {
        (primer_ir::BinaryOp::Add, primer_ir::Type::I64) => Instruction::I64Add,
        (primer_ir::BinaryOp::Subtract, primer_ir::Type::I64) => Instruction::I64Sub,
        (primer_ir::BinaryOp::Multiply, primer_ir::Type::I64) => Instruction::I64Mul,
        (primer_ir::BinaryOp::Divide, primer_ir::Type::I64) => Instruction::I64DivS,

        (primer_ir::BinaryOp::Add, primer_ir::Type::F32) => Instruction::F32Add,
        (primer_ir::BinaryOp::Subtract, primer_ir::Type::F32) => Instruction::F32Sub,
        (primer_ir::BinaryOp::Multiply, primer_ir::Type::F32) => Instruction::F32Mul,
        (primer_ir::BinaryOp::Divide, primer_ir::Type::F32) => Instruction::F32Div,

        (primer_ir::BinaryOp::Add, primer_ir::Type::F64) => Instruction::F64Add,
        (primer_ir::BinaryOp::Subtract, primer_ir::Type::F64) => Instruction::F64Sub,
        (primer_ir::BinaryOp::Multiply, primer_ir::Type::F64) => Instruction::F64Mul,
        (primer_ir::BinaryOp::Divide, primer_ir::Type::F64) => Instruction::F64Div,
    }
}

impl From<primer_ir::Type> for Type {
    fn from(value: primer_ir::Type) -> Self {
        match value {
            primer_ir::Type::I64 => Self::I64,
            primer_ir::Type::F32 => Self::F32,
            primer_ir::Type::F64 => Self::F64,
        }
    }
}
