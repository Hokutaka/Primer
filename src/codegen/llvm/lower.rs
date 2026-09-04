use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{
    BinaryOp, CompareOp, Function, Instruction, Label, Module, Operand, Parameter, PrintFormat,
    Slot, SlotId, Temp, Type, TypeDefinition,
};

pub fn lower(program: &primer_ir::Program) -> Module {
    let functions = program
        .function_definitions
        .iter()
        .map(lower_function)
        .collect();
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
        slot_map,
        instructions: Vec::new(),
        temp: 0,
        label: 0,
        loops: Vec::new(),
    };

    lowerer.lower_statements(&program.statements);

    Module {
        type_definitions: program
            .type_definitions
            .iter()
            .map(|definition| TypeDefinition {
                id: definition.id.0,
                name: definition.name.clone(),
                fields: definition
                    .fields
                    .iter()
                    .map(|field| field.ty.clone().into())
                    .collect(),
            })
            .collect(),
        functions,
        explicit_main: program
            .function_definitions
            .iter()
            .find(|function| function.name == "main")
            .map(|function| function.id.0),
        slots,
        instructions: lowerer.instructions,
    }
}

fn lower_function(function: &primer_ir::FunctionDefinition) -> Function {
    let mut slots = Vec::new();
    let mut slot_map = HashMap::new();
    let mut name_counts = HashMap::new();
    let parameters = function
        .parameters
        .iter()
        .map(|parameter| {
            let slot = SlotId(slots.len());
            slots.push(Slot {
                name: parameter.name.clone(),
                ty: parameter.ty.clone().into(),
            });
            slot_map.insert(parameter.id, slot);
            name_counts.insert(parameter.name.clone(), 1);
            Parameter {
                name: parameter.name.clone(),
                ty: parameter.ty.clone().into(),
                slot,
            }
        })
        .collect();
    collect_slots(&function.body, &mut slots, &mut slot_map, &mut name_counts);

    let mut lowerer = Lowerer {
        slot_map,
        instructions: Vec::new(),
        temp: 0,
        label: 0,
        loops: Vec::new(),
    };
    let terminates = lowerer.lower_statements(&function.body);
    let return_type = match &function.return_type {
        primer_ir::ReturnType::Void => None,
        primer_ir::ReturnType::Value(ty) => Some(ty.clone().into()),
    };
    if !terminates {
        lowerer
            .instructions
            .push(Instruction::Return { value: None });
    }

    Function {
        id: function.id.0,
        name: function.name.clone(),
        parameters,
        return_type,
        slots,
        instructions: lowerer.instructions,
    }
}

struct Lowerer {
    slot_map: HashMap<primer_ir::BindingId, SlotId>,
    instructions: Vec<Instruction>,
    temp: usize,
    label: usize,
    loops: Vec<LoopContext>,
}

#[derive(Debug, Clone, Copy)]
struct LoopContext {
    continue_label: Label,
    break_label: Label,
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
                let slot = self.slot(*id);
                let ty = ty.clone().into();

                let value = self.lower_expr(value);

