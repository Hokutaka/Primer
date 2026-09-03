use crate::ir as primer_ir;

use super::ir::{Instruction, Local, Module, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut locals = Vec::new();
    let mut local_names = std::collections::HashMap::new();
    let mut name_counts = std::collections::HashMap::new();
    collect_locals(
        &program.statements,
        &mut locals,
        &mut local_names,
        &mut name_counts,
    );

    let mut instructions = Vec::new();

    for statement in &program.statements {
        lower_statement(statement, &local_names, &mut instructions);
    }

    Module {
        locals,
        instructions,
    }
}

fn lower_statement(
    statement: &primer_ir::Statement,
    local_names: &std::collections::HashMap<primer_ir::BindingId, String>,
    instructions: &mut Vec<Instruction>,
) {
    match &statement.kind {
        primer_ir::StatementKind::Binding { id, value, .. } => {
            lower_expr(value, local_names, instructions);
            instructions.push(Instruction::LocalSet(local_names[id].clone()));
        }

        primer_ir::StatementKind::Assignment { id, value, .. } => {
            lower_expr(value, local_names, instructions);
            instructions.push(Instruction::LocalSet(local_names[id].clone()));
        }

        primer_ir::StatementKind::Print { value } => {
            lower_expr(value, local_names, instructions);
            instructions.push(Instruction::CallPrint(value.ty.into()));
        }

        primer_ir::StatementKind::If {
            condition,
            then_body,
            else_body,
        } => {
            lower_expr(condition, local_names, instructions);
            let mut then_instructions = Vec::new();
            let mut else_instructions = Vec::new();
            for statement in then_body {
                lower_statement(statement, local_names, &mut then_instructions);
            }
            for statement in else_body {
                lower_statement(statement, local_names, &mut else_instructions);
            }
            instructions.push(Instruction::If {
                then_instructions,
                else_instructions,
            });
        }

        primer_ir::StatementKind::While { condition, body } => {
            let mut condition_instructions = Vec::new();
            let mut body_instructions = Vec::new();
            lower_expr(condition, local_names, &mut condition_instructions);
            for statement in body {
                lower_statement(statement, local_names, &mut body_instructions);
            }
            instructions.push(Instruction::While {
                condition_instructions,
                body_instructions,
            });
        }
    }
}

