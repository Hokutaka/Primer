use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{
    Argument, BinaryOp, CompareOp, FloatConstant, Function, Instruction, Module, Type,
};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut strings = Vec::new();
    let mut float_id = 0;
    let mut float_constants = Vec::new();
    let mut functions = Vec::new();
    for function in &program.function_definitions {
        let lowered = lower_body(
            program,
            &function.parameters,
            Some(&function.return_type),
            &function.body,
            &mut float_id,
            &mut strings,
            true,
        );
        float_constants.extend(lowered.float_constants);
        functions.push(Function {
            id: function.id.0,
            name: function.name.clone(),
            frame_size: lowered.frame_size,
            instructions: lowered.instructions,
        });
    }

    let lowered = lower_body(
        program,
        &[],
        None,
        &program.statements,
        &mut float_id,
        &mut strings,
        false,
    );
    float_constants.extend(lowered.float_constants);

    Module {
        uses_strings: crate::codegen::support::first_string_span(program).is_some(),
        strings,
        functions,
        explicit_main: program
            .function_definitions
            .iter()
            .find(|function| function.name == "main")
            .map(|function| function.id.0),
        frame_size: lowered.frame_size,
        float_constants,
        instructions: lowered.instructions,
    }
}

struct LoweredBody {
    frame_size: usize,
    float_constants: Vec<FloatConstant>,
    instructions: Vec<Instruction>,
}

fn lower_body(
    program: &primer_ir::Program,
    parameters: &[primer_ir::Parameter],
    return_type: Option<&primer_ir::ReturnType>,
    statements: &[primer_ir::Statement],
    float_id: &mut usize,
    strings: &mut Vec<String>,
    is_function: bool,
) -> LoweredBody {
    let (binding_slots, binding_slot_count) = assign_binding_slots(program, parameters, statements);

    // 従来の式用領域を保ちつつ、入れ子の呼び出しに必要な引数領域だけを追加する。
    let scratch_count = count_statements_expr_nodes(statements)
        .max(required_scratch_slots(statements))
        .max(1);
    let scratch_base = binding_slot_count;
    let aggregate_base = scratch_base + scratch_count;

    let mut lowerer = Lowerer {
        strings,
        program,
        binding_slots,
        scratch_base,
        next_aggregate_slot: aggregate_base,
        float_id: *float_id,
        float_constants: Vec::new(),
        instructions: Vec::new(),
        label: 0,
        loops: Vec::new(),
        aggregate_return_pointer_slot: None,
    };

    for (index, parameter) in parameters.iter().enumerate() {
        match &parameter.ty {
            primer_ir::Type::String
            | primer_ir::Type::Bool
            | primer_ir::Type::Integer(_)
            | primer_ir::Type::F32
            | primer_ir::Type::F64 => {
                lowerer.instructions.push(Instruction::StoreParameter {
                    index,
                    ty: scalar_type(&parameter.ty),
                    offset: lowerer.binding_offset(parameter.id),
                });
            }
            primer_ir::Type::Named(_) | primer_ir::Type::Array { .. } => {
                lowerer
                    .instructions
                    .push(Instruction::StoreAggregateParameter {
                        index,
                        slots: type_slot_count(program, &parameter.ty),
                        destination_offset: lowerer.binding_offset(parameter.id),
                    });
            }
        }
    }
    if matches!(
        return_type,
        Some(primer_ir::ReturnType::Value(
            primer_ir::Type::Named(_) | primer_ir::Type::Array { .. }
        ))
    ) {
        let slot = lowerer.next_aggregate_slot;
        lowerer.next_aggregate_slot += 1;
        lowerer.aggregate_return_pointer_slot = Some(slot);
        lowerer
            .instructions
            .push(Instruction::StoreAggregateReturnPointer {
                offset: slot_offset(slot),
            });
    }
    let terminates = lowerer.lower_statements(statements);
    if is_function && !terminates {
        lowerer.instructions.push(Instruction::Return);
    }

    // Windows x64 ABI では、関数呼び出し用に 32 バイトの shadow space が必要になる。
    let local_bytes = 8 * lowerer.next_aggregate_slot;
    let frame_size = align16(32 + local_bytes);
    *float_id = lowerer.float_id;

    LoweredBody {
        frame_size,
        float_constants: lowerer.float_constants,
        instructions: lowerer.instructions,
    }
}

struct Lowerer<'a> {
    strings: &'a mut Vec<String>,
    program: &'a primer_ir::Program,
    binding_slots: HashMap<primer_ir::BindingId, usize>,
    scratch_base: usize,
    next_aggregate_slot: usize,
    float_id: usize,
    float_constants: Vec<FloatConstant>,
    instructions: Vec<Instruction>,
    label: usize,
    loops: Vec<LoopContext>,
    aggregate_return_pointer_slot: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
struct LoopContext {
    continue_label: usize,
    break_label: usize,
}

