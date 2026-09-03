use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{BinaryOp, CompareOp, Instruction, Module, Operand, PrintFormat, Slot, Temp, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut slots = Vec::new();
    let mut slot_map = HashMap::new();
    let mut name_counts = HashMap::new();
    collect_slots(
        &program.statements,
        program,
        &mut slots,
        &mut slot_map,
        &mut name_counts,
    );

    let mut lowerer = Lowerer {
        program,
        slots,
        instructions: Vec::new(),
        temp: 0,
        label: 0,
        aggregate_temp: 0,
        slot_map,
        loops: Vec::new(),
    };

    lowerer.lower_statements(&program.statements);

    Module {
        slots: lowerer.slots,
        instructions: lowerer.instructions,
    }
}

struct Lowerer<'a> {
    program: &'a primer_ir::Program,
    slots: Vec<Slot>,
    instructions: Vec<Instruction>,
    temp: usize,
    label: usize,
    aggregate_temp: usize,
    slot_map: HashMap<primer_ir::BindingId, usize>,
    loops: Vec<LoopContext>,
}

#[derive(Debug, Clone, Copy)]
struct LoopContext {
    continue_label: usize,
    break_label: usize,
}

#[derive(Debug, Clone)]
enum Value {
    Scalar { ty: Type, operand: Operand },
    Aggregate { type_id: usize, address: Operand },
}

