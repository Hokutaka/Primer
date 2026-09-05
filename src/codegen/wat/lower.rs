use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{Function, Instruction, Local, LoopKind, Module, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut next_address = 0;
    let mut functions = Vec::new();
    for function in &program.function_definitions {
        functions.push(lower_function(program, function, &mut next_address));
    }

    let mut locals = Vec::new();
    let mut locations = HashMap::new();
    let mut name_counts = HashMap::new();
    collect_locations(
        &program.statements,
        program,
        &mut locals,
        &mut locations,
        &mut name_counts,
        &mut next_address,
    );

    let mut context = LoweringContext {
        program,
        locations,
        next_address,
        aggregate_return_address: None,
        control: ControlContext {
            next_loop_id: 0,
            loops: Vec::new(),
        },
    };
    let mut instructions = Vec::new();
    context.lower_statements(&program.statements, &mut instructions);

    Module {
        memory_pages: if context.next_address == 0 {
            0
        } else {
            context.next_address.div_ceil(65_536) as u32
        },
        functions,
        explicit_main: program
            .function_definitions
            .iter()
            .find(|function| function.name == "main")
            .map(|function| function.id.0),
        locals,
        instructions,
    }
}

fn lower_function(
    program: &primer_ir::Program,
    function: &primer_ir::FunctionDefinition,
    next_address: &mut usize,
) -> Function {
    let mut locals = Vec::new();
    let mut locations = HashMap::new();
    let mut name_counts = HashMap::new();
    let mut parameters = Vec::new();
    let aggregate_return_type = match &function.return_type {
        primer_ir::ReturnType::Value(
            ty @ (primer_ir::Type::Named(_) | primer_ir::Type::Array { .. }),
        ) => Some(ty),
        primer_ir::ReturnType::Void
        | primer_ir::ReturnType::Value(
            primer_ir::Type::Bool
            | primer_ir::Type::Integer(_)
            | primer_ir::Type::F32
            | primer_ir::Type::F64,
        ) => None,
    };
    let aggregate_return_pointer = aggregate_return_type.map(|_| {
        parameters.push(Local {
            name: "abi.result".into(),
            ty: Type::Pointer,
        });
        let address = *next_address;
        *next_address += 4;
        address
    });
    let mut aggregate_parameters = Vec::new();
    for parameter in &function.parameters {
        name_counts.insert(parameter.name.clone(), 1);
        match &parameter.ty {
            primer_ir::Type::Bool
            | primer_ir::Type::Integer(_)
            | primer_ir::Type::F32
            | primer_ir::Type::F64 => {
                locations.insert(parameter.id, Location::Scalar(parameter.name.clone()));
                parameters.push(Local {
                    name: parameter.name.clone(),
                    ty: scalar_type(&parameter.ty),
                });
            }
            primer_ir::Type::Named(type_id) => {
                parameters.push(Local {
                    name: parameter.name.clone(),
                    ty: Type::Pointer,
                });
                let source_pointer = *next_address;
                *next_address += 4;
                let destination = *next_address;
                *next_address += type_size(program, &parameter.ty);
                locations.insert(
                    parameter.id,
                    Location::Aggregate {
                        type_id: type_id.0,
                        address: destination,
                    },
                );
                aggregate_parameters.push((
                    parameter.name.clone(),
                    source_pointer,
                    destination,
                    parameter.ty.clone(),
                ));
            }
            primer_ir::Type::Array { element, length } => {
                parameters.push(Local {
                    name: parameter.name.clone(),
                    ty: Type::Pointer,
                });
                let source_pointer = *next_address;
                *next_address += 4;
                let destination = *next_address;
                *next_address += type_size(program, &parameter.ty);
                locations.insert(
                    parameter.id,
                    Location::Array {
                        element: array_element_type(element),
                        length: *length,
                        address: destination,
                    },
                );
                aggregate_parameters.push((
                    parameter.name.clone(),
                    source_pointer,
                    destination,
                    parameter.ty.clone(),
                ));
            }
        }
    }
    collect_locations(
        &function.body,
        program,
        &mut locals,
        &mut locations,
        &mut name_counts,
        next_address,
    );

    let mut context = LoweringContext {
        program,
        locations,
        next_address: *next_address,
        aggregate_return_address: aggregate_return_pointer.map(Address::Indirect),
        control: ControlContext {
            next_loop_id: 0,
            loops: Vec::new(),
        },
    };
    let mut instructions = Vec::new();
    if let Some(pointer) = aggregate_return_pointer {
        instructions.push(Instruction::I32Const(pointer as i32));
        instructions.push(Instruction::LocalGet("abi.result".into()));
        instructions.push(Instruction::I32Store { offset: 0 });
    }
    for (name, source_pointer, destination, ty) in aggregate_parameters {
        instructions.push(Instruction::I32Const(source_pointer as i32));
        instructions.push(Instruction::LocalGet(name));
        instructions.push(Instruction::I32Store { offset: 0 });
        context.copy_value(
            &ty,
            Address::Indirect(source_pointer),
            Address::Static(destination),
            &mut instructions,
        );
    }
    context.lower_statements(&function.body, &mut instructions);
    if matches!(function.return_type, primer_ir::ReturnType::Void)
        && !matches!(instructions.last(), Some(Instruction::Return))
    {
        instructions.push(Instruction::Return);
    }
    *next_address = context.next_address;

    Function {
        id: function.id.0,
        name: function.name.clone(),
        parameters,
        return_type: match &function.return_type {
            primer_ir::ReturnType::Void => None,
            primer_ir::ReturnType::Value(
                ty @ (primer_ir::Type::Bool
                | primer_ir::Type::Integer(_)
                | primer_ir::Type::F32
                | primer_ir::Type::F64),
            ) => Some(scalar_type(ty)),
            primer_ir::ReturnType::Value(
                primer_ir::Type::Named(_) | primer_ir::Type::Array { .. },
            ) => None,
        },
        locals,
        instructions,
    }
}

