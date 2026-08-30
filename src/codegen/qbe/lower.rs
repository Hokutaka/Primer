use crate::ir as primer_ir;

use super::ir::{BinaryOp, Instruction, Module, Operand, PrintFormat, Temp, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut lowerer = Lowerer {
        instructions: Vec::new(),
        temp: 0,
    };

    for statement in &program.statements {
        lowerer.lower_statement(statement);
    }

    Module {
        instructions: lowerer.instructions,
    }
}

struct Lowerer {
    instructions: Vec<Instruction>,
    temp: usize,
}

#[derive(Debug, Clone)]
struct Value {
    ty: Type,
    operand: Operand,
}

impl Lowerer {
    fn lower_statement(&mut self, statement: &primer_ir::Statement) {
        match &statement.kind {
            primer_ir::StatementKind::Binding {
                name, ty, value, ..
            } => {
                let value = self.lower_expr(value);

                self.instructions.push(Instruction::Copy {
                    name: name.clone(),
                    ty: (*ty).into(),
                    value: value.operand,
                });
            }

            primer_ir::StatementKind::Print { value } => {
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
                primer_ir::Type::F32 => Value {
                    ty: Type::Single,
                    operand: Operand::Float32(text.clone()),
                },

                primer_ir::Type::F64 => Value {
                    ty: Type::Double,
                    operand: Operand::Float64(text.clone()),
                },

                primer_ir::Type::I64 => {
                    unreachable!("integer cannot be lowered as float")
                }
            },

            primer_ir::ExprKind::Variable(name) => Value {
                ty: expr.ty.into(),
                operand: Operand::Binding(name.clone()),
            },

            primer_ir::ExprKind::Unary { op, value } => {
                let value = self.lower_expr(value);
                let dest = self.next_temp();

                match op {
                    primer_ir::UnaryOp::Negate => {
                        self.instructions.push(Instruction::Negate {
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
                    op: (*op).into(),
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
                let dest = self.next_temp();

                self.instructions.push(Instruction::CallPrintf {
                    dest,
                    format: PrintFormat::I64,
                    arg_ty: Type::I64,
                    value: value.operand,
                });
            }

            Type::Single => {
                // C varargs promote float to double.
                let extended = self.next_temp();

                self.instructions.push(Instruction::ExtendSingleToDouble {
                    dest: extended,
                    value: value.operand,
                });

                let dest = self.next_temp();

                self.instructions.push(Instruction::CallPrintf {
                    dest,
                    format: PrintFormat::F32,
                    arg_ty: Type::Double,
                    value: Operand::Temp(extended),
                });
            }

            Type::Double => {
                let dest = self.next_temp();

                self.instructions.push(Instruction::CallPrintf {
                    dest,
                    format: PrintFormat::F64,
                    arg_ty: Type::Double,
                    value: value.operand,
                });
            }
        }
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
            primer_ir::Type::F32 => Self::Single,
            primer_ir::Type::F64 => Self::Double,
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
