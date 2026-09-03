use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{BinaryOp, CompareOp, Instruction, Module, Operand, PrintFormat, Slot, Temp, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut slots = Vec::new();
    let mut slot_map = HashMap::new();
    let mut name_counts = HashMap::new();
    collect_slots(
        &program.statements,
        &mut slots,
        &mut slot_map,
        &mut name_counts,
    );

    let mut lowerer = Lowerer {
        instructions: Vec::new(),
        temp: 0,
        label: 0,
        slot_map,
        loops: Vec::new(),
    };

    lowerer.lower_statements(&program.statements);

    Module {
        slots,
        instructions: lowerer.instructions,
    }
}

struct Lowerer {
    instructions: Vec<Instruction>,
    temp: usize,
    label: usize,
    slot_map: HashMap<primer_ir::BindingId, usize>,
    loops: Vec<LoopContext>,
}

#[derive(Debug, Clone, Copy)]
struct LoopContext {
    continue_label: usize,
    break_label: usize,
}

#[derive(Debug, Clone)]
struct Value {
    ty: Type,
    operand: Operand,
}

impl Lowerer {
    fn lower_statements(&mut self, statements: &[primer_ir::Statement]) -> bool {
        for statement in statements {
            if self.lower_statement(statement) {
                return true;
            }
        }

        false
    }

    fn lower_statement(&mut self, statement: &primer_ir::Statement) -> bool {
        match &statement.kind {
            primer_ir::StatementKind::Binding { id, ty, value, .. } => {
                let value = self.lower_expr(value);
                self.instructions.push(Instruction::Store {
                    slot: self.slot(*id),
                    ty: (*ty).into(),
                    value: value.operand,
                });
                false
            }

            primer_ir::StatementKind::Assignment { id, ty, value, .. } => {
                let value = self.lower_expr(value);
                self.instructions.push(Instruction::Store {
                    slot: self.slot(*id),
                    ty: (*ty).into(),
                    value: value.operand,
                });
                false
            }

            primer_ir::StatementKind::Print { value } => {
                let value = self.lower_expr(value);
                self.lower_print(value);
                false
            }

            primer_ir::StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition = self.lower_expr(condition);
                let then_label = self.next_label();
                let else_label = self.next_label();
                let end_label = self.next_label();
                self.instructions.push(Instruction::Branch {
                    condition: condition.operand,
                    then_label,
                    else_label,
                });
                self.instructions.push(Instruction::Label {
                    id: then_label,
                    name: "if_then",
                });
                let then_terminates = self.lower_statements(then_body);
                if !then_terminates {
                    self.instructions.push(Instruction::Jump(end_label));
                }
                self.instructions.push(Instruction::Label {
                    id: else_label,
                    name: "if_else",
                });
                let else_terminates = self.lower_statements(else_body);
                if !else_terminates {
                    self.instructions.push(Instruction::Jump(end_label));
                }

                if then_terminates && else_terminates {
                    true
                } else {
                    self.instructions.push(Instruction::Label {
                        id: end_label,
                        name: "if_end",
                    });
                    false
                }
            }

            primer_ir::StatementKind::While { condition, body } => {
                let condition_label = self.next_label();
                let body_label = self.next_label();
                let end_label = self.next_label();

                self.instructions.push(Instruction::Jump(condition_label));
                self.instructions.push(Instruction::Label {
                    id: condition_label,
                    name: "while_condition",
                });

                let condition = self.lower_expr(condition);
                self.instructions.push(Instruction::Branch {
                    condition: condition.operand,
                    then_label: body_label,
                    else_label: end_label,
                });
                self.instructions.push(Instruction::Label {
                    id: body_label,
                    name: "while_body",
                });

                self.loops.push(LoopContext {
                    continue_label: condition_label,
                    break_label: end_label,
                });
                let body_terminates = self.lower_statements(body);
                self.loops.pop().expect("while loop context must exist");

                if !body_terminates {
                    self.instructions.push(Instruction::Jump(condition_label));
                }
                self.instructions.push(Instruction::Label {
                    id: end_label,
                    name: "while_end",
                });
                false
            }

            primer_ir::StatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                self.lower_statement(initializer);

                let condition_label = self.next_label();
                let body_label = self.next_label();
                let update_label = self.next_label();
                let end_label = self.next_label();

                self.instructions.push(Instruction::Jump(condition_label));
                self.instructions.push(Instruction::Label {
                    id: condition_label,
                    name: "for_condition",
                });
                let condition = self.lower_expr(condition);
                self.instructions.push(Instruction::Branch {
                    condition: condition.operand,
                    then_label: body_label,
                    else_label: end_label,
                });
                self.instructions.push(Instruction::Label {
                    id: body_label,
                    name: "for_body",
                });

                self.loops.push(LoopContext {
                    continue_label: update_label,
                    break_label: end_label,
                });
                let body_terminates = self.lower_statements(body);
                self.loops.pop().expect("for loop context must exist");

                if !body_terminates {
                    self.instructions.push(Instruction::Jump(update_label));
                }
                self.instructions.push(Instruction::Label {
                    id: update_label,
                    name: "for_update",
                });
                self.lower_statement(update);
                self.instructions.push(Instruction::Jump(condition_label));
                self.instructions.push(Instruction::Label {
                    id: end_label,
                    name: "for_end",
                });
                false
            }