#[derive(Debug, Clone)]
enum Location {
    Scalar(String),
    Aggregate {
        type_id: usize,
        address: usize,
    },
    Array {
        element: ArrayElement,
        length: usize,
        address: usize,
    },
}

#[derive(Debug, Clone)]
enum Value {
    Scalar(Type),
    Aggregate {
        type_id: usize,
        address: Address,
    },
    Array {
        element: ArrayElement,
        length: usize,
        address: Address,
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
enum Address {
    /// lowering時点で位置が決まる、Wasm線形メモリ上のアドレス。
    Static(usize),
    /// 指定位置にi32として一時保存した、実行時に決まるアドレス。
    Indirect(usize),
}

struct LoweringContext<'a> {
    program: &'a primer_ir::Program,
    locations: HashMap<primer_ir::BindingId, Location>,
    next_address: usize,
    aggregate_return_address: Option<Address>,
    control: ControlContext,
}

struct ControlContext {
    next_loop_id: usize,
    loops: Vec<LoopTarget>,
}

#[derive(Debug, Clone, Copy)]
struct LoopTarget {
    kind: LoopKind,
    id: usize,
}

impl LoweringContext<'_> {
    fn lower_statements(
        &mut self,
        statements: &[primer_ir::Statement],
        instructions: &mut Vec<Instruction>,
    ) {
        for statement in statements {
            self.lower_statement(statement, instructions);
        }
    }

    fn lower_statement(
        &mut self,
        statement: &primer_ir::Statement,
        instructions: &mut Vec<Instruction>,
    ) {
        match &statement.kind {
            primer_ir::StatementKind::Binding { id, value, .. } => {
                self.assign_location(self.locations[id].clone(), value, instructions);
            }

            primer_ir::StatementKind::Assignment { target, value } => {
                if target.projections.is_empty() {
                    self.assign_location(self.locations[&target.id].clone(), value, instructions);
                } else {
                    let Location::Array { address, .. } = self.locations[&target.id].clone() else {
                        unreachable!("indexed assignment requires an array root")
                    };
                    let mut destination = Address::Static(address);
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
                            instructions,
                        );
                    }
                    self.assign_address(&target.ty, destination, value, instructions);
                }
            }

            primer_ir::StatementKind::Print { value } => {
                let Value::Scalar(ty) = self.lower_expr(value, instructions) else {
                    unreachable!("semantic analysis rejects aggregate printing")
                };
                instructions.push(Instruction::CallPrint(ty));
            }