#[derive(Debug, Clone)]
enum Value {
    Scalar(Type),
    Aggregate {
        type_id: usize,
        base_slot: usize,
    },
    Array {
        element: ArrayElement,
        length: usize,
        base_slot: usize,
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

#[derive(Debug, Clone, Copy)]
enum ArrayAddress {
    Direct(isize),
    Indirect(isize),
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
                let value = self.lower_expr(value, 0);
                let destination = self.binding_slot(*id);
                self.store_value(ty, value, destination);
                false
            }

            primer_ir::StatementKind::Assignment { target, value } => {
                if target.projections.is_empty() {
                    let value = self.lower_expr(value, 0);
                    self.store_value(&target.root_ty, value, self.binding_slot(target.id));
                    return false;
                }

                let mut address = ArrayAddress::Direct(self.binding_offset(target.id));
                for projection in &target.projections {
                    let primer_ir::AssignmentProjection::Index {
                        index,
                        element,
                        length,
                        ..
                    } = projection;
                    let Value::Scalar(Type::I64) = self.lower_expr(index, 0) else {
                        unreachable!("array index must be i64")
                    };
                    let pointer_slot = self.next_aggregate_slot;
                    self.next_aggregate_slot += 1;
                    let (base_offset, base_is_pointer) = match address {
                        ArrayAddress::Direct(offset) => (offset, false),
                        ArrayAddress::Indirect(offset) => (offset, true),
                    };
                    let label = self.next_label();
                    self.instructions.push(Instruction::CheckedArrayAddress {
                        base_offset,
                        base_is_pointer,
                        length: *length,
                        element_slots: type_slot_count(self.program, element),
                        destination_offset: slot_offset(pointer_slot),
                        label,
                    });
                    address = ArrayAddress::Indirect(slot_offset(pointer_slot));
                }

                let value = self.lower_expr(value, 0);
                let ArrayAddress::Indirect(pointer_offset) = address else {
                    unreachable!("indexed assignment produces an indirect address")
                };
                self.store_value_to_pointer(&target.ty, value, pointer_offset);
                false
            }

            primer_ir::StatementKind::Print { value } => {
                let Value::Scalar(ty) = self.lower_expr(value, 0) else {
                    unreachable!("semantic analysis rejects aggregate printing")
                };
                self.lower_print(ty);
                false
            }

            primer_ir::StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                let Value::Scalar(Type::Bool) = self.lower_expr(condition, 0) else {
                    unreachable!("semantic analysis requires a bool condition")
                };
                let else_label = self.next_label();
                let end_label = self.next_label();
                self.instructions
                    .push(Instruction::JumpIfZero(if else_body.is_empty() {
                        end_label
                    } else {
                        else_label
                    }));

                let then_terminates = self.lower_statements(then_body);
                if !then_terminates {
                    self.instructions.push(Instruction::Jump(end_label));
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
                let end_label = self.next_label();

                self.instructions.push(Instruction::Label {
                    id: condition_label,
                    name: "while_condition",
                });
                let Value::Scalar(Type::Bool) = self.lower_expr(condition, 0) else {
                    unreachable!("semantic analysis requires a bool condition")
                };
                self.instructions.push(Instruction::JumpIfZero(end_label));

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
                let update_label = self.next_label();
                let end_label = self.next_label();

                self.instructions.push(Instruction::Label {
                    id: condition_label,
                    name: "for_condition",
                });
                let Value::Scalar(Type::Bool) = self.lower_expr(condition, 0) else {
                    unreachable!("semantic analysis requires a bool condition")
                };
                self.instructions.push(Instruction::JumpIfZero(end_label));

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
                self.lower_call(function_id.0, arguments, None, 0);
                false
            }
            primer_ir::StatementKind::Return { value } => {
                if let Some(value) = value {
                    match self.lower_expr(value, 0) {
                        Value::Scalar(_) => {}
                        Value::Aggregate { base_slot, .. } | Value::Array { base_slot, .. } => {
                            let pointer_slot = self
                                .aggregate_return_pointer_slot
                                .expect("aggregate returns have a result pointer");
                            self.instructions.push(Instruction::CopyToAggregateReturn {
                                source_offset: slot_offset(base_slot),
                                slots: type_slot_count(self.program, &value.ty),
                                pointer_offset: slot_offset(pointer_slot),
                            });
                        }
                    }
                }
                self.instructions.push(Instruction::Return);
                true
            }
        }
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr, depth: usize) -> Value {
        let value = self.lower_expr_unchecked(expr, depth);
        if let Some(ty) = super::super::integer_range_check(expr) {
            let label = self.next_label();
            self.instructions
                .push(Instruction::CheckIntegerRange { ty, label });
        }
        value
    }

    fn lower_expr_unchecked(&mut self, expr: &primer_ir::Expr, depth: usize) -> Value {
        match &expr.kind {
            primer_ir::ExprKind::StringByteLength { value } => {
                self.lower_expr(value, depth);
                self.instructions.push(Instruction::LoadStringLength);
                Value::Scalar(Type::I64)
            }
            primer_ir::ExprKind::String(value) => {
                let id = self.strings.len();
                self.strings.push(value.clone());
                self.instructions.push(Instruction::LoadStringConstant(id));
                Value::Scalar(Type::String)
            }
            primer_ir::ExprKind::ConvertNumeric {
                value, from, to, ..
            } => {
                self.lower_expr(value, depth);
                if from != to {
                    let label = self.next_label();
                    self.instructions.push(Instruction::ConvertNumeric {
                        conversion: crate::codegen::NumericConversion {
                            from: *from,
                            to: *to,
                        },
                        label,
                    });
                }
                Value::Scalar(scalar_type(&expr.ty))
            }
            primer_ir::ExprKind::ConvertInteger { value, .. } => self.lower_expr(value, depth),
            primer_ir::ExprKind::Boolean(value) => {
                self.instructions
                    .push(Instruction::MovI64ImmediateToRax(i64::from(*value)));
                Value::Scalar(Type::Bool)
            }
            primer_ir::ExprKind::Integer(value) => {
                self.instructions
                    .push(Instruction::MovI64ImmediateToRax(*value));
                Value::Scalar(Type::I64)
            }
            primer_ir::ExprKind::Float { text } => {
                let ty = scalar_type(&expr.ty);
                let id = self.add_float_constant(text, ty);
                match ty {
                    Type::F32 => self.instructions.push(Instruction::LoadF32Constant(id)),
                    Type::F64 => self.instructions.push(Instruction::LoadF64Constant(id)),
                    Type::String | Type::Bool | Type::I64 => {
                        unreachable!("a float literal has a float type")
                    }
                }
                Value::Scalar(ty)
            }
            primer_ir::ExprKind::Variable { id, .. } => match &expr.ty {
                primer_ir::Type::Named(type_id) => Value::Aggregate {
                    type_id: type_id.0,
                    base_slot: self.binding_slot(*id),
                },
                primer_ir::Type::Array { element, length } => Value::Array {
                    element: array_element_type(element),
                    length: *length,
                    base_slot: self.binding_slot(*id),
                },
                scalar => {
                    let ty = scalar_type(scalar);
                    self.load_scalar(ty, self.binding_offset(*id));
                    Value::Scalar(ty)
                }
            },
            primer_ir::ExprKind::Unary { op, value } => {
                let Value::Scalar(ty) = self.lower_expr(value, depth) else {
                    unreachable!("semantic analysis rejects aggregate unary operands")
                };
                match (*op, ty) {
                    (primer_ir::UnaryOp::BitNot, Type::I64) => {
                        self.instructions.push(Instruction::BitNot {
                            mask: crate::codegen::complement_mask(&expr.ty),
                        })
                    }
                    (primer_ir::UnaryOp::Negate, Type::I64) => {
                        self.instructions.push(Instruction::NegI64);
                        let label = self.next_label();
                        self.instructions.push(Instruction::TrapIfOverflow(label));
                    }
                    (primer_ir::UnaryOp::Negate, Type::F32) => {
                        self.instructions.push(Instruction::NegF32)
                    }
                    (primer_ir::UnaryOp::Negate, Type::F64) => {
                        self.instructions.push(Instruction::NegF64)
                    }
                    (primer_ir::UnaryOp::Not, Type::Bool) => {
                        self.instructions.push(Instruction::NotBool)
                    }
                    _ => unreachable!("semantic analysis rejects invalid unary operands"),
                }
                Value::Scalar(ty)
            }
            primer_ir::ExprKind::Logical { op, left, right } => {
                self.lower_expr(left, depth);
                let false_label = self.next_label();
                let end_label = self.next_label();
                self.instructions.push(Instruction::JumpIfZero(false_label));
                if *op == primer_ir::LogicalOp::And {
                    self.lower_expr(right, depth);
                }
                self.instructions.push(Instruction::Jump(end_label));
                self.instructions.push(Instruction::Label {
                    id: false_label,
                    name: "logical_false",
                });
                if *op == primer_ir::LogicalOp::Or {
                    self.lower_expr(right, depth);
                }
                self.instructions.push(Instruction::Label {
                    id: end_label,
                    name: "logical_end",
                });
                Value::Scalar(Type::Bool)
            }
            primer_ir::ExprKind::Binary { op, left, right } => {
                let Value::Scalar(operand_ty) = self.lower_expr(left, depth + 1) else {
                    unreachable!("semantic analysis rejects aggregate binary operands")
                };

                // 左辺の計算結果を、右辺を計算している間だけ退避する。
                let scratch = self.scratch_offset(depth);
                self.store_scalar(operand_ty, scratch);

                let Value::Scalar(right_ty) = self.lower_expr(right, depth + 1) else {
                    unreachable!("semantic analysis rejects aggregate binary operands")
                };
                debug_assert_eq!(operand_ty, right_ty);

                match operand_ty {
                    Type::String => {
                        self.instructions.push(Instruction::CompareString {
                            left_offset: scratch,
                            equal: *op == primer_ir::BinaryOp::Equal,
                        });
                    }
                    Type::Bool | Type::I64 => {
                        self.instructions.push(Instruction::MoveRaxToRcx);
                        self.instructions
                            .push(Instruction::LoadI64ScratchToRax(scratch));

                        if let Some(op) = crate::codegen::integer_binary_op(*op) {
                            let label = self.next_label();
                            self.instructions.push(Instruction::IntegerBinary {
                                op,
                                ty: crate::codegen::integer_type(&expr.ty),
                                label,
                            });
                        } else if let Some(op) = compare_op(*op) {
                            self.instructions.push(Instruction::CompareI64(op));
                        } else {
                            let op = (*op).into();
                            if op == BinaryOp::Divide {
                                let label = self.next_label();
                                self.instructions
                                    .push(Instruction::TrapIfInvalidI64Division(label));
                                self.instructions.push(Instruction::SignExtendRax);
                                self.instructions.push(Instruction::DivideRaxByRcx);
                            } else {
                                self.instructions.push(Instruction::I64Binary(op));
                                let label = self.next_label();
                                self.instructions.push(Instruction::TrapIfOverflow(label));
                            }
                        }
                    }
                    Type::F32 => {
                        self.instructions.push(Instruction::CopyXmm0ToXmm1F32);
                        self.instructions
                            .push(Instruction::LoadF32ScratchToXmm0(scratch));
                        if let Some(op) = compare_op(*op) {
                            self.instructions.push(Instruction::CompareF32(op));
                        } else {
                            self.instructions.push(Instruction::F32Binary((*op).into()));
                        }
                    }
                    Type::F64 => {
                        self.instructions.push(Instruction::CopyXmm0ToXmm1F64);
                        self.instructions
                            .push(Instruction::LoadF64ScratchToXmm0(scratch));
                        if let Some(op) = compare_op(*op) {
                            self.instructions.push(Instruction::CompareF64(op));
                        } else {
                            self.instructions.push(Instruction::F64Binary((*op).into()));
                        }
                    }
                }

                Value::Scalar(scalar_type(&expr.ty))
            }
            primer_ir::ExprKind::Construct {
                type_id, fields, ..
            } => {
                let destination = self.allocate_aggregate(&expr.ty);
                for field in fields {
                    let definition = &self.program.type_definitions[type_id.0].fields[field.id.0];
                    let value = self.lower_expr(&field.value, depth);
                    let field_slot =
                        destination + field_slot_offset(self.program, type_id.0, field.id.0);
                    match (&definition.ty, value) {
                        (
                            primer_ir::Type::Named(nested),
                            Value::Aggregate {
                                type_id: actual,
                                base_slot: source,
                            },
                        ) => {
                            debug_assert_eq!(nested.0, actual);
                            self.copy_aggregate(nested.0, source, field_slot);
                        }
                        (
                            primer_ir::Type::Array { element, length },
                            Value::Array {
                                element: actual_element,
                                length: actual_length,
                                base_slot: source,
                            },
                        ) => {
                            let element = array_element_type(element);
                            debug_assert_eq!(element, actual_element);
                            debug_assert_eq!(*length, actual_length);
                            self.copy_array(&element, *length, source, field_slot);
                        }
                        (scalar, Value::Scalar(actual)) => {
                            debug_assert_eq!(scalar_type(scalar), actual);
                            self.store_scalar(actual, slot_offset(field_slot));
                        }
                        _ => unreachable!("semantic analysis keeps field types equal"),
                    }
                }
                Value::Aggregate {
                    type_id: type_id.0,
                    base_slot: destination,
                }
            }
            primer_ir::ExprKind::FieldAccess {
                type_id,
                field_id,
                base,
                ..
            } => {
                let Value::Aggregate { base_slot, .. } = self.lower_expr(base, depth) else {
                    unreachable!("semantic analysis requires an aggregate field base")
                };
                let field_slot = base_slot + field_slot_offset(self.program, type_id.0, field_id.0);
                match &expr.ty {
                    primer_ir::Type::Named(nested) => Value::Aggregate {
                        type_id: nested.0,
                        base_slot: field_slot,
                    },
                    primer_ir::Type::Array { element, length } => Value::Array {
                        element: array_element_type(element),
                        length: *length,
                        base_slot: field_slot,
                    },
                    scalar => {
                        let ty = scalar_type(scalar);
                        self.load_scalar(ty, slot_offset(field_slot));
                        Value::Scalar(ty)
                    }
                }
            }
            primer_ir::ExprKind::Array(values) => {
                let primer_ir::Type::Array { element, length } = &expr.ty else {
                    unreachable!("array expression must have an array type")
                };
                let destination = self.allocate_aggregate(&expr.ty);
                let element = array_element_type(element);
                let stride = array_element_slot_count(self.program, &element);
                for (index, value) in values.iter().enumerate() {
                    let destination = destination + index * stride;
                    match (element.clone(), self.lower_expr(value, depth)) {
                        (ArrayElement::Scalar(expected), Value::Scalar(actual)) => {
                            debug_assert_eq!(expected, actual);
                            self.store_scalar(actual, slot_offset(destination));
                        }
                        (
                            ArrayElement::Named(expected),
                            Value::Aggregate {
                                type_id,
                                base_slot: source,
                            },
                        ) => {
                            debug_assert_eq!(expected, type_id);
                            self.copy_aggregate(type_id, source, destination);
                        }
                        (
                            ArrayElement::Array {
                                element: expected_element,
                                length: expected_length,
                            },
                            Value::Array {
                                element,
                                length,
                                base_slot: source,
                            },
                        ) => {
                            debug_assert_eq!(*expected_element, element);
                            debug_assert_eq!(expected_length, length);
                            self.copy_array(
                                &expected_element,
                                expected_length,
                                source,
                                destination,
                            );
                        }
                        _ => unreachable!("semantic analysis keeps array element types equal"),
                    }
                }
                Value::Array {
                    element,
                    length: *length,
                    base_slot: destination,
                }
            }
            primer_ir::ExprKind::Index { base, index } => {
                let Value::Array {
                    element,
                    length,
                    base_slot,
                } = self.lower_expr(base, depth)
                else {
                    unreachable!("indexed expression must have an array base")
                };
                let Value::Scalar(Type::I64) = self.lower_expr(index, depth) else {
                    unreachable!("array index must be i64")
                };
                let label = self.next_label();
                match element {
                    ArrayElement::Scalar(ty) => {
                        self.instructions.push(Instruction::CheckedArrayLoad {
                            ty,
                            base_offset: slot_offset(base_slot),
                            length,
                            label,
                        });
                        Value::Scalar(ty)
                    }
                    ArrayElement::Named(type_id) => {
                        let element_slots =
                            array_element_slot_count(self.program, &ArrayElement::Named(type_id));
                        let destination = self.allocate_aggregate(&primer_ir::Type::Named(
                            primer_ir::TypeId(type_id),
                        ));
                        self.instructions.push(Instruction::CheckedArrayCopy {
                            base_offset: slot_offset(base_slot),
                            length,
                            element_slots,
                            destination_offset: slot_offset(destination),
                            label,
                        });
                        Value::Aggregate {
                            type_id,
                            base_slot: destination,
                        }
                    }
                    ArrayElement::Array {
                        element: nested_element,
                        length: nested_length,
                    } => {
                        let element = ArrayElement::Array {
                            element: nested_element.clone(),
                            length: nested_length,
                        };
                        let element_slots = array_element_slot_count(self.program, &element);
                        let destination = self.next_aggregate_slot;
                        self.next_aggregate_slot += element_slots;
                        self.instructions.push(Instruction::CheckedArrayCopy {
                            base_offset: slot_offset(base_slot),
                            length,
                            element_slots,
                            destination_offset: slot_offset(destination),
                            label,
                        });
                        Value::Array {
                            element: *nested_element,
                            length: nested_length,
                            base_slot: destination,
                        }
                    }
                }
            }
            primer_ir::ExprKind::Call {
                function_id,
                arguments,
                ..
            } => self
                .lower_call(function_id.0, arguments, Some(&expr.ty), depth)
                .expect("value-producing calls have a result"),
        }
    }

    fn lower_call(
        &mut self,
        function_id: usize,
        arguments: &[primer_ir::Expr],
        result_type: Option<&primer_ir::Type>,
        depth: usize,
    ) -> Option<Value> {
        let mut lowered_arguments = Vec::with_capacity(arguments.len());
        for (index, argument) in arguments.iter().enumerate() {
            match self.lower_expr(argument, depth + 4) {
                Value::Scalar(ty) => {
                    let offset = self.scratch_offset(depth + index);
                    self.store_scalar(ty, offset);
                    lowered_arguments.push(Argument::Scalar { ty, offset });
                }
                Value::Aggregate { base_slot, .. } | Value::Array { base_slot, .. } => {
                    lowered_arguments.push(Argument::Aggregate {
                        offset: slot_offset(base_slot),
                    });
                }
            }
        }

        let aggregate_result = result_type.and_then(|ty| match ty {
            primer_ir::Type::Named(_) | primer_ir::Type::Array { .. } => {
                Some((ty, self.allocate_aggregate(ty)))
            }
            primer_ir::Type::String
            | primer_ir::Type::Bool
            | primer_ir::Type::Integer(_)
            | primer_ir::Type::F32
            | primer_ir::Type::F64 => None,
        });
        self.instructions.push(Instruction::Call {
            function_id,
            arguments: lowered_arguments,
            aggregate_result_offset: aggregate_result
                .as_ref()
                .map(|(_, slot)| slot_offset(*slot)),
        });

        if let Some((ty, base_slot)) = aggregate_result {
            return Some(match ty {
                primer_ir::Type::Named(type_id) => Value::Aggregate {
                    type_id: type_id.0,
                    base_slot,
                },
                primer_ir::Type::Array { element, length } => Value::Array {
                    element: array_element_type(element),
                    length: *length,
                    base_slot,
                },
                primer_ir::Type::String
                | primer_ir::Type::Bool
                | primer_ir::Type::Integer(_)
                | primer_ir::Type::F32
                | primer_ir::Type::F64 => unreachable!("aggregate result type is checked above"),
            });
        }

        result_type.map(|ty| Value::Scalar(scalar_type(ty)))
    }

    fn copy_aggregate(&mut self, type_id: usize, source: usize, destination: usize) {
        for (field_id, field) in self.program.type_definitions[type_id]
            .fields
            .iter()
            .enumerate()
        {
            let offset = field_slot_offset(self.program, type_id, field_id);
            match &field.ty {
                primer_ir::Type::Named(nested) => {
                    self.copy_aggregate(nested.0, source + offset, destination + offset)
                }
                primer_ir::Type::Array { element, length } => self.copy_array(
                    &array_element_type(element),
                    *length,
                    source + offset,
                    destination + offset,
                ),
                scalar => {
                    let ty = scalar_type(scalar);
                    self.load_scalar(ty, slot_offset(source + offset));
                    self.store_scalar(ty, slot_offset(destination + offset));
                }
            }
        }
    }

    fn store_value(&mut self, ty: &primer_ir::Type, value: Value, destination: usize) {
        match (ty, value) {
            (
                primer_ir::Type::Named(type_id),
                Value::Aggregate {
                    type_id: actual,
                    base_slot: source,
                },
            ) => {
                debug_assert_eq!(type_id.0, actual);
                self.copy_aggregate(type_id.0, source, destination);
            }
            (
                primer_ir::Type::Array { element, length },
                Value::Array {
                    element: actual_element,
                    length: actual_length,
                    base_slot: source,
                },
            ) => {
                debug_assert_eq!(array_element_type(element), actual_element);
                debug_assert_eq!(*length, actual_length);
                self.copy_array(&actual_element, *length, source, destination);
            }
            (scalar, Value::Scalar(actual)) => {
                debug_assert_eq!(scalar_type(scalar), actual);
                self.store_scalar(actual, slot_offset(destination));
            }
            _ => unreachable!("semantic analysis keeps assignment types equal"),
        }
    }

    fn store_value_to_pointer(
        &mut self,
        ty: &primer_ir::Type,
        value: Value,
        pointer_offset: isize,
    ) {
        match (ty, value) {
            (
                primer_ir::Type::String | primer_ir::Type::Bool | primer_ir::Type::Integer(_),
                Value::Scalar(actual),
            ) => {
                debug_assert!(matches!(actual, Type::String | Type::Bool | Type::I64));
                self.instructions
                    .push(Instruction::StoreI64ToPointer(pointer_offset));
            }
            (primer_ir::Type::F32, Value::Scalar(Type::F32)) => self
                .instructions
                .push(Instruction::StoreF32ToPointer(pointer_offset)),
            (primer_ir::Type::F64, Value::Scalar(Type::F64)) => self
                .instructions
                .push(Instruction::StoreF64ToPointer(pointer_offset)),
            (
                primer_ir::Type::Named(_),
                Value::Aggregate {
                    base_slot: source, ..
                },
            )
            | (
                primer_ir::Type::Array { .. },
                Value::Array {
                    base_slot: source, ..
                },
            ) => self.instructions.push(Instruction::CopyToPointer {
                source_offset: slot_offset(source),
                slots: type_slot_count(self.program, ty),
                pointer_offset,
            }),
            _ => unreachable!("semantic analysis keeps assignment types equal"),
        }
    }

    fn copy_array(
        &mut self,
        element: &ArrayElement,
        length: usize,
        source: usize,
        destination: usize,
    ) {
        let stride = array_element_slot_count(self.program, element);
        for index in 0..length {
            let source = source + index * stride;
            let destination = destination + index * stride;
            match element {
                ArrayElement::Scalar(ty) => {
                    self.load_scalar(*ty, slot_offset(source));
                    self.store_scalar(*ty, slot_offset(destination));
                }
                ArrayElement::Named(type_id) => self.copy_aggregate(*type_id, source, destination),
                ArrayElement::Array { element, length } => {
                    self.copy_array(element, *length, source, destination)
                }
            }
        }
    }

    fn load_scalar(&mut self, ty: Type, offset: isize) {
        self.instructions.push(match ty {
            Type::String | Type::Bool | Type::I64 => Instruction::LoadI64FromStack(offset),
            Type::F32 => Instruction::LoadF32FromStack(offset),
            Type::F64 => Instruction::LoadF64FromStack(offset),
        });
    }

    fn store_scalar(&mut self, ty: Type, offset: isize) {
        self.instructions.push(match ty {
            Type::String | Type::Bool | Type::I64 => Instruction::StoreI64ToStack(offset),
            Type::F32 => Instruction::StoreF32ToStack(offset),
            Type::F64 => Instruction::StoreF64ToStack(offset),
        });
    }

    fn lower_print(&mut self, ty: Type) {
        match ty {
            Type::String => self.instructions.push(Instruction::PrintString),
            Type::Bool => self.instructions.push(Instruction::CallPrintBool),
            Type::I64 => {
                self.instructions.push(Instruction::MoveRaxToRdx);
                self.instructions.push(Instruction::LoadFormatI64ToRcx);
                self.instructions.push(Instruction::CallPrintf);
            }
            Type::F32 => {
                // C の可変長引数では float を double に拡張する。
                self.instructions.push(Instruction::ConvertF32ToF64Argument);
                // Windows x64 の可変長引数では、浮動小数点数を汎用レジスタにも複製する。
                self.instructions.push(Instruction::MoveXmm1ToRdx);
                self.instructions.push(Instruction::LoadFormatF32ToRcx);
                self.instructions.push(Instruction::CallPrintf);
            }
            Type::F64 => {
                self.instructions.push(Instruction::CopyXmm0ToXmm1F64Scalar);
                self.instructions.push(Instruction::MoveXmm1ToRdx);
                self.instructions.push(Instruction::LoadFormatF64ToRcx);
                self.instructions.push(Instruction::CallPrintf);
            }
        }
    }

    fn add_float_constant(&mut self, text: &str, ty: Type) -> usize {
        let id = self.float_id;
        self.float_id += 1;

        match ty {
            Type::F32 => {
                let value = text
                    .parse::<f32>()
                    .expect("validated floating-point literal");
                self.float_constants.push(FloatConstant::F32 {
                    id,
                    bits: value.to_bits(),
                });
            }
            Type::F64 => {
                let value = text
                    .parse::<f64>()
                    .expect("validated floating-point literal");
                self.float_constants.push(FloatConstant::F64 {
                    id,
                    bits: value.to_bits(),
                });
            }
            Type::String | Type::Bool | Type::I64 => {
                unreachable!("a float literal has a float type")
            }
        }

        id
    }

    fn allocate_aggregate(&mut self, ty: &primer_ir::Type) -> usize {
        let base = self.next_aggregate_slot;
        self.next_aggregate_slot += type_slot_count(self.program, ty);
        base
    }

    fn binding_slot(&self, id: primer_ir::BindingId) -> usize {
        self.binding_slots[&id]
    }

    fn binding_offset(&self, id: primer_ir::BindingId) -> isize {
        slot_offset(self.binding_slot(id))
    }

    fn scratch_offset(&self, depth: usize) -> isize {
        slot_offset(self.scratch_base + depth)
    }

    fn next_label(&mut self) -> usize {
        let label = self.label;
        self.label += 1;
        label
    }
}

