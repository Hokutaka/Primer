use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{Instruction, Local, LoopKind, Module, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let mut locals = Vec::new();
    let mut locations = HashMap::new();
    let mut name_counts = HashMap::new();
    let mut next_address = 0;
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
        locals,
        instructions,
    }
}

#[derive(Debug, Clone)]
enum Location {
    Scalar(String),
    Aggregate { type_id: usize, address: usize },
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Scalar(Type),
    Aggregate { type_id: usize, address: usize },
}

struct LoweringContext<'a> {
    program: &'a primer_ir::Program,
    locations: HashMap<primer_ir::BindingId, Location>,
    next_address: usize,
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
            primer_ir::StatementKind::Binding { id, value, .. }
            | primer_ir::StatementKind::Assignment { id, value, .. } => {
                let destination = self.locations[id].clone();
                match destination {
                    Location::Scalar(name) => {
                        let Value::Scalar(_) = self.lower_expr(value, instructions) else {
                            unreachable!("semantic analysis keeps assignment types equal")
                        };
                        instructions.push(Instruction::LocalSet(name));
                    }
                    Location::Aggregate { type_id, address } => {
                        let Value::Aggregate {
                            type_id: source_type,
                            address: source,
                        } = self.lower_expr(value, instructions)
                        else {
                            unreachable!("semantic analysis keeps assignment types equal")
                        };
                        debug_assert_eq!(type_id, source_type);
                        self.copy_aggregate(type_id, source, address, instructions);
                    }
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
        }
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr, instructions: &mut Vec<Instruction>) -> Value {
        match &expr.kind {
            primer_ir::ExprKind::Boolean(value) => {
                instructions.push(Instruction::I32Const(i32::from(*value)));
                Value::Scalar(Type::Bool)
            }
            primer_ir::ExprKind::Integer(value) => {
                instructions.push(Instruction::I64Const(*value));
                Value::Scalar(Type::I64)
            }
            primer_ir::ExprKind::Float { text } => {
                let ty = scalar_type(expr.ty);
                match ty {
                    Type::F32 => instructions.push(Instruction::F32Const(text.clone())),
                    Type::F64 => instructions.push(Instruction::F64Const(text.clone())),
                    Type::Bool | Type::I64 => unreachable!("a float literal has a float type"),
                }
                Value::Scalar(ty)
            }
            primer_ir::ExprKind::Variable { id, .. } => match &self.locations[id] {
                Location::Scalar(name) => {
                    instructions.push(Instruction::LocalGet(name.clone()));
                    Value::Scalar(scalar_type(expr.ty))
                }
                Location::Aggregate { type_id, address } => Value::Aggregate {
                    type_id: *type_id,
                    address: *address,
                },
            },
            primer_ir::ExprKind::Construct {
                type_id, fields, ..
            } => {
                let address = self.allocate(type_size(self.program, expr.ty));
                for field in fields {
                    let field_definition =
                        &self.program.type_definitions[type_id.0].fields[field.id.0];
                    let destination = address + field_offset(self.program, type_id.0, field.id.0);
                    match field_definition.ty {
                        primer_ir::Type::Named(nested) => {
                            let Value::Aggregate {
                                type_id: source_type,
                                address: source,
                            } = self.lower_expr(&field.value, instructions)
                            else {
                                unreachable!("semantic analysis keeps field types equal")
                            };
                            debug_assert_eq!(source_type, nested.0);
                            self.copy_aggregate(nested.0, source, destination, instructions);
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
                    address,
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
                let address = address + field_offset(self.program, type_id.0, field_id.0);
                match expr.ty {
                    primer_ir::Type::Named(nested) => Value::Aggregate {
                        type_id: nested.0,
                        address,
                    },
                    scalar => {
                        instructions.push(Instruction::I32Const(address as i32));
                        let ty = scalar_type(scalar);
                        instructions.push(load_instruction(ty, 0));
                        Value::Scalar(ty)
                    }
                }
            }
            primer_ir::ExprKind::Unary { op, value } => {
                let ty = scalar_type(expr.ty);
                match (*op, ty) {
                    (primer_ir::UnaryOp::Negate, Type::I64) => {
                        instructions.push(Instruction::I64Const(0));
                        self.lower_expr(value, instructions);
                        instructions.push(Instruction::I64Sub);
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
            primer_ir::ExprKind::Binary { op, left, right } => {
                let Value::Scalar(left_ty) = self.lower_expr(left, instructions) else {
                    unreachable!("semantic analysis rejects aggregate binary operands")
                };
                let Value::Scalar(right_ty) = self.lower_expr(right, instructions) else {
                    unreachable!("semantic analysis rejects aggregate binary operands")
                };
                debug_assert_eq!(left_ty, right_ty);
                instructions.push(lower_binary(*op, left.ty));
                Value::Scalar(scalar_type(expr.ty))
            }
        }
    }

    fn copy_aggregate(
        &self,
        type_id: usize,
        source: usize,
        destination: usize,
        instructions: &mut Vec<Instruction>,
    ) {
        for (field_id, field) in self.program.type_definitions[type_id]
            .fields
            .iter()
            .enumerate()
        {
            let offset = field_offset(self.program, type_id, field_id);
            match field.ty {
                primer_ir::Type::Named(nested) => self.copy_aggregate(
                    nested.0,
                    source + offset,
                    destination + offset,
                    instructions,
                ),
                scalar => {
                    let ty = scalar_type(scalar);
                    instructions.push(Instruction::I32Const((destination + offset) as i32));
                    instructions.push(Instruction::I32Const((source + offset) as i32));
                    instructions.push(load_instruction(ty, 0));
                    instructions.push(store_instruction(ty, 0));
                }
            }
        }
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
                    *next_address += type_size(program, *ty);
                    locations.insert(
                        *id,
                        Location::Aggregate {
                            type_id: type_id.0,
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
                        ty: scalar_type(*scalar),
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
        primer_ir::Type::F32 => Type::F32,
        primer_ir::Type::F64 => Type::F64,
        primer_ir::Type::Named(_) => unreachable!("expected a scalar type"),
    }
}

fn load_instruction(ty: Type, offset: u32) -> Instruction {
    match ty {
        Type::Bool => Instruction::I32Load { offset },
        Type::I64 => Instruction::I64Load { offset },
        Type::F32 => Instruction::F32Load { offset },
        Type::F64 => Instruction::F64Load { offset },
    }
}

fn store_instruction(ty: Type, offset: u32) -> Instruction {
    match ty {
        Type::Bool => Instruction::I32Store { offset },
        Type::I64 => Instruction::I64Store { offset },
        Type::F32 => Instruction::F32Store { offset },
        Type::F64 => Instruction::F64Store { offset },
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
        _ => unreachable!("semantic analysis rejects invalid binary operands"),
    }
}