            primer_ir::StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                let Value::Scalar(Type::Bool) = self.lower_expr(condition, instructions) else {
                    unreachable!("semantic analysis requires a bool condition")
                };
                let mut then_instructions = Vec::new();
                let mut else_instructions = Vec::new();
                self.lower_statements(then_body, &mut then_instructions);
                self.lower_statements(else_body, &mut else_instructions);
                instructions.push(Instruction::If {
                    then_instructions,
                    else_instructions,
                });
            }

            primer_ir::StatementKind::While { condition, body } => {
                let id = self.control.next_loop_id;
                self.control.next_loop_id += 1;
                let mut condition_instructions = Vec::new();
                let mut body_instructions = Vec::new();
                let Value::Scalar(Type::Bool) =
                    self.lower_expr(condition, &mut condition_instructions)
                else {
                    unreachable!("semantic analysis requires a bool condition")
                };
                self.control.loops.push(LoopTarget {
                    kind: LoopKind::While,
                    id,
                });
                self.lower_statements(body, &mut body_instructions);
                self.control
                    .loops
                    .pop()
                    .expect("while loop context must exist");
                instructions.push(Instruction::Loop {
                    kind: LoopKind::While,
                    id,
                    condition_instructions,
                    body_instructions,
                    update_instructions: Vec::new(),
                });
            }

            primer_ir::StatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                self.lower_statement(initializer, instructions);
                let id = self.control.next_loop_id;
                self.control.next_loop_id += 1;
                let mut condition_instructions = Vec::new();
                let mut body_instructions = Vec::new();
                let mut update_instructions = Vec::new();
                let Value::Scalar(Type::Bool) =
                    self.lower_expr(condition, &mut condition_instructions)
                else {
                    unreachable!("semantic analysis requires a bool condition")
                };
                self.control.loops.push(LoopTarget {
                    kind: LoopKind::For,
                    id,
                });
                self.lower_statements(body, &mut body_instructions);
                self.control
                    .loops
                    .pop()
                    .expect("for loop context must exist");
                self.lower_statement(update, &mut update_instructions);
                instructions.push(Instruction::Loop {
                    kind: LoopKind::For,
                    id,
                    condition_instructions,
                    body_instructions,
                    update_instructions,
                });
            }

            primer_ir::StatementKind::Break => {
                let target = *self
                    .control
                    .loops
                    .last()
                    .expect("semantic analysis rejects break outside a loop");
                instructions.push(Instruction::Break {
                    kind: target.kind,
                    id: target.id,
                });
            }

            primer_ir::StatementKind::Continue => {
                let target = *self
                    .control
                    .loops
                    .last()
                    .expect("semantic analysis rejects continue outside a loop");
                instructions.push(Instruction::Continue {
                    kind: target.kind,
                    id: target.id,
                });
            }
            primer_ir::StatementKind::Call {
                function_id,
                arguments,
                ..
            } => {
                self.lower_call(function_id.0, arguments, None, instructions);
            }
            primer_ir::StatementKind::Return { value } => {
                if let Some(value) = value {
                    match self.lower_expr(value, instructions) {
                        Value::Scalar(_) => {}
                        Value::Aggregate { address, .. } | Value::Array { address, .. } => {
                            let destination = self
                                .aggregate_return_address
                                .expect("aggregate functions have a hidden result address");
                            self.copy_value(&value.ty, address, destination, instructions);
                        }
                    }
                }
                instructions.push(Instruction::Return);
            }
        }
    }

    fn assign_location(
        &mut self,
        destination: Location,
        value: &primer_ir::Expr,
        instructions: &mut Vec<Instruction>,
    ) {
        match destination {
            Location::Scalar(name) => {
                let Value::Scalar(_) = self.lower_expr(value, instructions) else {
                    unreachable!("semantic analysis keeps assignment types equal")
                };
                instructions.push(Instruction::LocalSet(name));
            }
            Location::Aggregate { type_id, address } => {
                self.assign_address(
                    &primer_ir::Type::Named(primer_ir::TypeId(type_id)),
                    Address::Static(address),
                    value,
                    instructions,
                );
            }
            Location::Array {
                element,
                length,
                address,
            } => {
                let Value::Array {
                    element: source_element,
                    length: source_length,
                    address: source,
                } = self.lower_expr(value, instructions)
                else {
                    unreachable!("semantic analysis keeps assignment types equal")
                };
                debug_assert_eq!(element, source_element);
                debug_assert_eq!(length, source_length);
                self.copy_array(
                    &element,
                    length,
                    source,
                    Address::Static(address),
                    instructions,
                );
            }
        }
    }

    fn assign_address(
        &mut self,
        ty: &primer_ir::Type,
        destination: Address,
        value: &primer_ir::Expr,
        instructions: &mut Vec<Instruction>,
    ) {
        match ty {
            primer_ir::Type::Bool
            | primer_ir::Type::Integer(_)
            | primer_ir::Type::F32
            | primer_ir::Type::F64 => {
                self.emit_address(destination, instructions);
                let Value::Scalar(actual) = self.lower_expr(value, instructions) else {
                    unreachable!("semantic analysis keeps assignment types equal")
                };
                instructions.push(store_instruction(actual, 0));
            }
            primer_ir::Type::Named(type_id) => {
                let Value::Aggregate {
                    type_id: source_type,
                    address: source,
                } = self.lower_expr(value, instructions)
                else {
                    unreachable!("semantic analysis keeps assignment types equal")
                };
                debug_assert_eq!(type_id.0, source_type);
                self.copy_aggregate(type_id.0, source, destination, instructions);
            }
            primer_ir::Type::Array { element, length } => {
                let Value::Array {
                    element: source_element,
                    length: source_length,
                    address: source,
                } = self.lower_expr(value, instructions)
                else {
                    unreachable!("semantic analysis keeps assignment types equal")
                };
                let element = array_element_type(element);
                debug_assert_eq!(element, source_element);
                debug_assert_eq!(*length, source_length);
                self.copy_array(&element, *length, source, destination, instructions);
            }
        }
    }

    fn lower_checked_array_address(
        &mut self,
        base: Address,
        element: &ArrayElement,
        length: usize,
        index: &primer_ir::Expr,
        instructions: &mut Vec<Instruction>,
    ) -> Address {
        let index_address = self.allocate(8);
        instructions.push(Instruction::I32Const(index_address as i32));
        let Value::Scalar(Type::I64) = self.lower_expr(index, instructions) else {
            unreachable!("array index must be i64")
        };
        instructions.push(Instruction::I64Store { offset: 0 });

        instructions.push(Instruction::I32Const(index_address as i32));
        instructions.push(Instruction::I64Load { offset: 0 });
        instructions.push(Instruction::I64Const(0));
        instructions.push(Instruction::I64LtS);
        instructions.push(Instruction::If {
            then_instructions: vec![Instruction::Unreachable],
            else_instructions: Vec::new(),
        });
        instructions.push(Instruction::I32Const(index_address as i32));
        instructions.push(Instruction::I64Load { offset: 0 });
        instructions.push(Instruction::I64Const(length as i64));
        instructions.push(Instruction::I64GeS);
        instructions.push(Instruction::If {
            then_instructions: vec![Instruction::Unreachable],
            else_instructions: Vec::new(),
        });

        let result_address = self.allocate(4);
        instructions.push(Instruction::I32Const(result_address as i32));
        self.emit_indexed_address(
            base,
            index_address,
            array_element_size(self.program, element),
            instructions,
        );
        instructions.push(Instruction::I32Store { offset: 0 });
        Address::Indirect(result_address)
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr, instructions: &mut Vec<Instruction>) -> Value {
        let value = self.lower_expr_unchecked(expr, instructions);
        if let Some(ty) = super::super::integer_range_check(expr) {
            instructions.push(Instruction::CheckIntegerRange(ty));
        }
        value
    }

    fn lower_expr_unchecked(
        &mut self,
        expr: &primer_ir::Expr,
        instructions: &mut Vec<Instruction>,
    ) -> Value {
        match &expr.kind {
            primer_ir::ExprKind::ConvertInteger { value, .. } => {
                self.lower_expr(value, instructions)
            }
            primer_ir::ExprKind::Boolean(value) => {
                instructions.push(Instruction::I32Const(i32::from(*value)));
                Value::Scalar(Type::Bool)
            }
            primer_ir::ExprKind::Integer(value) => {
                instructions.push(Instruction::I64Const(*value));
                Value::Scalar(Type::I64)
            }
            primer_ir::ExprKind::Float { text } => {
                let ty = scalar_type(&expr.ty);
                match ty {
                    Type::F32 => instructions.push(Instruction::F32Const(text.clone())),
                    Type::F64 => instructions.push(Instruction::F64Const(text.clone())),
                    Type::Bool | Type::I64 | Type::Pointer => {
                        unreachable!("a float literal has a float type")
                    }
                }
                Value::Scalar(ty)
            }
            primer_ir::ExprKind::Variable { id, .. } => match &self.locations[id] {
                Location::Scalar(name) => {
                    instructions.push(Instruction::LocalGet(name.clone()));
                    Value::Scalar(scalar_type(&expr.ty))
                }
                Location::Aggregate { type_id, address } => Value::Aggregate {
                    type_id: *type_id,
                    address: Address::Static(*address),
                },
                Location::Array {
                    element,
                    length,
                    address,
                } => Value::Array {
                    element: element.clone(),
                    length: *length,
                    address: Address::Static(*address),
                },
            },
            primer_ir::ExprKind::Construct {
                type_id, fields, ..
            } => {
                let address = self.allocate(type_size(self.program, &expr.ty));
                for field in fields {
                    let field_definition =
                        &self.program.type_definitions[type_id.0].fields[field.id.0];
                    let destination = address + field_offset(self.program, type_id.0, field.id.0);
                    match &field_definition.ty {
                        primer_ir::Type::Named(nested) => {
                            let Value::Aggregate {
                                type_id: source_type,
                                address: source,
                            } = self.lower_expr(&field.value, instructions)
                            else {
                                unreachable!("semantic analysis keeps field types equal")
                            };
                            debug_assert_eq!(source_type, nested.0);
                            self.copy_aggregate(
                                nested.0,
                                source,
                                Address::Static(destination),
                                instructions,
                            );
                        }
                        primer_ir::Type::Array { element, length } => {
                            let Value::Array {
                                element: actual_element,
                                length: actual_length,
                                address: source,
                            } = self.lower_expr(&field.value, instructions)
                            else {
                                unreachable!("semantic analysis keeps field types equal")
                            };
                            let element = array_element_type(element);
                            debug_assert_eq!(element, actual_element);
                            debug_assert_eq!(*length, actual_length);
                            self.copy_array(
                                &element,
                                *length,
                                source,
                                Address::Static(destination),
                                instructions,
                            );
                        }
                        scalar => {
                            instructions.push(Instruction::I32Const(destination as i32));
                            let Value::Scalar(actual) = self.lower_expr(&field.value, instructions)
                            else {
                                unreachable!("semantic analysis keeps field types equal")
                            };
                            debug_assert_eq!(actual, scalar_type(scalar));
                            instructions.push(store_instruction(actual, 0));
                        }
                    }
                }
                Value::Aggregate {
                    type_id: type_id.0,
                    address: Address::Static(address),
                }
            }
            primer_ir::ExprKind::FieldAccess {
                type_id,
                field_id,
                base,
                ..
            } => {
                let Value::Aggregate { address, .. } = self.lower_expr(base, instructions) else {
                    unreachable!("semantic analysis requires an aggregate field base")
                };
                let address = self.offset_address(
                    address,
                    field_offset(self.program, type_id.0, field_id.0),
                    instructions,
                );
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
                        self.emit_address(address, instructions);
                        let ty = scalar_type(scalar);
                        instructions.push(load_instruction(ty, 0));
                        Value::Scalar(ty)
                    }
                }
            }
            primer_ir::ExprKind::Array(values) => {
                let primer_ir::Type::Array { element, length } = &expr.ty else {
                    unreachable!("array expression must have an array type")
                };
                let address = self.allocate(type_size(self.program, &expr.ty));
                let element = array_element_type(element);
                let stride = array_element_size(self.program, &element);
                for (index, value) in values.iter().enumerate() {
                    let destination = Address::Static(address + index * stride);
                    match &element {
                        ArrayElement::Scalar(expected) => {
                            self.emit_address(destination, instructions);
                            let Value::Scalar(actual) = self.lower_expr(value, instructions) else {
                                unreachable!("semantic analysis keeps array element types equal")
                            };
                            debug_assert_eq!(*expected, actual);
                            instructions.push(store_instruction(actual, 0));
                        }
                        ArrayElement::Named(expected) => {
                            let Value::Aggregate {
                                type_id,
                                address: source,
                            } = self.lower_expr(value, instructions)
                            else {
                                unreachable!("semantic analysis keeps array element types equal")
                            };
                            debug_assert_eq!(*expected, type_id);
                            self.copy_aggregate(type_id, source, destination, instructions);
                        }
                        ArrayElement::Array {
                            element: expected_element,
                            length: expected_length,
                        } => {
                            let Value::Array {
                                element,
                                length,
                                address: source,
                            } = self.lower_expr(value, instructions)
                            else {
                                unreachable!("semantic analysis keeps array element types equal")
                            };
                            debug_assert_eq!(**expected_element, element);
                            debug_assert_eq!(*expected_length, length);
                            self.copy_array(
                                expected_element,
                                *expected_length,
                                source,
                                destination,
                                instructions,
                            );
                        }
                    }
                }
                Value::Array {
                    element,
                    length: *length,
                    address: Address::Static(address),
                }
            }
            primer_ir::ExprKind::Index { base, index } => {
                let Value::Array {
                    element,
                    length,
                    address,
                } = self.lower_expr(base, instructions)
                else {
                    unreachable!("indexed expression must have an array base")
                };

                // 添字を一度だけ評価し、検査とアドレス計算で同じ値を使います。
                let index_address = self.allocate(8);
                instructions.push(Instruction::I32Const(index_address as i32));
                let Value::Scalar(Type::I64) = self.lower_expr(index, instructions) else {
                    unreachable!("array index must be i64")
                };
                instructions.push(Instruction::I64Store { offset: 0 });

                instructions.push(Instruction::I32Const(index_address as i32));
                instructions.push(Instruction::I64Load { offset: 0 });
                instructions.push(Instruction::I64Const(0));
                instructions.push(Instruction::I64LtS);
                instructions.push(Instruction::If {
                    then_instructions: vec![Instruction::Unreachable],
                    else_instructions: Vec::new(),
                });

                instructions.push(Instruction::I32Const(index_address as i32));
                instructions.push(Instruction::I64Load { offset: 0 });
                instructions.push(Instruction::I64Const(length as i64));
                instructions.push(Instruction::I64GeS);
                instructions.push(Instruction::If {
                    then_instructions: vec![Instruction::Unreachable],
                    else_instructions: Vec::new(),
                });

                match &element {
                    ArrayElement::Scalar(ty) => {
                        self.emit_indexed_address(
                            address,
                            index_address,
                            array_element_size(self.program, &element),
                            instructions,
                        );
                        instructions.push(load_instruction(*ty, 0));
                        Value::Scalar(*ty)
                    }
                    ArrayElement::Named(type_id) => {
                        let result_address = self.allocate(4);
                        instructions.push(Instruction::I32Const(result_address as i32));
                        self.emit_indexed_address(
                            address,
                            index_address,
                            array_element_size(self.program, &element),
                            instructions,
                        );
                        instructions.push(Instruction::I32Store { offset: 0 });
                        Value::Aggregate {
                            type_id: *type_id,
                            address: Address::Indirect(result_address),
                        }
                    }
                    ArrayElement::Array {
                        element: nested_element,
                        length: nested_length,
                    } => {
                        let result_address = self.allocate(4);
                        instructions.push(Instruction::I32Const(result_address as i32));
                        self.emit_indexed_address(
                            address,
                            index_address,
                            array_element_size(self.program, &element),
                            instructions,
                        );
                        instructions.push(Instruction::I32Store { offset: 0 });
                        Value::Array {
                            element: (**nested_element).clone(),
                            length: *nested_length,
                            address: Address::Indirect(result_address),
                        }
                    }
                }
            }
            primer_ir::ExprKind::Unary { op, value } => {
                let ty = scalar_type(&expr.ty);
                match (*op, ty) {
                    (primer_ir::UnaryOp::Negate, Type::I64) => {
                        instructions.push(Instruction::I64Const(0));
                        self.lower_expr(value, instructions);
                        instructions.push(Instruction::CheckedI64Sub);
                    }
                    (primer_ir::UnaryOp::Negate, Type::F32) => {
                        self.lower_expr(value, instructions);
                        instructions.push(Instruction::F32Neg)
                    }
                    (primer_ir::UnaryOp::Negate, Type::F64) => {
                        self.lower_expr(value, instructions);
                        instructions.push(Instruction::F64Neg)
                    }
                    (primer_ir::UnaryOp::Not, Type::Bool) => {
                        self.lower_expr(value, instructions);
                        instructions.push(Instruction::I32Eqz)
                    }
                    _ => unreachable!("semantic analysis rejects invalid unary operands"),
                }
                Value::Scalar(ty)
            }
            primer_ir::ExprKind::Logical { op, left, right } => {
                self.lower_expr(left, instructions);
                let mut rhs = Vec::new();
                self.lower_expr(right, &mut rhs);
                let (then_instructions, else_instructions) = match op {
                    primer_ir::LogicalOp::And => (rhs, vec![Instruction::I32Const(0)]),
                    primer_ir::LogicalOp::Or => (vec![Instruction::I32Const(1)], rhs),
                };
                instructions.push(Instruction::IfBool {
                    then_instructions,
                    else_instructions,
                });
                Value::Scalar(Type::Bool)
            }
            primer_ir::ExprKind::Binary { op, left, right } => {
                let Value::Scalar(left_ty) = self.lower_expr(left, instructions) else {
                    unreachable!("semantic analysis rejects aggregate binary operands")
                };
                let Value::Scalar(right_ty) = self.lower_expr(right, instructions) else {
                    unreachable!("semantic analysis rejects aggregate binary operands")
                };
                debug_assert_eq!(left_ty, right_ty);
                instructions.push(lower_binary(*op, left.ty.clone()));
                Value::Scalar(scalar_type(&expr.ty))
            }
            primer_ir::ExprKind::Call {
                function_id,
                arguments,
                ..
            } => self
                .lower_call(function_id.0, arguments, Some(&expr.ty), instructions)
                .expect("call expressions produce a value"),
        }
    }

    fn lower_call(
        &mut self,
        function_id: usize,
        arguments: &[primer_ir::Expr],
        result_type: Option<&primer_ir::Type>,
        instructions: &mut Vec<Instruction>,
    ) -> Option<Value> {
        let aggregate_result = result_type.and_then(|ty| match ty {
            primer_ir::Type::Named(_) | primer_ir::Type::Array { .. } => {
                let address = self.allocate(type_size(self.program, ty));
                instructions.push(Instruction::I32Const(address as i32));
                Some((ty, address))
            }
            primer_ir::Type::Bool
            | primer_ir::Type::Integer(_)
            | primer_ir::Type::F32
            | primer_ir::Type::F64 => None,
        });

        for argument in arguments {
            match self.lower_expr(argument, instructions) {
                Value::Scalar(_) => {}
                Value::Aggregate { address, .. } | Value::Array { address, .. } => {
                    self.emit_address(address, instructions);
                }
            }
        }
        instructions.push(Instruction::Call { function_id });

        if let Some((ty, address)) = aggregate_result {
            return Some(match ty {
                primer_ir::Type::Named(type_id) => Value::Aggregate {
                    type_id: type_id.0,
                    address: Address::Static(address),
                },
                primer_ir::Type::Array { element, length } => Value::Array {
                    element: array_element_type(element),
                    length: *length,
                    address: Address::Static(address),
                },
                primer_ir::Type::Bool
                | primer_ir::Type::Integer(_)
                | primer_ir::Type::F32
                | primer_ir::Type::F64 => unreachable!("aggregate result type is checked above"),
            });
        }

        result_type.map(|ty| Value::Scalar(scalar_type(ty)))
    }

    fn copy_value(
        &mut self,
        ty: &primer_ir::Type,
        source: Address,
        destination: Address,
        instructions: &mut Vec<Instruction>,
    ) {
        match ty {
            primer_ir::Type::Named(type_id) => {
                self.copy_aggregate(type_id.0, source, destination, instructions)
            }
            primer_ir::Type::Array { element, length } => self.copy_array(
                &array_element_type(element),
                *length,
                source,
                destination,
                instructions,
            ),
            scalar => {
                let ty = scalar_type(scalar);
                self.emit_address(destination, instructions);
                self.emit_address(source, instructions);
                instructions.push(load_instruction(ty, 0));
                instructions.push(store_instruction(ty, 0));
            }
        }
    }

    fn copy_aggregate(
        &mut self,
        type_id: usize,
        source: Address,
        destination: Address,
        instructions: &mut Vec<Instruction>,
    ) {
        for (field_id, field) in self.program.type_definitions[type_id]
            .fields
            .iter()
            .enumerate()
        {
            let offset = field_offset(self.program, type_id, field_id);
            let source = self.offset_address(source, offset, instructions);
            let destination = self.offset_address(destination, offset, instructions);
            match &field.ty {
                primer_ir::Type::Named(nested) => {
                    self.copy_aggregate(nested.0, source, destination, instructions)
                }
                primer_ir::Type::Array { element, length } => self.copy_array(
                    &array_element_type(element),
                    *length,
                    source,
                    destination,
                    instructions,
                ),
                scalar => {
                    let ty = scalar_type(scalar);
                    self.emit_address(destination, instructions);
                    self.emit_address(source, instructions);
                    instructions.push(load_instruction(ty, 0));
                    instructions.push(store_instruction(ty, 0));
                }
            }
        }
    }

    fn copy_array(
        &mut self,
        element: &ArrayElement,
        length: usize,
        source: Address,
        destination: Address,
        instructions: &mut Vec<Instruction>,
    ) {
        let stride = array_element_size(self.program, element);
        for index in 0..length {
            let offset = index * stride;
            let source = self.offset_address(source, offset, instructions);
            let destination = self.offset_address(destination, offset, instructions);
            match element {
                ArrayElement::Scalar(ty) => {
                    self.emit_address(destination, instructions);
                    self.emit_address(source, instructions);
                    instructions.push(load_instruction(*ty, 0));
                    instructions.push(store_instruction(*ty, 0));
                }
                ArrayElement::Named(type_id) => {
                    self.copy_aggregate(*type_id, source, destination, instructions)
                }
                ArrayElement::Array { element, length } => {
                    self.copy_array(element, *length, source, destination, instructions)
                }
            }
        }
    }

    fn emit_address(&self, address: Address, instructions: &mut Vec<Instruction>) {
        match address {
            Address::Static(address) => instructions.push(Instruction::I32Const(address as i32)),
            Address::Indirect(address) => {
                instructions.push(Instruction::I32Const(address as i32));
                instructions.push(Instruction::I32Load { offset: 0 });
            }
        }
    }

    fn offset_address(
        &mut self,
        address: Address,
        offset: usize,
        instructions: &mut Vec<Instruction>,
    ) -> Address {
        if offset == 0 {
            return address;
        }
        match address {
            Address::Static(address) => Address::Static(address + offset),
            Address::Indirect(_) => {
                let result = self.allocate(4);
                instructions.push(Instruction::I32Const(result as i32));
                self.emit_address(address, instructions);
                instructions.push(Instruction::I32Const(offset as i32));
                instructions.push(Instruction::I32Add);
                instructions.push(Instruction::I32Store { offset: 0 });
                Address::Indirect(result)
            }
        }
    }

    fn emit_indexed_address(
        &self,
        base: Address,
        index_address: usize,
        stride: usize,
        instructions: &mut Vec<Instruction>,
    ) {
        self.emit_address(base, instructions);
        instructions.push(Instruction::I32Const(index_address as i32));
        instructions.push(Instruction::I64Load { offset: 0 });
        instructions.push(Instruction::I32WrapI64);
        instructions.push(Instruction::I32Const(stride as i32));
        instructions.push(Instruction::I32Mul);
        instructions.push(Instruction::I32Add);
    }

    fn allocate(&mut self, size: usize) -> usize {
        let address = self.next_address;
        self.next_address += size;
        address
    }
}