                self.instructions.push(Instruction::Store {
                    ty,
                    value: value.operand,
                    slot,
                });
                false
            }

            primer_ir::StatementKind::Assignment { target, value } => {
                let slot = self.slot(target.id);
                let root_ty: Type = target.root_ty.clone().into();
                if target.projections.is_empty() {
                    let value = self.lower_expr(value);
                    self.instructions.push(Instruction::Store {
                        ty: root_ty,
                        value: value.operand,
                        slot,
                    });
                    return false;
                }

                let root = self.next_temp();
                self.instructions.push(Instruction::Load {
                    dest: root,
                    ty: root_ty.clone(),
                    slot,
                });
                let mut current = Value {
                    ty: root_ty.clone(),
                    operand: Operand::Temp(root),
                };
                let mut parents = Vec::with_capacity(target.projections.len());
                for projection in &target.projections {
                    let primer_ir::AssignmentProjection::Index {
                        index,
                        element,
                        length,
                        ..
                    } = projection;
                    let index = self.lower_expr(index);
                    let element: Type = element.clone().into();
                    let dest = self.next_temp();
                    self.instructions.push(Instruction::ArrayGet {
                        dest,
                        element: element.clone(),
                        length: *length,
                        array: current.operand,
                        index: index.operand,
                    });
                    parents.push((current.operand, index.operand, element.clone(), *length));
                    current = Value {
                        ty: element,
                        operand: Operand::Temp(dest),
                    };
                }

                let mut updated = self.lower_expr(value).operand;
                for (array, index, element, length) in parents.into_iter().rev() {
                    let dest = self.next_temp();
                    self.instructions.push(Instruction::ArraySet {
                        dest,
                        element: element.clone(),
                        length,
                        array,
                        index,
                        value: updated,
                    });
                    updated = Operand::Temp(dest);
                }
                self.instructions.push(Instruction::Store {
                    ty: root_ty,
                    value: updated,
                    slot,
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
                    else_label: if else_body.is_empty() {
                        end_label
                    } else {
                        else_label
                    },
                });
                self.instructions.push(Instruction::Label {
                    id: then_label,
                    name: "if_then",
                });
                let then_terminates = self.lower_statements(then_body);
                if !then_terminates {
                    self.instructions
                        .push(Instruction::Jump { label: end_label });
                }

                if else_body.is_empty() {
                    self.instructions.push(Instruction::Label {
                        id: end_label,
                        name: "if_end",
                    });
                    false
                } else {
                    self.instructions.push(Instruction::Label {
                        id: else_label,
                        name: "if_else",
                    });
                    let else_terminates = self.lower_statements(else_body);
                    if !else_terminates {
                        self.instructions
                            .push(Instruction::Jump { label: end_label });
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
            }

            primer_ir::StatementKind::While { condition, body } => {
                let condition_label = self.next_label();
                let body_label = self.next_label();
                let end_label = self.next_label();

                self.instructions.push(Instruction::Jump {
                    label: condition_label,
                });
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
                    self.instructions.push(Instruction::Jump {
                        label: condition_label,
                    });
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

                self.instructions.push(Instruction::Jump {
                    label: condition_label,
                });
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
                    self.instructions.push(Instruction::Jump {
                        label: update_label,
                    });
                }
                self.instructions.push(Instruction::Label {
                    id: update_label,
                    name: "for_update",
                });
                self.lower_statement(update);
                self.instructions.push(Instruction::Jump {
                    label: condition_label,
                });
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
                self.instructions.push(Instruction::Jump { label: target });
                true
            }

            primer_ir::StatementKind::Continue => {
                let target = self
                    .loops
                    .last()
                    .expect("semantic analysis rejects continue outside a loop")
                    .continue_label;
                self.instructions.push(Instruction::Jump { label: target });
                true
            }
            primer_ir::StatementKind::Call {
                function_id,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| {
                        let value = self.lower_expr(argument);
                        (value.ty, value.operand)
                    })
                    .collect();
                self.instructions.push(Instruction::Call {
                    dest: None,
                    function_id: function_id.0,
                    return_type: None,
                    arguments,
                });
                false
            }
            primer_ir::StatementKind::Return { value } => {
                let value = value.as_ref().map(|value| {
                    let value = self.lower_expr(value);
                    (value.ty, value.operand)
                });
                self.instructions.push(Instruction::Return { value });
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

                primer_ir::Type::Bool => {
                    unreachable!("boolean cannot be lowered as float")
                }
                primer_ir::Type::Named(_) | primer_ir::Type::Array { .. } => {
                    unreachable!("a float literal cannot have an aggregate type")
                }
            },

            primer_ir::ExprKind::Variable { id, .. } => {
                let ty: Type = expr.ty.clone().into();
                let dest = self.next_temp();
                let slot = self.slot(*id);

                self.instructions.push(Instruction::Load {
                    dest,
                    ty: ty.clone(),
                    slot,
                });

                Value {
                    ty,
                    operand: Operand::Temp(dest),
                }
            }

            primer_ir::ExprKind::Unary { op, value } => {
                let value = self.lower_expr(value);
                let dest = self.next_temp();

                match (*op, &value.ty) {
                    (primer_ir::UnaryOp::Negate, Type::I64) => {
                        self.instructions.push(Instruction::Binary {
                            dest,
                            op: BinaryOp::CheckedI64Sub,
                            ty: Type::I64,
                            left: Operand::Integer(0),
                            right: value.operand,
                        });
                    }

                    (primer_ir::UnaryOp::Negate, Type::Float | Type::Double) => {
                        self.instructions.push(Instruction::FNeg {
                            dest,
                            ty: value.ty.clone(),
                            value: value.operand,
                        });
                    }
                    (primer_ir::UnaryOp::Not, Type::Bool) => {
                        self.instructions.push(Instruction::Binary {
                            dest,
                            op: BinaryOp::Xor,
                            ty: Type::Bool,
                            left: value.operand,
                            right: Operand::Boolean(true),
                        });
                    }
                    (primer_ir::UnaryOp::Negate, Type::Bool)
                    | (primer_ir::UnaryOp::Not, Type::I64 | Type::Float | Type::Double) => {
                        unreachable!("semantic analysis rejects invalid unary operands");
                    }
                    (_, Type::Named(_) | Type::Array { .. }) => {
                        unreachable!("semantic analysis rejects aggregate unary operands");
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
                        operand_ty: left.ty.clone(),
                        left: left.operand,
                        right: right.operand,
                    });
                } else {
                    self.instructions.push(Instruction::Binary {
                        dest,
                        op: binary_op(*op, &left.ty),
                        ty: left.ty.clone(),
                        left: left.operand,
                        right: right.operand,
                    });
                }

                Value {
                    ty: expr.ty.clone().into(),
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::Construct {
                type_id, fields, ..
            } => {
                let ty = Type::Named(type_id.0);
                let mut aggregate = Operand::Poison;
                for field in fields {
                    let value = self.lower_expr(&field.value);
                    let dest = self.next_temp();
                    self.instructions.push(Instruction::InsertValue {
                        dest,
                        ty: ty.clone(),
                        aggregate,
                        value_ty: value.ty,
                        value: value.operand,
                        field: field.id.0,
                    });
                    aggregate = Operand::Temp(dest);
                }
                Value {
                    ty,
                    operand: aggregate,
                }
            }
            primer_ir::ExprKind::FieldAccess {
                type_id,
                field_id,
                base,
                ..
            } => {
                let aggregate = self.lower_expr(base);
                let dest = self.next_temp();
                self.instructions.push(Instruction::ExtractValue {
                    dest,
                    ty: Type::Named(type_id.0),
                    aggregate: aggregate.operand,
                    field: field_id.0,
                });
                Value {
                    ty: expr.ty.clone().into(),
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::Array(values) => {
                let ty: Type = expr.ty.clone().into();
                let mut aggregate = Operand::Poison;
                for (index, value) in values.iter().enumerate() {
                    let value = self.lower_expr(value);
                    let dest = self.next_temp();
                    self.instructions.push(Instruction::InsertValue {
                        dest,
                        ty: ty.clone(),
                        aggregate,
                        value_ty: value.ty,
                        value: value.operand,
                        field: index,
                    });
                    aggregate = Operand::Temp(dest);
                }
                Value {
                    ty,
                    operand: aggregate,
                }
            }
            primer_ir::ExprKind::Index { base, index } => {
                let array = self.lower_expr(base);
                let index = self.lower_expr(index);
                let Type::Array { element, length } = &array.ty else {
                    unreachable!("indexed expression must have an array base")
                };
                let dest = self.next_temp();
                self.instructions.push(Instruction::ArrayGet {
                    dest,
                    element: (**element).clone(),
                    length: *length,
                    array: array.operand,
                    index: index.operand,
                });
                Value {
                    ty: expr.ty.clone().into(),
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::Call {
                function_id,
                arguments,
                ..
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| {
                        let value = self.lower_expr(argument);
                        (value.ty, value.operand)
                    })
                    .collect();
                let ty: Type = expr.ty.clone().into();
                let dest = self.next_temp();
                self.instructions.push(Instruction::Call {
                    dest: Some(dest),
                    function_id: function_id.0,
                    return_type: Some(ty.clone()),
                    arguments,
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
            Type::Bool => {
                let dest = self.next_temp();
                self.instructions.push(Instruction::SelectBoolText {
                    dest,
                    value: value.operand,
                });
                self.instructions.push(Instruction::CallPuts {
                    value: Operand::Temp(dest),
                });
            }

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
            Type::Named(_) | Type::Array { .. } => {
                unreachable!("semantic analysis rejects aggregate printing")
            }
        }
    }

    fn slot(&self, id: primer_ir::BindingId) -> SlotId {
        self.slot_map
            .get(&id)
            .copied()
            .expect("binding must have an LLVM slot")
    }

    fn next_temp(&mut self) -> Temp {
        let temp = Temp(self.temp);
        self.temp += 1;
        temp
    }

    fn next_label(&mut self) -> Label {
        let label = Label(self.label);
        self.label += 1;
        label
    }
}

fn collect_slots(
    statements: &[primer_ir::Statement],
    slots: &mut Vec<Slot>,
    slot_map: &mut HashMap<primer_ir::BindingId, SlotId>,
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

                let slot = SlotId(slots.len());
                slots.push(Slot {
                    name: lowered_name,
                    ty: ty.clone().into(),
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
            | primer_ir::StatementKind::Call { .. }
            | primer_ir::StatementKind::Return { .. }
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
            primer_ir::Type::F32 => Self::Float,
            primer_ir::Type::F64 => Self::Double,
            primer_ir::Type::Named(id) => Self::Named(id.0),
            primer_ir::Type::Array { element, length } => Self::Array {
                element: Box::new((*element).into()),
                length,
            },
        }
    }
}

fn binary_op(op: primer_ir::BinaryOp, ty: &Type) -> BinaryOp {
    match (op, ty) {
        (primer_ir::BinaryOp::Add, Type::I64) => BinaryOp::CheckedI64Add,
        (primer_ir::BinaryOp::Subtract, Type::I64) => BinaryOp::CheckedI64Sub,
        (primer_ir::BinaryOp::Multiply, Type::I64) => BinaryOp::CheckedI64Mul,
        (primer_ir::BinaryOp::Divide, Type::I64) => BinaryOp::CheckedI64Div,

        (primer_ir::BinaryOp::Add, Type::Float | Type::Double) => BinaryOp::FAdd,
        (primer_ir::BinaryOp::Subtract, Type::Float | Type::Double) => BinaryOp::FSub,
        (primer_ir::BinaryOp::Multiply, Type::Float | Type::Double) => BinaryOp::FMul,
        (primer_ir::BinaryOp::Divide, Type::Float | Type::Double) => BinaryOp::FDiv,

        (primer_ir::BinaryOp::Add, Type::Bool)
        | (primer_ir::BinaryOp::Subtract, Type::Bool)
        | (primer_ir::BinaryOp::Multiply, Type::Bool)
        | (primer_ir::BinaryOp::Divide, Type::Bool)
        | (
            primer_ir::BinaryOp::Add
            | primer_ir::BinaryOp::Subtract
            | primer_ir::BinaryOp::Multiply
            | primer_ir::BinaryOp::Divide,
            Type::Named(_),
        )
        | (
            primer_ir::BinaryOp::Add
            | primer_ir::BinaryOp::Subtract
            | primer_ir::BinaryOp::Multiply
            | primer_ir::BinaryOp::Divide,
            Type::Array { .. },
        )
        | (primer_ir::BinaryOp::Equal, _)
        | (primer_ir::BinaryOp::NotEqual, _)
        | (primer_ir::BinaryOp::Less, _)
        | (primer_ir::BinaryOp::LessEqual, _)
        | (primer_ir::BinaryOp::Greater, _)
        | (primer_ir::BinaryOp::GreaterEqual, _) => {
            unreachable!("comparison and invalid arithmetic use separate lowering")
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