            primer_ir::StatementKind::Break => {
                let target = self
                    .loops
                    .last()
                    .expect("semantic analysis rejects break outside a loop")
                    .break_label;
                self.instructions.push(Instruction::Jump(target));
                true
            }

            primer_ir::StatementKind::Continue => {
                let target = self
                    .loops
                    .last()
                    .expect("semantic analysis rejects continue outside a loop")
                    .continue_label;
                self.instructions.push(Instruction::Jump(target));
                true
            }
        }
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr) -> Value {
        match &expr.kind {
            primer_ir::ExprKind::Boolean(value) => Value {
                ty: Type::Bool,
                operand: Operand::Boolean(*value),
            },

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

                primer_ir::Type::Bool => {
                    unreachable!("boolean cannot be lowered as float")
                }
            },

            primer_ir::ExprKind::Variable { id, .. } => {
                let dest = self.next_temp();
                let ty = expr.ty.into();
                self.instructions.push(Instruction::Load {
                    dest,
                    slot: self.slot(*id),
                    ty,
                });
                Value {
                    ty,
                    operand: Operand::Temp(dest),
                }
            }

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
                    primer_ir::UnaryOp::Not => {
                        self.instructions.push(Instruction::Not {
                            dest,
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
                let left = self.lower_expr(left);
                let right = self.lower_expr(right);
                let dest = self.next_temp();

                if let Some(op) = compare_op(*op) {
                    self.instructions.push(Instruction::Compare {
                        dest,
                        op,
                        operand_ty: left.ty,
                        left: left.operand,
                        right: right.operand,
                    });
                } else {
                    self.instructions.push(Instruction::Binary {
                        dest,
                        op: (*op).into(),
                        ty: left.ty,
                        left: left.operand,
                        right: right.operand,
                    });
                }

                Value {
                    ty: expr.ty.into(),
                    operand: Operand::Temp(dest),
                }
            }
        }
    }

    fn lower_print(&mut self, value: Value) {
        match value.ty {
            Type::Bool => {
                let offset = self.next_temp();
                let scaled_offset = self.next_temp();
                let address = self.next_temp();
                let text = self.next_temp();
                let result = self.next_temp();

                self.instructions.push(Instruction::CallPrintBool {
                    offset,
                    scaled_offset,
                    address,
                    text,
                    result,
                    value: value.operand,
                });
            }

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

    fn next_label(&mut self) -> usize {
        let label = self.label;
        self.label += 1;
        label
    }

    fn slot(&self, id: primer_ir::BindingId) -> usize {
        self.slot_map[&id]
    }
}

fn collect_slots(
    statements: &[primer_ir::Statement],
    slots: &mut Vec<Slot>,
    slot_map: &mut HashMap<primer_ir::BindingId, usize>,
    name_counts: &mut HashMap<String, usize>,
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
                let slot = slots.len();
                slots.push(Slot {
                    name: lowered_name,
                    ty: (*ty).into(),
                });
                slot_map.insert(*id, slot);
            }
            primer_ir::StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_slots(then_body, slots, slot_map, name_counts);
                collect_slots(else_body, slots, slot_map, name_counts);
            }
            primer_ir::StatementKind::While { body, .. } => {
                collect_slots(body, slots, slot_map, name_counts);
            }
            primer_ir::StatementKind::For {
                initializer,
                update,
                body,
                ..
            } => {
                collect_slots(
                    std::slice::from_ref(initializer),
                    slots,
                    slot_map,
                    name_counts,
                );
                collect_slots(std::slice::from_ref(update), slots, slot_map, name_counts);
                collect_slots(body, slots, slot_map, name_counts);
            }
            primer_ir::StatementKind::Assignment { .. }
            | primer_ir::StatementKind::Print { .. }
            | primer_ir::StatementKind::Break
            | primer_ir::StatementKind::Continue => {}
        }
    }
}

impl From<primer_ir::Type> for Type {
    fn from(value: primer_ir::Type) -> Self {
        match value {
            primer_ir::Type::Bool => Self::Bool,
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
            primer_ir::BinaryOp::Equal
            | primer_ir::BinaryOp::NotEqual
            | primer_ir::BinaryOp::Less
            | primer_ir::BinaryOp::LessEqual
            | primer_ir::BinaryOp::Greater
            | primer_ir::BinaryOp::GreaterEqual => {
                unreachable!("comparisons use a dedicated QBE instruction")
            }
        }
    }
}

const fn compare_op(op: primer_ir::BinaryOp) -> Option<CompareOp> {
    match op {
        primer_ir::BinaryOp::Add
        | primer_ir::BinaryOp::Subtract
        | primer_ir::BinaryOp::Multiply
        | primer_ir::BinaryOp::Divide => None,
        primer_ir::BinaryOp::Equal => Some(CompareOp::Equal),
        primer_ir::BinaryOp::NotEqual => Some(CompareOp::NotEqual),
        primer_ir::BinaryOp::Less => Some(CompareOp::Less),
        primer_ir::BinaryOp::LessEqual => Some(CompareOp::LessEqual),
        primer_ir::BinaryOp::Greater => Some(CompareOp::Greater),
        primer_ir::BinaryOp::GreaterEqual => Some(CompareOp::GreaterEqual),
    }
}