fn collect_locations(
    statements: &[primer_ir::Statement],
    program: &primer_ir::Program,
    locals: &mut Vec<Local>,
    locations: &mut HashMap<primer_ir::BindingId, Location>,
    name_counts: &mut HashMap<String, usize>,
    next_address: &mut usize,
) {
    for statement in statements {
        match &statement.kind {
            primer_ir::StatementKind::Binding { id, name, ty, .. } => match ty {
                primer_ir::Type::Named(type_id) => {
                    let address = *next_address;
                    *next_address += type_size(program, ty);
                    locations.insert(
                        *id,
                        Location::Aggregate {
                            type_id: type_id.0,
                            address,
                        },
                    );
                }
                primer_ir::Type::Array { element, length } => {
                    let address = *next_address;
                    *next_address += type_size(program, ty);
                    locations.insert(
                        *id,
                        Location::Array {
                            element: array_element_type(element),
                            length: *length,
                            address,
                        },
                    );
                }
                scalar => {
                    let count = name_counts.entry(name.clone()).or_default();
                    let lowered_name = if *count == 0 {
                        name.clone()
                    } else {
                        format!("{name}_{}", id.0)
                    };
                    *count += 1;
                    locals.push(Local {
                        name: lowered_name.clone(),
                        ty: scalar_type(scalar),
                    });
                    locations.insert(*id, Location::Scalar(lowered_name));
                }
            },
            primer_ir::StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_locations(
                    then_body,
                    program,
                    locals,
                    locations,
                    name_counts,
                    next_address,
                );
                collect_locations(
                    else_body,
                    program,
                    locals,
                    locations,
                    name_counts,
                    next_address,
                );
            }
            primer_ir::StatementKind::While { body, .. } => {
                collect_locations(body, program, locals, locations, name_counts, next_address)
            }
            primer_ir::StatementKind::For {
                initializer,
                update,
                body,
                ..
            } => {
                collect_locations(
                    std::slice::from_ref(initializer),
                    program,
                    locals,
                    locations,
                    name_counts,
                    next_address,
                );
                collect_locations(
                    std::slice::from_ref(update),
                    program,
                    locals,
                    locations,
                    name_counts,
                    next_address,
                );
                collect_locations(body, program, locals, locations, name_counts, next_address);
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
        primer_ir::Type::Bool
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

fn array_element_size(program: &primer_ir::Program, element: &ArrayElement) -> usize {
    match element {
        ArrayElement::Scalar(_) => 8,
        ArrayElement::Named(id) => {
            type_size(program, &primer_ir::Type::Named(primer_ir::TypeId(*id)))
        }
        ArrayElement::Array { element, length } => array_element_size(program, element) * length,
    }
}

fn load_instruction(ty: Type, offset: u32) -> Instruction {
    match ty {
        Type::Bool => Instruction::I32Load { offset },
        Type::I64 => Instruction::I64Load { offset },
        Type::F32 => Instruction::F32Load { offset },
        Type::F64 => Instruction::F64Load { offset },
        Type::Pointer => unreachable!("pointers are not loaded as Primer scalar values"),
    }
}

fn store_instruction(ty: Type, offset: u32) -> Instruction {
    match ty {
        Type::Bool => Instruction::I32Store { offset },
        Type::I64 => Instruction::I64Store { offset },
        Type::F32 => Instruction::F32Store { offset },
        Type::F64 => Instruction::F64Store { offset },
        Type::Pointer => unreachable!("pointers are not stored as Primer scalar values"),
    }
}

fn lower_binary(op: primer_ir::BinaryOp, ty: primer_ir::Type) -> Instruction {
    match (op, ty) {
        (primer_ir::BinaryOp::Add, primer_ir::Type::Integer(_)) => Instruction::CheckedI64Add,
        (primer_ir::BinaryOp::Subtract, primer_ir::Type::Integer(_)) => Instruction::CheckedI64Sub,
        (primer_ir::BinaryOp::Multiply, primer_ir::Type::Integer(_)) => Instruction::CheckedI64Mul,
        (primer_ir::BinaryOp::Divide, primer_ir::Type::Integer(_)) => Instruction::CheckedI64DivS,
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
        (primer_ir::BinaryOp::Equal, primer_ir::Type::Integer(_)) => Instruction::I64Eq,
        (primer_ir::BinaryOp::NotEqual, primer_ir::Type::Integer(_)) => Instruction::I64Ne,
        (primer_ir::BinaryOp::Less, primer_ir::Type::Integer(_)) => Instruction::I64LtS,
        (primer_ir::BinaryOp::LessEqual, primer_ir::Type::Integer(_)) => Instruction::I64LeS,
        (primer_ir::BinaryOp::Greater, primer_ir::Type::Integer(_)) => Instruction::I64GtS,
        (primer_ir::BinaryOp::GreaterEqual, primer_ir::Type::Integer(_)) => Instruction::I64GeS,
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
        _ => unreachable!("semantic analysis rejects invalid binary operands"),
    }
}
