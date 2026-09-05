use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{
    BinaryOp, CompareOp, Function, Instruction, Module, Operand, Parameter, ParameterPassing,
    PrintFormat, Slot, Temp, Type,
};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut strings = Vec::new();
    let functions = program
        .function_definitions
        .iter()
        .map(|function| lower_function(program, function, &mut strings))
        .collect();
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
        strings: &mut strings,
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
        target: None,
        uses_strings: crate::codegen::support::first_string_span(program).is_some(),
        functions,
        explicit_main: program
            .function_definitions
            .iter()
            .find(|function| function.name == "main")
            .map(|function| function.id.0),
        slots: lowerer.slots,
        instructions: lowerer.instructions,
        strings,
    }
}

fn lower_function(
    program: &primer_ir::Program,
    function: &primer_ir::FunctionDefinition,
    strings: &mut Vec<String>,
) -> Function {
    let mut slots = Vec::new();
    let mut slot_map = HashMap::new();
    let mut name_counts = HashMap::new();
    let parameters = function
        .parameters
        .iter()
        .map(|parameter| {
            let slot = slots.len();
            slots.push(Slot {
                name: parameter.name.clone(),
                size: type_size(program, &parameter.ty),
            });
            slot_map.insert(parameter.id, slot);
            name_counts.insert(parameter.name.clone(), 1);
            Parameter {
                name: parameter.name.clone(),
                passing: match &parameter.ty {
                    primer_ir::Type::String
                    | primer_ir::Type::Bool
                    | primer_ir::Type::Integer(_)
                    | primer_ir::Type::F32
                    | primer_ir::Type::F64 => ParameterPassing::Scalar(scalar_type(&parameter.ty)),
                    primer_ir::Type::Named(_) | primer_ir::Type::Array { .. } => {
                        ParameterPassing::Aggregate {
                            size: type_size(program, &parameter.ty),
                        }
                    }
                },
                slot,
            }
        })
        .collect();
    collect_slots(
        &function.body,
        program,
        &mut slots,
        &mut slot_map,
        &mut name_counts,
    );

    let mut lowerer = Lowerer {
        strings,
        program,
        slots,
        instructions: Vec::new(),
        temp: 0,
        label: 0,
        aggregate_temp: 0,
        slot_map,
        loops: Vec::new(),
    };
    let terminates = lowerer.lower_statements(&function.body);
    if !terminates {
        lowerer
            .instructions
            .push(Instruction::Return { value: None });
    }

    Function {
        id: function.id.0,
        name: function.name.clone(),
        parameters,
        return_type: match &function.return_type {
            primer_ir::ReturnType::Void => None,
            primer_ir::ReturnType::Value(
                ty @ (primer_ir::Type::String
                | primer_ir::Type::Bool
                | primer_ir::Type::Integer(_)
                | primer_ir::Type::F32
                | primer_ir::Type::F64),
            ) => Some(scalar_type(ty)),
            primer_ir::ReturnType::Value(
                primer_ir::Type::Named(_) | primer_ir::Type::Array { .. },
            ) => None,
        },
        aggregate_return_size: match &function.return_type {
            primer_ir::ReturnType::Value(
                ty @ (primer_ir::Type::Named(_) | primer_ir::Type::Array { .. }),
            ) => Some(type_size(program, ty)),
            primer_ir::ReturnType::Void
            | primer_ir::ReturnType::Value(
                primer_ir::Type::String
                | primer_ir::Type::Bool
                | primer_ir::Type::Integer(_)
                | primer_ir::Type::F32
                | primer_ir::Type::F64,
            ) => None,
        },
        slots: lowerer.slots,
        instructions: lowerer.instructions,
    }
}