fn lower_expr(
    expr: &primer_ir::Expr,
    local_names: &std::collections::HashMap<primer_ir::BindingId, String>,
    instructions: &mut Vec<Instruction>,
) {
    match &expr.kind {
        primer_ir::ExprKind::Boolean(value) => {
            instructions.push(Instruction::I32Const(i32::from(*value)));
        }

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

            primer_ir::Type::Bool => {
                unreachable!("boolean cannot be lowered as float");
            }
        },

        primer_ir::ExprKind::Variable { id, .. } => {
            instructions.push(Instruction::LocalGet(local_names[id].clone()));
        }

        primer_ir::ExprKind::Unary { op, value } => match (*op, expr.ty) {
            (primer_ir::UnaryOp::Negate, primer_ir::Type::I64) => {
                instructions.push(Instruction::I64Const(0));
                lower_expr(value, local_names, instructions);
                instructions.push(Instruction::I64Sub);
            }

            (primer_ir::UnaryOp::Negate, primer_ir::Type::F32) => {
                lower_expr(value, local_names, instructions);
                instructions.push(Instruction::F32Neg);
            }

            (primer_ir::UnaryOp::Negate, primer_ir::Type::F64) => {
                lower_expr(value, local_names, instructions);
                instructions.push(Instruction::F64Neg);
            }

            (primer_ir::UnaryOp::Not, primer_ir::Type::Bool) => {
                lower_expr(value, local_names, instructions);
                instructions.push(Instruction::I32Eqz);
            }

            (primer_ir::UnaryOp::Negate, primer_ir::Type::Bool)
            | (primer_ir::UnaryOp::Not, primer_ir::Type::I64)
            | (primer_ir::UnaryOp::Not, primer_ir::Type::F32)
            | (primer_ir::UnaryOp::Not, primer_ir::Type::F64) => {
                unreachable!("semantic analysis rejects invalid unary operands");
            }
        },

        primer_ir::ExprKind::Binary { op, left, right } => {
            lower_expr(left, local_names, instructions);
            lower_expr(right, local_names, instructions);
            instructions.push(lower_binary(*op, left.ty));
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

        (primer_ir::BinaryOp::Equal, primer_ir::Type::Bool) => Instruction::I32Eq,
        (primer_ir::BinaryOp::NotEqual, primer_ir::Type::Bool) => Instruction::I32Ne,

        (primer_ir::BinaryOp::Equal, primer_ir::Type::I64) => Instruction::I64Eq,
        (primer_ir::BinaryOp::NotEqual, primer_ir::Type::I64) => Instruction::I64Ne,
        (primer_ir::BinaryOp::Less, primer_ir::Type::I64) => Instruction::I64LtS,
        (primer_ir::BinaryOp::LessEqual, primer_ir::Type::I64) => Instruction::I64LeS,
        (primer_ir::BinaryOp::Greater, primer_ir::Type::I64) => Instruction::I64GtS,
        (primer_ir::BinaryOp::GreaterEqual, primer_ir::Type::I64) => Instruction::I64GeS,

        (primer_ir::BinaryOp::Equal, primer_ir::Type::F32) => Instruction::F32Eq,
        (primer_ir::BinaryOp::NotEqual, primer_ir::Type::F32) => Instruction::F32Ne,
        (primer_ir::BinaryOp::Less, primer_ir::Type::F32) => Instruction::F32Lt,
        (primer_ir::BinaryOp::LessEqual, primer_ir::Type::F32) => Instruction::F32Le,
        (primer_ir::BinaryOp::Greater, primer_ir::Type::F32) => Instruction::F32Gt,
        (primer_ir::BinaryOp::GreaterEqual, primer_ir::Type::F32) => Instruction::F32Ge,

        (primer_ir::BinaryOp::Equal, primer_ir::Type::F64) => Instruction::F64Eq,
        (primer_ir::BinaryOp::NotEqual, primer_ir::Type::F64) => Instruction::F64Ne,
        (primer_ir::BinaryOp::Less, primer_ir::Type::F64) => Instruction::F64Lt,
        (primer_ir::BinaryOp::LessEqual, primer_ir::Type::F64) => Instruction::F64Le,
        (primer_ir::BinaryOp::Greater, primer_ir::Type::F64) => Instruction::F64Gt,
        (primer_ir::BinaryOp::GreaterEqual, primer_ir::Type::F64) => Instruction::F64Ge,

        (primer_ir::BinaryOp::Add, primer_ir::Type::Bool)
        | (primer_ir::BinaryOp::Subtract, primer_ir::Type::Bool)
        | (primer_ir::BinaryOp::Multiply, primer_ir::Type::Bool)
        | (primer_ir::BinaryOp::Divide, primer_ir::Type::Bool)
        | (primer_ir::BinaryOp::Less, primer_ir::Type::Bool)
        | (primer_ir::BinaryOp::LessEqual, primer_ir::Type::Bool)
        | (primer_ir::BinaryOp::Greater, primer_ir::Type::Bool)
        | (primer_ir::BinaryOp::GreaterEqual, primer_ir::Type::Bool) => {
            unreachable!("semantic analysis rejects invalid binary operands")
        }
    }
}

fn collect_locals(
    statements: &[primer_ir::Statement],
    locals: &mut Vec<Local>,
    local_names: &mut std::collections::HashMap<primer_ir::BindingId, String>,
    name_counts: &mut std::collections::HashMap<String, usize>,
) {
    for statement in statements {
        match &statement.kind {
            primer_ir::StatementKind::Binding { id, name, ty, .. } => {
                let count = name_counts.entry(name.clone()).or_default();
                let lowered_name = if *count == 0 {
                    name.clone()
                } else {
                    format!("{name}_{}", id.0)
                };
                *count += 1;
                locals.push(Local {
                    name: lowered_name.clone(),
                    ty: (*ty).into(),
                });
                local_names.insert(*id, lowered_name);
            }
            primer_ir::StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_locals(then_body, locals, local_names, name_counts);
                collect_locals(else_body, locals, local_names, name_counts);
            }
            primer_ir::StatementKind::While { body, .. } => {
                collect_locals(body, locals, local_names, name_counts);
            }
            primer_ir::StatementKind::Assignment { .. }
            | primer_ir::StatementKind::Print { .. } => {}
        }
    }
}

impl From<primer_ir::Type> for Type {
    fn from(value: primer_ir::Type) -> Self {
        match value {
            primer_ir::Type::Bool => Self::Bool,
            primer_ir::Type::I64 => Self::I64,
            primer_ir::Type::F32 => Self::F32,
            primer_ir::Type::F64 => Self::F64,
        }
    }
}