impl Lowerer<'_> {
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
            primer_ir::StatementKind::Binding { id, ty, value, .. }
            | primer_ir::StatementKind::Assignment { id, ty, value, .. } => {
                let value = self.lower_expr(value);
                let destination = Operand::Slot(self.slot(*id));
                match (*ty, value) {
                    (
                        primer_ir::Type::Named(type_id),
                        Value::Aggregate {
                            type_id: actual,
                            address,
                        },
                    ) => {
                        debug_assert_eq!(type_id.0, actual);
                        self.instructions.push(Instruction::Blit {
                            source: address,
                            destination,
                            size: type_size(self.program, *ty),
                        });
                    }
                    (
                        scalar,
                        Value::Scalar {
                            ty: actual,
                            operand,
                        },
                    ) => {
                        debug_assert_eq!(scalar_type(scalar), actual);
                        self.instructions.push(Instruction::Store {
                            address: destination,
                            ty: actual,
                            value: operand,
                        });
                    }
                    _ => unreachable!("semantic analysis keeps assignment types equal"),
                }
                false
            }

            primer_ir::StatementKind::Print { value } => {
                let Value::Scalar { ty, operand } = self.lower_expr(value) else {
                    unreachable!("semantic analysis rejects aggregate printing")
                };
                self.lower_print(ty, operand);
                false
            }

            primer_ir::StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                let condition = self.lower_scalar_expr(condition);
                let then_label = self.next_label();
                let else_label = self.next_label();
                let end_label = self.next_label();
                self.instructions.push(Instruction::Branch {
                    condition: condition.1,
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
                let condition = self.lower_scalar_expr(condition);
                self.instructions.push(Instruction::Branch {
                    condition: condition.1,
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
                let condition = self.lower_scalar_expr(condition);
                self.instructions.push(Instruction::Branch {
                    condition: condition.1,
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
            primer_ir::StatementKind::Call { .. } | primer_ir::StatementKind::Return { .. } => {
                unreachable!("functions are rejected before QBE lowering")
            }
        }
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr) -> Value {
        match &expr.kind {
            primer_ir::ExprKind::Boolean(value) => Value::Scalar {
                ty: Type::Bool,
                operand: Operand::Boolean(*value),
            },
            primer_ir::ExprKind::Integer(value) => Value::Scalar {
                ty: Type::I64,
                operand: Operand::Integer(*value),
            },
            primer_ir::ExprKind::Float { text } => Value::Scalar {
                ty: scalar_type(expr.ty),
                operand: match expr.ty {
                    primer_ir::Type::F32 => Operand::Float32(text.clone()),
                    primer_ir::Type::F64 => Operand::Float64(text.clone()),
                    _ => unreachable!("a float literal has a float type"),
                },
            },
            primer_ir::ExprKind::Variable { id, .. } => match expr.ty {
                primer_ir::Type::Named(type_id) => Value::Aggregate {
                    type_id: type_id.0,
                    address: Operand::Slot(self.slot(*id)),
                },
                scalar => {
                    let ty = scalar_type(scalar);
                    let dest = self.next_temp();
                    self.instructions.push(Instruction::Load {
                        dest,
                        address: Operand::Slot(self.slot(*id)),
                        ty,
                    });
                    Value::Scalar {
                        ty,
                        operand: Operand::Temp(dest),
                    }
                }
            },
            primer_ir::ExprKind::Unary { op, value } => {
                let (ty, operand) = self.lower_scalar_expr(value);
                let dest = self.next_temp();
                match op {
                    primer_ir::UnaryOp::Negate => self.instructions.push(Instruction::Negate {
                        dest,
                        ty,
                        value: operand,
                    }),
                    primer_ir::UnaryOp::Not => self.instructions.push(Instruction::Not {
                        dest,
                        value: operand,
                    }),
                }
                Value::Scalar {
                    ty,
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::Binary { op, left, right } => {
                let (left_ty, left) = self.lower_scalar_expr(left);
                let (right_ty, right) = self.lower_scalar_expr(right);
                debug_assert_eq!(left_ty, right_ty);
                let dest = self.next_temp();

                if let Some(op) = compare_op(*op) {
                    self.instructions.push(Instruction::Compare {
                        dest,
                        op,
                        operand_ty: left_ty,
                        left,
                        right,
                    });
                } else {
                    self.instructions.push(Instruction::Binary {
                        dest,
                        op: (*op).into(),
                        ty: left_ty,
                        left,
                        right,
                    });
                }

                Value::Scalar {
                    ty: scalar_type(expr.ty),
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::Construct {
                type_id, fields, ..
            } => {
                let slot = self.allocate_aggregate(type_size(self.program, expr.ty));
                for field in fields {
                    let definition = &self.program.type_definitions[type_id.0].fields[field.id.0];
                    let value = self.lower_expr(&field.value);
                    let offset = field_offset(self.program, type_id.0, field.id.0);
                    let destination = self.address(Operand::Slot(slot), offset);
                    match (definition.ty, value) {
                        (
                            primer_ir::Type::Named(nested),
                            Value::Aggregate {
                                type_id: actual,
                                address,
                            },
                        ) => {
                            debug_assert_eq!(nested.0, actual);
                            self.instructions.push(Instruction::Blit {
                                source: address,
                                destination,
                                size: type_size(self.program, definition.ty),
                            });
                        }
                        (scalar, Value::Scalar { ty, operand }) => {
                            debug_assert_eq!(scalar_type(scalar), ty);
                            self.instructions.push(Instruction::Store {
                                address: destination,
                                ty,
                                value: operand,
                            });
                        }
                        _ => unreachable!("semantic analysis keeps field types equal"),
                    }
                }
                Value::Aggregate {
                    type_id: type_id.0,
                    address: Operand::Slot(slot),
                }
            }
            primer_ir::ExprKind::FieldAccess {
                type_id,
                field_id,
                base,
                ..
            } => {
                let Value::Aggregate { address, .. } = self.lower_expr(base) else {
                    unreachable!("semantic analysis requires an aggregate field base")
                };
                let address =
                    self.address(address, field_offset(self.program, type_id.0, field_id.0));
                match expr.ty {
                    primer_ir::Type::Named(nested) => Value::Aggregate {
                        type_id: nested.0,
                        address,
                    },
                    scalar => {
                        let ty = scalar_type(scalar);
                        let dest = self.next_temp();
                        self.instructions
                            .push(Instruction::Load { dest, address, ty });
                        Value::Scalar {
                            ty,
                            operand: Operand::Temp(dest),
                        }
                    }
                }
            }
            primer_ir::ExprKind::Call { .. } => {
                unreachable!("functions are rejected before QBE lowering")
            }
        }
    }

    fn lower_scalar_expr(&mut self, expr: &primer_ir::Expr) -> (Type, Operand) {
        let Value::Scalar { ty, operand } = self.lower_expr(expr) else {
            unreachable!("semantic analysis requires a scalar value here")
        };
        (ty, operand)
    }

    fn lower_print(&mut self, ty: Type, operand: Operand) {
        match ty {
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
                    value: operand,
                });
            }
            Type::I64 => {
                let dest = self.next_temp();
                self.instructions.push(Instruction::CallPrintf {
                    dest,
                    format: PrintFormat::I64,
                    arg_ty: Type::I64,
                    value: operand,
                });
            }
            Type::Single => {
                // C の可変長引数では float が double に拡張される。
                let extended = self.next_temp();
                self.instructions.push(Instruction::ExtendSingleToDouble {
                    dest: extended,
                    value: operand,
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
                    value: operand,
                });
            }
        }
    }

    fn address(&mut self, base: Operand, offset: usize) -> Operand {
        if offset == 0 {
            base
        } else {
            let dest = self.next_temp();
            self.instructions
                .push(Instruction::Address { dest, base, offset });
            Operand::Temp(dest)
        }
    }

    fn allocate_aggregate(&mut self, size: usize) -> usize {
        let id = self.aggregate_temp;
        self.aggregate_temp += 1;
        let slot = self.slots.len();
        self.slots.push(Slot {
            name: format!("aggregate_tmp{id}"),
            size,
        });
        slot
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
    program: &primer_ir::Program,
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
                    size: type_size(program, *ty),
                });
                slot_map.insert(*id, slot);
            }
            primer_ir::StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_slots(then_body, program, slots, slot_map, name_counts);
                collect_slots(else_body, program, slots, slot_map, name_counts);
            }
            primer_ir::StatementKind::While { body, .. } => {
                collect_slots(body, program, slots, slot_map, name_counts);
            }
            primer_ir::StatementKind::For {
                initializer,
                update,
                body,
                ..
            } => {
                collect_slots(
                    std::slice::from_ref(initializer),
                    program,
                    slots,
                    slot_map,
                    name_counts,
                );
                collect_slots(
                    std::slice::from_ref(update),
                    program,
                    slots,
                    slot_map,
                    name_counts,
                );
                collect_slots(body, program, slots, slot_map, name_counts);
            }
            primer_ir::StatementKind::Assignment { .. }
            | primer_ir::StatementKind::Print { .. }
            | primer_ir::StatementKind::Call { .. }
            | primer_ir::StatementKind::Return { .. }
            | primer_ir::StatementKind::Break
            | primer_ir::StatementKind::Continue => {}
        }
    }
}

fn type_size(program: &primer_ir::Program, ty: primer_ir::Type) -> usize {
    match ty {
        primer_ir::Type::Bool
        | primer_ir::Type::I64
        | primer_ir::Type::F32
        | primer_ir::Type::F64 => 8,
        primer_ir::Type::Named(id) => program.type_definitions[id.0]
            .fields
            .iter()
            .map(|field| type_size(program, field.ty))
            .sum(),
    }
}

fn field_offset(program: &primer_ir::Program, type_id: usize, field_id: usize) -> usize {
    program.type_definitions[type_id].fields[..field_id]
        .iter()
        .map(|field| type_size(program, field.ty))
        .sum()
}

fn scalar_type(ty: primer_ir::Type) -> Type {
    match ty {
        primer_ir::Type::Bool => Type::Bool,
        primer_ir::Type::I64 => Type::I64,
        primer_ir::Type::F32 => Type::Single,
        primer_ir::Type::F64 => Type::Double,
        primer_ir::Type::Named(_) => unreachable!("expected a scalar type"),
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
