use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{BinaryOp, Instruction, Module, Operand, PrintFormat, Slot, SlotId, Temp, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut slots = Vec::new();
    let mut slot_map = HashMap::new();

    for statement in &program.statements {
        if let primer_ir::Statement::Binding { name, ty, .. } = statement {
            let id = SlotId(slots.len());

            slots.push(Slot {
                name: name.clone(),
                ty: (*ty).into(),
            });

            slot_map.insert(name.clone(), id);
        }
    }

    let mut lowerer = Lowerer {
        slot_map,
        instructions: Vec::new(),
        temp: 0,
    };

    for statement in &program.statements {
        lowerer.lower_statement(statement);
    }

    Module {
        slots,
        instructions: lowerer.instructions,
    }
}

struct Lowerer {
    slot_map: HashMap<String, SlotId>,
    instructions: Vec<Instruction>,
    temp: usize,
}

#[derive(Debug, Clone, Copy)]
struct Value {
    ty: Type,
    operand: Operand,
}

impl Lowerer {
    fn lower_statement(&mut self, statement: &primer_ir::Statement) {
        match statement {
            primer_ir::Statement::Binding {
                name, ty, value, ..
            } => {
                let slot = self.slot(name);
                let ty = (*ty).into();

                self.instructions.push(Instruction::Alloca { slot });

                let value = self.lower_expr(value);

                self.instructions.push(Instruction::Store {
                    ty,
                    value: value.operand,
                    slot,
                });
            }

            primer_ir::Statement::Print { value } => {
                let value = self.lower_expr(value);
                self.lower_print(value);
            }
        }
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr) -> Value {
        match &expr.kind {
            primer_ir::ExprKind::Integer(value) => Value {
                ty: Type::I64,
                operand: Operand::Integer(*value),
            },

            primer_ir::ExprKind::Float { text } => match expr.ty {
                primer_ir::Type::F32 => {
                    let value = text
                        .parse::<f32>()
                        .expect("validated floating-point literal");

                    Value {
                        ty: Type::Float,
                        operand: Operand::Float32(value.to_bits()),
                    }
                }

                primer_ir::Type::F64 => {
                    let value = text
                        .parse::<f64>()
                        .expect("validated floating-point literal");

                    Value {
                        ty: Type::Double,
                        operand: Operand::Float64(value.to_bits()),
                    }
                }

                primer_ir::Type::I64 => {
                    unreachable!("integer cannot be lowered as float")
                }
            },

            primer_ir::ExprKind::Variable(name) => {
                let ty = expr.ty.into();
                let dest = self.next_temp();
                let slot = self.slot(name);

                self.instructions.push(Instruction::Load { dest, ty, slot });

                Value {
                    ty,
                    operand: Operand::Temp(dest),
                }
            }

            primer_ir::ExprKind::Unary { op, value } => {
                let value = self.lower_expr(value);
                let dest = self.next_temp();

                match (*op, value.ty) {
                    (primer_ir::UnaryOp::Negate, Type::I64) => {
                        self.instructions.push(Instruction::Binary {
                            dest,
                            op: BinaryOp::Sub,
                            ty: Type::I64,
                            left: Operand::Integer(0),
                            right: value.operand,
                        });
                    }

                    (primer_ir::UnaryOp::Negate, Type::Float | Type::Double) => {
                        self.instructions.push(Instruction::FNeg {
                            dest,
                            ty: value.ty,
                            value: value.operand,
                        });
                    }
                }

                Value {
                    ty: value.ty,
                    operand: Operand::Temp(dest),
                }
            }

            primer_ir::ExprKind::Binary { op, left, right } => {
                let ty = expr.ty.into();
                let left = self.lower_expr(left);
                let right = self.lower_expr(right);
                let dest = self.next_temp();

                self.instructions.push(Instruction::Binary {
                    dest,
                    op: binary_op(*op, ty),
                    ty,
                    left: left.operand,
                    right: right.operand,
                });

                Value {
                    ty,
                    operand: Operand::Temp(dest),
                }
            }
        }
    }

    fn lower_print(&mut self, value: Value) {
        match value.ty {
            Type::I64 => {
                self.instructions.push(Instruction::CallPrintf {
                    format: PrintFormat::I64,
                    arg_ty: Type::I64,
                    value: value.operand,
                });
            }

            Type::Float => {
                // C varargs promote float to double.
                let dest = self.next_temp();

                self.instructions.push(Instruction::FPExt {
                    dest,
                    value: value.operand,
                });

                self.instructions.push(Instruction::CallPrintf {
                    format: PrintFormat::F32,
                    arg_ty: Type::Double,
                    value: Operand::Temp(dest),
                });
            }

            Type::Double => {
                self.instructions.push(Instruction::CallPrintf {
                    format: PrintFormat::F64,
                    arg_ty: Type::Double,
                    value: value.operand,
                });
            }
        }
    }

    fn slot(&self, name: &str) -> SlotId {
        self.slot_map
            .get(name)
            .copied()
            .expect("binding must have an LLVM slot")
    }

    fn next_temp(&mut self) -> Temp {
        let temp = Temp(self.temp);
        self.temp += 1;
        temp
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

fn binary_op(op: primer_ir::BinaryOp, ty: Type) -> BinaryOp {
    match (op, ty) {
        (primer_ir::BinaryOp::Add, Type::I64) => BinaryOp::Add,
        (primer_ir::BinaryOp::Subtract, Type::I64) => BinaryOp::Sub,
        (primer_ir::BinaryOp::Multiply, Type::I64) => BinaryOp::Mul,
        (primer_ir::BinaryOp::Divide, Type::I64) => BinaryOp::SDiv,

        (primer_ir::BinaryOp::Add, Type::Float | Type::Double) => BinaryOp::FAdd,
        (primer_ir::BinaryOp::Subtract, Type::Float | Type::Double) => BinaryOp::FSub,
        (primer_ir::BinaryOp::Multiply, Type::Float | Type::Double) => BinaryOp::FMul,
        (primer_ir::BinaryOp::Divide, Type::Float | Type::Double) => BinaryOp::FDiv,
    }
}