fn assign_binding_slots(
    program: &primer_ir::Program,
    parameters: &[primer_ir::Parameter],
    statements: &[primer_ir::Statement],
) -> (HashMap<primer_ir::BindingId, usize>, usize) {
    let mut slots = HashMap::new();
    let mut next = 0;
    for parameter in parameters {
        slots.insert(parameter.id, next);
        next += type_slot_count(program, &parameter.ty);
    }
    collect_binding_slots(statements, program, &mut slots, &mut next);
    (slots, next)
}

fn collect_binding_slots(
    statements: &[primer_ir::Statement],
    program: &primer_ir::Program,
    slots: &mut HashMap<primer_ir::BindingId, usize>,
    next: &mut usize,
) {
    for statement in statements {
        match &statement.kind {
            primer_ir::StatementKind::Binding { id, ty, .. } => {
                slots.insert(*id, *next);
                *next += type_slot_count(program, ty);
            }
            primer_ir::StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_binding_slots(then_body, program, slots, next);
                collect_binding_slots(else_body, program, slots, next);
            }
            primer_ir::StatementKind::While { body, .. } => {
                collect_binding_slots(body, program, slots, next)
            }
            primer_ir::StatementKind::For {
                initializer,
                update,
                body,
                ..
            } => {
                collect_binding_slots(std::slice::from_ref(initializer), program, slots, next);
                collect_binding_slots(std::slice::from_ref(update), program, slots, next);
                collect_binding_slots(body, program, slots, next);
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

fn type_slot_count(program: &primer_ir::Program, ty: &primer_ir::Type) -> usize {
    match ty {
        primer_ir::Type::String
        | primer_ir::Type::Bool
        | primer_ir::Type::Integer(_)
        | primer_ir::Type::F32
        | primer_ir::Type::F64 => 1,
        primer_ir::Type::Named(id) => program.type_definitions[id.0]
            .fields
            .iter()
            .map(|field| type_slot_count(program, &field.ty))
            .sum(),
        primer_ir::Type::Array { element, length } => type_slot_count(program, element) * length,
    }
}

fn field_slot_offset(program: &primer_ir::Program, type_id: usize, field_id: usize) -> usize {
    program.type_definitions[type_id].fields[..field_id]
        .iter()
        .map(|field| type_slot_count(program, &field.ty))
        .sum()
}

fn count_statements_expr_nodes(statements: &[primer_ir::Statement]) -> usize {
    statements
        .iter()
        .map(|statement| match &statement.kind {
            primer_ir::StatementKind::Binding { value, .. }
            | primer_ir::StatementKind::Assignment { value, .. }
            | primer_ir::StatementKind::Print { value } => count_expr_nodes(value),
            primer_ir::StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                count_expr_nodes(condition)
                    + count_statements_expr_nodes(then_body)
                    + count_statements_expr_nodes(else_body)
            }
            primer_ir::StatementKind::While { condition, body } => {
                count_expr_nodes(condition) + count_statements_expr_nodes(body)
            }
            primer_ir::StatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                count_statements_expr_nodes(std::slice::from_ref(initializer))
                    + count_expr_nodes(condition)
                    + count_statements_expr_nodes(std::slice::from_ref(update))
                    + count_statements_expr_nodes(body)
            }
            primer_ir::StatementKind::Call { arguments, .. } => {
                arguments.iter().map(count_expr_nodes).sum()
            }
            primer_ir::StatementKind::Return { value } => {
                value.as_ref().map_or(0, count_expr_nodes)
            }
            primer_ir::StatementKind::Break | primer_ir::StatementKind::Continue => 0,
        })
        .sum()
}

fn count_expr_nodes(expr: &primer_ir::Expr) -> usize {
    match &expr.kind {
        primer_ir::ExprKind::String(_)
        | primer_ir::ExprKind::Boolean(_)
        | primer_ir::ExprKind::Integer(_)
        | primer_ir::ExprKind::Float { .. }
        | primer_ir::ExprKind::Variable { .. } => 1,
        primer_ir::ExprKind::ConvertNumeric { value, .. }
        | primer_ir::ExprKind::ConvertInteger { value, .. } => count_expr_nodes(value),
        primer_ir::ExprKind::StringByteLength { value }
        | primer_ir::ExprKind::Unary { value, .. } => 1 + count_expr_nodes(value),
        primer_ir::ExprKind::Binary { left, right, .. }
        | primer_ir::ExprKind::Logical { left, right, .. } => {
            1 + count_expr_nodes(left) + count_expr_nodes(right)
        }
        primer_ir::ExprKind::Construct { fields, .. } => {
            1 + fields
                .iter()
                .map(|field| count_expr_nodes(&field.value))
                .sum::<usize>()
        }
        primer_ir::ExprKind::FieldAccess { base, .. } => 1 + count_expr_nodes(base),
        primer_ir::ExprKind::Array(values) => {
            1 + values.iter().map(count_expr_nodes).sum::<usize>()
        }
        primer_ir::ExprKind::Index { base, index } => {
            1 + count_expr_nodes(base) + count_expr_nodes(index)
        }
        primer_ir::ExprKind::Call { arguments, .. } => {
            1 + arguments.iter().map(count_expr_nodes).sum::<usize>()
        }
    }
}

fn required_scratch_slots(statements: &[primer_ir::Statement]) -> usize {
    statements
        .iter()
        .map(|statement| match &statement.kind {
            primer_ir::StatementKind::Binding { value, .. }
            | primer_ir::StatementKind::Assignment { value, .. }
            | primer_ir::StatementKind::Print { value } => required_expr_scratch(value, 0),
            primer_ir::StatementKind::Call { arguments, .. } => arguments
                .iter()
                .map(|argument| required_expr_scratch(argument, 4))
                .max()
                .unwrap_or(0)
                .max(arguments.len()),
            primer_ir::StatementKind::Return { value } => value
                .as_ref()
                .map_or(0, |value| required_expr_scratch(value, 0)),
            primer_ir::StatementKind::If {
                condition,
                then_body,
                else_body,
            } => required_expr_scratch(condition, 0)
                .max(required_scratch_slots(then_body))
                .max(required_scratch_slots(else_body)),
            primer_ir::StatementKind::While { condition, body } => {
                required_expr_scratch(condition, 0).max(required_scratch_slots(body))
            }
            primer_ir::StatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => required_scratch_slots(std::slice::from_ref(initializer))
                .max(required_expr_scratch(condition, 0))
                .max(required_scratch_slots(std::slice::from_ref(update)))
                .max(required_scratch_slots(body)),
            primer_ir::StatementKind::Break | primer_ir::StatementKind::Continue => 0,
        })
        .max()
        .unwrap_or(0)
}

fn required_expr_scratch(expr: &primer_ir::Expr, depth: usize) -> usize {
    match &expr.kind {
        primer_ir::ExprKind::String(_)
        | primer_ir::ExprKind::Boolean(_)
        | primer_ir::ExprKind::Integer(_)
        | primer_ir::ExprKind::Float { .. }
        | primer_ir::ExprKind::Variable { .. } => 0,
        primer_ir::ExprKind::StringByteLength { value }
        | primer_ir::ExprKind::Unary { value, .. }
        | primer_ir::ExprKind::ConvertNumeric { value, .. }
        | primer_ir::ExprKind::ConvertInteger { value, .. }
        | primer_ir::ExprKind::FieldAccess { base: value, .. } => {
            required_expr_scratch(value, depth)
        }
        primer_ir::ExprKind::Array(values) => values
            .iter()
            .map(|value| required_expr_scratch(value, depth))
            .max()
            .unwrap_or(0),
        primer_ir::ExprKind::Index { base, index } => {
            required_expr_scratch(base, depth).max(required_expr_scratch(index, depth))
        }
        primer_ir::ExprKind::Logical { left, right, .. } => {
            required_expr_scratch(left, depth).max(required_expr_scratch(right, depth))
        }
        primer_ir::ExprKind::Binary { left, right, .. } => (depth + 1)
            .max(required_expr_scratch(left, depth + 1))
            .max(required_expr_scratch(right, depth + 1)),
        primer_ir::ExprKind::Construct { fields, .. } => fields
            .iter()
            .map(|field| required_expr_scratch(&field.value, depth))
            .max()
            .unwrap_or(0),
        primer_ir::ExprKind::Call { arguments, .. } => arguments
            .iter()
            .map(|argument| required_expr_scratch(argument, depth + 4))
            .max()
            .unwrap_or(0)
            .max(depth + arguments.len()),
    }
}

fn scalar_type(ty: &primer_ir::Type) -> Type {
    match ty {
        primer_ir::Type::String => Type::String,
        primer_ir::Type::Bool => Type::Bool,
        primer_ir::Type::Integer(_) => Type::I64,
        primer_ir::Type::F32 => Type::F32,
        primer_ir::Type::F64 => Type::F64,
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
        primer_ir::Type::F32 => ArrayElement::Scalar(Type::F32),
        primer_ir::Type::F64 => ArrayElement::Scalar(Type::F64),
        primer_ir::Type::Named(id) => ArrayElement::Named(id.0),
        primer_ir::Type::Array { element, length } => ArrayElement::Array {
            element: Box::new(array_element_type(element)),
            length: *length,
        },
    }
}

fn array_element_slot_count(program: &primer_ir::Program, element: &ArrayElement) -> usize {
    match element {
        ArrayElement::Scalar(_) => 1,
        ArrayElement::Named(id) => {
            type_slot_count(program, &primer_ir::Type::Named(primer_ir::TypeId(*id)))
        }
        ArrayElement::Array { element, length } => {
            array_element_slot_count(program, element) * length
        }
    }
}

fn slot_offset(slot: usize) -> isize {
    -8 * (slot as isize + 1)
}

fn align16(value: usize) -> usize {
    (value + 15) & !15
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
                unreachable!("comparisons use dedicated x86-64 instructions")
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