struct Lowerer<'a> {
    strings: &'a mut Vec<String>,
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
    Scalar {
        ty: Type,
        operand: Operand,
    },
    Aggregate {
        type_id: usize,
        address: Operand,
    },
    Array {
        element: ArrayElement,
        length: usize,
        address: Operand,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ArrayElement {
    Scalar(Type),
    Named(usize),
    Array {
        element: Box<ArrayElement>,
        length: usize,
    },
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
            primer_ir::StatementKind::Binding { id, ty, value, .. } => {
                let value = self.lower_expr(value);
                let destination = Operand::Slot(self.slot(*id));
                self.store_value(ty, value, destination);
                false
            }

            primer_ir::StatementKind::Assignment { target, value } => {
                let mut destination = Operand::Slot(self.slot(target.id));
                for projection in &target.projections {
                    let primer_ir::AssignmentProjection::Index {
                        index,
                        element,
                        length,
                        ..
                    } = projection;
                    destination = self.lower_checked_array_address(
                        destination,
                        &array_element_type(element),
                        *length,
                        index,
                    );
                }
                let value = self.lower_expr(value);
                self.store_value(&target.ty, value, destination);
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
            primer_ir::StatementKind::Call {
                function_id,
                arguments,
                ..
            } => {
                self.lower_call(function_id.0, arguments, None);
                false
            }
            primer_ir::StatementKind::Return { value } => {
                match value.as_ref().map(|value| (value, self.lower_expr(value))) {
                    Some((_, Value::Scalar { ty, operand })) => {
                        self.instructions.push(Instruction::Return {
                            value: Some((ty, operand)),
                        });
                    }
                    Some((
                        value,
                        Value::Aggregate { address, .. } | Value::Array { address, .. },
                    )) => {
                        self.instructions.push(Instruction::Blit {
                            source: address,
                            destination: Operand::ReturnPointer,
                            size: type_size(self.program, &value.ty),
                        });
                        self.instructions.push(Instruction::Return { value: None });
                    }
                    None => self.instructions.push(Instruction::Return { value: None }),
                }
                true
            }
        }
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr) -> Value {
        let value = self.lower_expr_unchecked(expr);
        if let Some(ty) = super::super::integer_range_check(expr) {
            let Value::Scalar { operand, .. } = value else {
                unreachable!()
            };
            let dest = self.next_temp();
            self.instructions.push(Instruction::CheckIntegerRange {
                dest,
                value: operand,
                ty,
            });
            return Value::Scalar {
                ty: Type::I64,
                operand: Operand::Temp(dest),
            };
        }
        value
    }

    fn lower_expr_unchecked(&mut self, expr: &primer_ir::Expr) -> Value {
        match &expr.kind {
            primer_ir::ExprKind::StringByteLength { value } => {
                let value = self.lower_expr(value);
                let Value::Scalar {
                    ty,
                    operand: address,
                } = value
                else {
                    unreachable!("byte_len operand is a string")
                };
                debug_assert_eq!(ty, Type::String);
                let dest = self.next_temp();
                self.instructions.push(Instruction::Load {
                    dest,
                    address,
                    ty: Type::I64,
                });
                Value::Scalar {
                    ty: Type::I64,
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::String(value) => {
                let id = self.strings.len();
                self.strings.push(value.clone());
                Value::Scalar {
                    ty: Type::String,
                    operand: Operand::String(id),
                }
            }
            primer_ir::ExprKind::ConvertNumeric {
                value, from, to, ..
            } => {
                let (ty, value) = self.lower_scalar_expr(value);
                if from == to {
                    return Value::Scalar { ty, operand: value };
                }
                let dest = self.next_temp();
                self.instructions.push(Instruction::ConvertNumeric {
                    dest,
                    value,
                    conversion: crate::codegen::NumericConversion {
                        from: *from,
                        to: *to,
                    },
                });
                Value::Scalar {
                    ty: scalar_type(&expr.ty),
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::ConvertInteger { value, .. } => self.lower_expr(value),
            primer_ir::ExprKind::Boolean(value) => Value::Scalar {
                ty: Type::Bool,
                operand: Operand::Boolean(*value),
            },
            primer_ir::ExprKind::Integer(value) => Value::Scalar {
                ty: Type::I64,
                operand: Operand::Integer(*value),
            },
            primer_ir::ExprKind::Float { text } => Value::Scalar {
                ty: scalar_type(&expr.ty),
                operand: match &expr.ty {
                    primer_ir::Type::F32 => Operand::Float32(text.clone()),
                    primer_ir::Type::F64 => Operand::Float64(text.clone()),
                    _ => unreachable!("a float literal has a float type"),
                },
            },
            primer_ir::ExprKind::Variable { id, .. } => match &expr.ty {
                primer_ir::Type::Named(type_id) => Value::Aggregate {
                    type_id: type_id.0,
                    address: Operand::Slot(self.slot(*id)),
                },
                primer_ir::Type::Array { element, length } => Value::Array {
                    element: array_element_type(element),
                    length: *length,
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
                match (op, ty) {
                    (primer_ir::UnaryOp::BitNot, _) => {
                        self.instructions.push(Instruction::IntegerBinary {
                            dest,
                            op: crate::codegen::IntegerBinaryOp::BitXor,
                            ty: crate::codegen::integer_type(&expr.ty),
                            left: operand,
                            right: Operand::Integer(crate::codegen::complement_mask(&expr.ty)),
                        })
                    }
                    (primer_ir::UnaryOp::Negate, Type::I64) => {
                        self.instructions.push(Instruction::CheckedI64Negate {
                            dest,
                            value: operand,
                        })
                    }
                    (primer_ir::UnaryOp::Negate, _) => {
                        self.instructions.push(Instruction::Negate {
                            dest,
                            ty,
                            value: operand,
                        })
                    }
                    (primer_ir::UnaryOp::Not, _) => self.instructions.push(Instruction::Not {
                        dest,
                        value: operand,
                    }),
                }
                Value::Scalar {
                    ty,
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::Logical { op, left, right } => {
                let (_, left) = self.lower_scalar_expr(left);
                let rhs_label = self.next_label();
                let end_label = self.next_label();
                let slot = self.slots.len();
                let mut name = format!("logical_result{slot}");
                while self.slots.iter().any(|slot| slot.name == name) {
                    name.push('_');
                }
                self.slots.push(Slot { name, size: 8 });
                self.instructions.push(Instruction::Store {
                    ty: Type::Bool,
                    value: left.clone(),
                    address: Operand::Slot(slot),
                });
                let (then_label, else_label) = match op {
                    primer_ir::LogicalOp::And => (rhs_label, end_label),
                    primer_ir::LogicalOp::Or => (end_label, rhs_label),
                };
                self.instructions.push(Instruction::Branch {
                    condition: left,
                    then_label,
                    else_label,
                });
                self.instructions.push(Instruction::Label {
                    id: rhs_label,
                    name: "logical_rhs",
                });
                let (_, right) = self.lower_scalar_expr(right);
                self.instructions.push(Instruction::Store {
                    ty: Type::Bool,
                    value: right,
                    address: Operand::Slot(slot),
                });
                self.instructions.push(Instruction::Jump(end_label));
                self.instructions.push(Instruction::Label {
                    id: end_label,
                    name: "logical_end",
                });
                let dest = self.next_temp();
                self.instructions.push(Instruction::Load {
                    dest,
                    ty: Type::Bool,
                    address: Operand::Slot(slot),
                });
                Value::Scalar {
                    ty: Type::Bool,
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::Binary { op, left, right } => {
                let (left_ty, left) = self.lower_scalar_expr(left);
                let (right_ty, right) = self.lower_scalar_expr(right);
                debug_assert_eq!(left_ty, right_ty);
                let dest = self.next_temp();

                if let Some(op) = crate::codegen::integer_binary_op(*op) {
                    self.instructions.push(Instruction::IntegerBinary {
                        dest,
                        op,
                        ty: crate::codegen::integer_type(&expr.ty),
                        left,
                        right,
                    });
                } else if let Some(op) = compare_op(*op) {
                    self.instructions.push(Instruction::Compare {
                        dest,
                        op,
                        operand_ty: left_ty,
                        left,
                        right,
                    });
                } else {
                    let op = if left_ty == Type::I64 {
                        match op {
                            primer_ir::BinaryOp::Add => BinaryOp::CheckedI64Add,
                            primer_ir::BinaryOp::Subtract => BinaryOp::CheckedI64Subtract,
                            primer_ir::BinaryOp::Multiply => BinaryOp::CheckedI64Multiply,
                            primer_ir::BinaryOp::Divide => BinaryOp::CheckedI64Divide,
                            _ => unreachable!("comparisons use dedicated QBE instructions"),
                        }
                    } else {
                        (*op).into()
                    };
                    self.instructions.push(Instruction::Binary {
                        dest,
                        op,
                        ty: left_ty,
                        left,
                        right,
                    });
                }

                Value::Scalar {
                    ty: scalar_type(&expr.ty),
                    operand: Operand::Temp(dest),
                }
            }
            primer_ir::ExprKind::Construct {
                type_id, fields, ..
            } => {
                let slot = self.allocate_aggregate(type_size(self.program, &expr.ty));
                for field in fields {
                    let definition = &self.program.type_definitions[type_id.0].fields[field.id.0];
                    let value = self.lower_expr(&field.value);
                    let offset = field_offset(self.program, type_id.0, field.id.0);
                    let destination = self.address(Operand::Slot(slot), offset);
                    match (&definition.ty, value) {
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
                                size: type_size(self.program, &definition.ty),
                            });
                        }
                        (
                            primer_ir::Type::Array { element, length },
                            Value::Array {
                                element: actual_element,
                                length: actual_length,
                                address,
                            },
                        ) => {
                            debug_assert_eq!(array_element_type(element), actual_element);
                            debug_assert_eq!(*length, actual_length);
                            self.instructions.push(Instruction::Blit {
                                source: address,
                                destination,
                                size: type_size(self.program, &definition.ty),
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
                match &expr.ty {
                    primer_ir::Type::Named(nested) => Value::Aggregate {
                        type_id: nested.0,
                        address,
                    },
                    primer_ir::Type::Array { element, length } => Value::Array {
                        element: array_element_type(element),
                        length: *length,
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
            primer_ir::ExprKind::Array(values) => {
                let primer_ir::Type::Array { element, length } = &expr.ty else {
                    unreachable!("array expression must have an array type")
                };
                let slot = self.allocate_aggregate(type_size(self.program, &expr.ty));
                let expected = array_element_type(element);
                let stride = type_size(self.program, element);
                for (index, value) in values.iter().enumerate() {
                    let value = self.lower_expr(value);
                    let destination = self.address(Operand::Slot(slot), index * stride);
                    match (expected.clone(), value) {
                        (ArrayElement::Scalar(expected), Value::Scalar { ty, operand }) => {
                            debug_assert_eq!(expected, ty);
                            self.instructions.push(Instruction::Store {
                                address: destination,
                                ty,
                                value: operand,
                            });
                        }
                        (
                            ArrayElement::Named(expected),
                            Value::Aggregate {
                                type_id,
                                address: source,
                            },
                        ) => {
                            debug_assert_eq!(expected, type_id);
                            self.instructions.push(Instruction::Blit {
                                source,
                                destination,
                                size: stride,
                            });
                        }
                        (
                            ArrayElement::Array {
                                element: expected_element,
                                length: expected_length,
                            },
                            Value::Array {
                                element,
                                length,
                                address: source,
                            },
                        ) => {
                            debug_assert_eq!(*expected_element, element);
                            debug_assert_eq!(expected_length, length);
                            self.instructions.push(Instruction::Blit {
                                source,
                                destination,
                                size: stride,
                            });
                        }
                        _ => unreachable!("semantic analysis keeps array element types equal"),
                    }
                }
                Value::Array {
                    element: expected,
                    length: *length,
                    address: Operand::Slot(slot),
                }
            }
            primer_ir::ExprKind::Index { base, index } => {
                let Value::Array {
                    element,
                    length,
                    address: base,
                } = self.lower_expr(base)
                else {
                    unreachable!("indexed expression must have an array base")
                };
                let address = self.lower_checked_array_address(base, &element, length, index);
                match element {
                    ArrayElement::Scalar(ty) => {
                        let dest = self.next_temp();
                        self.instructions.push(Instruction::Load {
                            dest,
                            address: address.clone(),
                            ty,
                        });
                        Value::Scalar {
                            ty,
                            operand: Operand::Temp(dest),
                        }
                    }
                    ArrayElement::Named(type_id) => Value::Aggregate {
                        type_id,
                        address: address.clone(),
                    },
                    ArrayElement::Array { element, length } => Value::Array {
                        element: *element,
                        length,
                        address,
                    },
                }
            }
            primer_ir::ExprKind::Call {
                function_id,
                arguments,
                ..
            } => self
                .lower_call(function_id.0, arguments, Some(&expr.ty))
                .expect("call expressions produce a value"),
        }
    }

    fn store_value(&mut self, ty: &primer_ir::Type, value: Value, destination: Operand) {
        match (ty, value) {
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
                    size: type_size(self.program, ty),
                });
            }
            (
                primer_ir::Type::Array { element, length },
                Value::Array {
                    element: actual_element,
                    length: actual_length,
                    address,
                },
            ) => {
                debug_assert_eq!(array_element_type(element), actual_element);
                debug_assert_eq!(*length, actual_length);
                self.instructions.push(Instruction::Blit {
                    source: address,
                    destination,
                    size: type_size(self.program, ty),
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
    }

    fn lower_checked_array_address(
        &mut self,
        base: Operand,
        element: &ArrayElement,
        length: usize,
        index: &primer_ir::Expr,
    ) -> Operand {
        let (index_ty, index) = self.lower_scalar_expr(index);
        debug_assert_eq!(index_ty, Type::I64);
        let non_negative = self.next_label();
        let in_bounds = self.next_label();
        let out_of_bounds = self.next_label();
        let is_negative = self.next_temp();
        self.instructions.push(Instruction::Compare {
            dest: is_negative,
            op: CompareOp::Less,
            operand_ty: Type::I64,
            left: index.clone(),
            right: Operand::Integer(0),
        });
        self.instructions.push(Instruction::Branch {
            condition: Operand::Temp(is_negative),
            then_label: out_of_bounds,
            else_label: non_negative,
        });
        self.instructions.push(Instruction::Label {
            id: non_negative,
            name: "array_index_non_negative",
        });
        let is_too_large = self.next_temp();
        self.instructions.push(Instruction::Compare {
            dest: is_too_large,
            op: CompareOp::GreaterEqual,
            operand_ty: Type::I64,
            left: index.clone(),
            right: Operand::Integer(length as i64),
        });
        self.instructions.push(Instruction::Branch {
            condition: Operand::Temp(is_too_large),
            then_label: out_of_bounds,
            else_label: in_bounds,
        });
        self.instructions.push(Instruction::Label {
            id: out_of_bounds,
            name: "array_index_out_of_bounds",
        });
        self.instructions.push(Instruction::Abort);
        self.instructions.push(Instruction::Label {
            id: in_bounds,
            name: "array_index_in_bounds",
        });
        let scaled = self.next_temp();
        self.instructions.push(Instruction::Binary {
            dest: scaled,
            op: BinaryOp::Multiply,
            ty: Type::I64,
            left: index,
            right: Operand::Integer(array_element_size(self.program, element) as i64),
        });
        let address = self.next_temp();
        self.instructions.push(Instruction::Binary {
            dest: address,
            op: BinaryOp::Add,
            ty: Type::I64,
            left: base,
            right: Operand::Temp(scaled),
        });
        Operand::Temp(address)
    }

    fn lower_call(
        &mut self,
        function_id: usize,
        arguments: &[primer_ir::Expr],
        result_type: Option<&primer_ir::Type>,
    ) -> Option<Value> {
        let mut lowered_arguments = Vec::new();
        let aggregate_result = result_type.and_then(|ty| match ty {
            primer_ir::Type::Named(_) | primer_ir::Type::Array { .. } => {
                let slot = self.allocate_aggregate(type_size(self.program, ty));
                lowered_arguments.push((Type::Pointer, Operand::Slot(slot)));
                Some((ty, slot))
            }
            primer_ir::Type::String
            | primer_ir::Type::Bool
            | primer_ir::Type::Integer(_)
            | primer_ir::Type::F32
            | primer_ir::Type::F64 => None,
        });

        for argument in arguments {
            match self.lower_expr(argument) {
                Value::Scalar { ty, operand } => lowered_arguments.push((ty, operand)),
                Value::Aggregate { address, .. } | Value::Array { address, .. } => {
                    lowered_arguments.push((Type::Pointer, address));
                }
            }
        }

        let (dest, return_type, scalar_result) = match result_type {
            Some(
                ty @ (primer_ir::Type::String
                | primer_ir::Type::Bool
                | primer_ir::Type::Integer(_)
                | primer_ir::Type::F32
                | primer_ir::Type::F64),
            ) => {
                let ty = scalar_type(ty);
                let dest = self.next_temp();
                (Some(dest), Some(ty), Some((ty, dest)))
            }
            Some(primer_ir::Type::Named(_) | primer_ir::Type::Array { .. }) | None => {
                (None, None, None)
            }
        };

        self.instructions.push(Instruction::Call {
            dest,
            function_id,
            return_type,
            arguments: lowered_arguments,
        });

        if let Some((ty, dest)) = scalar_result {
            return Some(Value::Scalar {
                ty,
                operand: Operand::Temp(dest),
            });
        }

        aggregate_result.map(|(ty, slot)| match ty {
            primer_ir::Type::Named(id) => Value::Aggregate {
                type_id: id.0,
                address: Operand::Slot(slot),
            },
            primer_ir::Type::Array { element, length } => Value::Array {
                element: array_element_type(element),
                length: *length,
                address: Operand::Slot(slot),
            },
            primer_ir::Type::String
            | primer_ir::Type::Bool
            | primer_ir::Type::Integer(_)
            | primer_ir::Type::F32
            | primer_ir::Type::F64 => unreachable!("aggregate result type is checked above"),
        })
    }

    fn lower_scalar_expr(&mut self, expr: &primer_ir::Expr) -> (Type, Operand) {
        let Value::Scalar { ty, operand } = self.lower_expr(expr) else {
            unreachable!("semantic analysis requires a scalar value here")
        };
        (ty, operand)
    }

    fn lower_print(&mut self, ty: Type, operand: Operand) {
        match ty {
            Type::String => self
                .instructions
                .push(Instruction::PrintString { value: operand }),
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
            Type::Pointer => unreachable!("pointers are not printable Primer values"),
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
                    size: type_size(program, ty),
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

fn type_size(program: &primer_ir::Program, ty: &primer_ir::Type) -> usize {
    match ty {
        primer_ir::Type::String
        | primer_ir::Type::Bool
        | primer_ir::Type::Integer(_)
        | primer_ir::Type::F32
        | primer_ir::Type::F64 => 8,
        primer_ir::Type::Named(id) => program.type_definitions[id.0]
            .fields
            .iter()
            .map(|field| type_size(program, &field.ty))
            .sum(),
        primer_ir::Type::Array { element, length } => type_size(program, element) * length,
    }
}

fn field_offset(program: &primer_ir::Program, type_id: usize, field_id: usize) -> usize {
    program.type_definitions[type_id].fields[..field_id]
        .iter()
        .map(|field| type_size(program, &field.ty))
        .sum()
}

fn scalar_type(ty: &primer_ir::Type) -> Type {
    match ty {
        primer_ir::Type::String => Type::String,
        primer_ir::Type::Bool => Type::Bool,
        primer_ir::Type::Integer(_) => Type::I64,
        primer_ir::Type::F32 => Type::Single,
        primer_ir::Type::F64 => Type::Double,
        primer_ir::Type::Named(_) | primer_ir::Type::Array { .. } => {
            unreachable!("expected a scalar type")
        }
    }
}

fn array_element_type(element: &primer_ir::Type) -> ArrayElement {
    match element {
        primer_ir::Type::String => ArrayElement::Scalar(Type::String),
        primer_ir::Type::Bool => ArrayElement::Scalar(Type::Bool),
        primer_ir::Type::Integer(_) => ArrayElement::Scalar(Type::I64),
        primer_ir::Type::F32 => ArrayElement::Scalar(Type::Single),
        primer_ir::Type::F64 => ArrayElement::Scalar(Type::Double),
        primer_ir::Type::Named(id) => ArrayElement::Named(id.0),
        primer_ir::Type::Array { element, length } => ArrayElement::Array {
            element: Box::new(array_element_type(element)),
            length: *length,
        },
    }
}

fn array_element_size(program: &primer_ir::Program, element: &ArrayElement) -> usize {
    match element {
        ArrayElement::Scalar(_) => 8,
        ArrayElement::Named(id) => {
            type_size(program, &primer_ir::Type::Named(primer_ir::TypeId(*id)))
        }
        ArrayElement::Array { element, length } => array_element_size(program, element) * length,
    }
}

impl From<primer_ir::BinaryOp> for BinaryOp {
    fn from(value: primer_ir::BinaryOp) -> Self {
        match value {
            primer_ir::BinaryOp::Add => Self::Add,
            primer_ir::BinaryOp::Subtract => Self::Subtract,
            primer_ir::BinaryOp::Multiply => Self::Multiply,
            primer_ir::BinaryOp::Divide => Self::Divide,
            primer_ir::BinaryOp::Remainder
            | primer_ir::BinaryOp::BitAnd
            | primer_ir::BinaryOp::BitOr
            | primer_ir::BinaryOp::BitXor
            | primer_ir::BinaryOp::ShiftLeft
            | primer_ir::BinaryOp::ShiftRight => {
                unreachable!("integer operation uses separate lowering")
            }
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
        | primer_ir::BinaryOp::Divide
        | primer_ir::BinaryOp::Remainder
        | primer_ir::BinaryOp::BitAnd
        | primer_ir::BinaryOp::BitOr
        | primer_ir::BinaryOp::BitXor
        | primer_ir::BinaryOp::ShiftLeft
        | primer_ir::BinaryOp::ShiftRight => None,
        primer_ir::BinaryOp::Equal => Some(CompareOp::Equal),
        primer_ir::BinaryOp::NotEqual => Some(CompareOp::NotEqual),
        primer_ir::BinaryOp::Less => Some(CompareOp::Less),
        primer_ir::BinaryOp::LessEqual => Some(CompareOp::LessEqual),
        primer_ir::BinaryOp::Greater => Some(CompareOp::Greater),
        primer_ir::BinaryOp::GreaterEqual => Some(CompareOp::GreaterEqual),
    }
}
