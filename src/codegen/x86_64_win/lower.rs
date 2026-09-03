use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{BinaryOp, CompareOp, FloatConstant, Instruction, Module, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let (binding_slots, binding_slot_count) = assign_binding_slots(program);

    // 二項演算の左辺を一時退避する領域を、式の深さに十分なだけ確保する。
    let scratch_count = count_program_expr_nodes(program).max(1);
    let scratch_base = binding_slot_count;
    let aggregate_base = scratch_base + scratch_count;

    let mut lowerer = Lowerer {
        program,
        binding_slots,
        scratch_base,
        next_aggregate_slot: aggregate_base,
        float_id: 0,
        float_constants: Vec::new(),
        instructions: Vec::new(),
        label: 0,
        loops: Vec::new(),
    };

    lowerer.lower_statements(&program.statements);

    // Windows x64 ABI では、関数呼び出し用に 32 バイトの shadow space が必要になる。
    let local_bytes = 8 * lowerer.next_aggregate_slot;
    let frame_size = align16(32 + local_bytes);

    Module {
        frame_size,
        float_constants: lowerer.float_constants,
        instructions: lowerer.instructions,
    }
}

struct Lowerer<'a> {
    program: &'a primer_ir::Program,
    binding_slots: HashMap<primer_ir::BindingId, usize>,
    scratch_base: usize,
    next_aggregate_slot: usize,
    float_id: usize,
    float_constants: Vec<FloatConstant>,
    instructions: Vec<Instruction>,
    label: usize,
    loops: Vec<LoopContext>,
}

#[derive(Debug, Clone, Copy)]
struct LoopContext {
    continue_label: usize,
    break_label: usize,
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Scalar(Type),
    Aggregate { type_id: usize, base_slot: usize },
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
                let value = self.lower_expr(value, 0);
                let destination = self.binding_slot(*id);
                match (*ty, value) {
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
                    (scalar, Value::Scalar(actual)) => {
                        debug_assert_eq!(scalar_type(scalar), actual);
                        self.store_scalar(actual, slot_offset(destination));
                    }
                    _ => unreachable!("semantic analysis keeps assignment types equal"),
                }
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
            primer_ir::StatementKind::Call { .. } | primer_ir::StatementKind::Return { .. } => {
                unreachable!("functions are rejected before x86-64 lowering")
            }
        }
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr, depth: usize) -> Value {
        match &expr.kind {
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
                let ty = scalar_type(expr.ty);
                let id = self.add_float_constant(text, ty);
                match ty {
                    Type::F32 => self.instructions.push(Instruction::LoadF32Constant(id)),
                    Type::F64 => self.instructions.push(Instruction::LoadF64Constant(id)),
                    Type::Bool | Type::I64 => unreachable!("a float literal has a float type"),
                }
                Value::Scalar(ty)
            }
            primer_ir::ExprKind::Variable { id, .. } => match expr.ty {
                primer_ir::Type::Named(type_id) => Value::Aggregate {
                    type_id: type_id.0,
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
                    (primer_ir::UnaryOp::Negate, Type::I64) => {
                        self.instructions.push(Instruction::NegI64)
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
                    Type::Bool | Type::I64 => {
                        self.instructions.push(Instruction::MoveRaxToRcx);
                        self.instructions
                            .push(Instruction::LoadI64ScratchToRax(scratch));

                        if let Some(op) = compare_op(*op) {
                            self.instructions.push(Instruction::CompareI64(op));
                        } else {
                            let op = (*op).into();
                            if op == BinaryOp::Divide {
                                self.instructions.push(Instruction::SignExtendRax);
                                self.instructions.push(Instruction::DivideRaxByRcx);
                            } else {
                                self.instructions.push(Instruction::I64Binary(op));
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

                Value::Scalar(scalar_type(expr.ty))
            }
            primer_ir::ExprKind::Construct {
                type_id, fields, ..
            } => {
                let destination = self.allocate_aggregate(expr.ty);
                for field in fields {
                    let definition = &self.program.type_definitions[type_id.0].fields[field.id.0];
                    let value = self.lower_expr(&field.value, depth);
                    let field_slot =
                        destination + field_slot_offset(self.program, type_id.0, field.id.0);
                    match (definition.ty, value) {
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
                match expr.ty {
                    primer_ir::Type::Named(nested) => Value::Aggregate {
                        type_id: nested.0,
                        base_slot: field_slot,
                    },
                    scalar => {
                        let ty = scalar_type(scalar);
                        self.load_scalar(ty, slot_offset(field_slot));
                        Value::Scalar(ty)
                    }
                }
            }
            primer_ir::ExprKind::Call { .. } => {
                unreachable!("functions are rejected before x86-64 lowering")
            }
        }
    }

    fn copy_aggregate(&mut self, type_id: usize, source: usize, destination: usize) {
        for (field_id, field) in self.program.type_definitions[type_id]
            .fields
            .iter()
            .enumerate()
        {
            let offset = field_slot_offset(self.program, type_id, field_id);
            match field.ty {
                primer_ir::Type::Named(nested) => {
                    self.copy_aggregate(nested.0, source + offset, destination + offset)
                }
                scalar => {
                    let ty = scalar_type(scalar);
                    self.load_scalar(ty, slot_offset(source + offset));
                    self.store_scalar(ty, slot_offset(destination + offset));
                }
            }
        }
    }

    fn load_scalar(&mut self, ty: Type, offset: isize) {
        self.instructions.push(match ty {
            Type::Bool | Type::I64 => Instruction::LoadI64FromStack(offset),
            Type::F32 => Instruction::LoadF32FromStack(offset),
            Type::F64 => Instruction::LoadF64FromStack(offset),
        });
    }

    fn store_scalar(&mut self, ty: Type, offset: isize) {
        self.instructions.push(match ty {
            Type::Bool | Type::I64 => Instruction::StoreI64ToStack(offset),
            Type::F32 => Instruction::StoreF32ToStack(offset),
            Type::F64 => Instruction::StoreF64ToStack(offset),
        });
    }

    fn lower_print(&mut self, ty: Type) {
        match ty {
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
            Type::Bool | Type::I64 => unreachable!("a float literal has a float type"),
        }

        id
    }

    fn allocate_aggregate(&mut self, ty: primer_ir::Type) -> usize {
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
) -> (HashMap<primer_ir::BindingId, usize>, usize) {
    let mut slots = HashMap::new();
    let mut next = 0;
    collect_binding_slots(&program.statements, program, &mut slots, &mut next);
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
                *next += type_slot_count(program, *ty);
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

fn type_slot_count(program: &primer_ir::Program, ty: primer_ir::Type) -> usize {
    match ty {
        primer_ir::Type::Bool
        | primer_ir::Type::I64
        | primer_ir::Type::F32
        | primer_ir::Type::F64 => 1,
        primer_ir::Type::Named(id) => program.type_definitions[id.0]
            .fields
            .iter()
            .map(|field| type_slot_count(program, field.ty))
            .sum(),
    }
}

fn field_slot_offset(program: &primer_ir::Program, type_id: usize, field_id: usize) -> usize {
    program.type_definitions[type_id].fields[..field_id]
        .iter()
        .map(|field| type_slot_count(program, field.ty))
        .sum()
}

fn count_program_expr_nodes(program: &primer_ir::Program) -> usize {
    count_statements_expr_nodes(&program.statements)
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
        primer_ir::ExprKind::Boolean(_)
        | primer_ir::ExprKind::Integer(_)
        | primer_ir::ExprKind::Float { .. }
        | primer_ir::ExprKind::Variable { .. } => 1,
        primer_ir::ExprKind::Unary { value, .. } => 1 + count_expr_nodes(value),
        primer_ir::ExprKind::Binary { left, right, .. } => {
            1 + count_expr_nodes(left) + count_expr_nodes(right)
        }
        primer_ir::ExprKind::Construct { fields, .. } => {
            1 + fields
                .iter()
                .map(|field| count_expr_nodes(&field.value))
                .sum::<usize>()
        }
        primer_ir::ExprKind::FieldAccess { base, .. } => 1 + count_expr_nodes(base),
        primer_ir::ExprKind::Call { arguments, .. } => {
            1 + arguments.iter().map(count_expr_nodes).sum::<usize>()
        }
    }
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
        | primer_ir::BinaryOp::Divide => None,
        primer_ir::BinaryOp::Equal => Some(CompareOp::Equal),
        primer_ir::BinaryOp::NotEqual => Some(CompareOp::NotEqual),
        primer_ir::BinaryOp::Less => Some(CompareOp::Less),
        primer_ir::BinaryOp::LessEqual => Some(CompareOp::LessEqual),
        primer_ir::BinaryOp::Greater => Some(CompareOp::Greater),
        primer_ir::BinaryOp::GreaterEqual => Some(CompareOp::GreaterEqual),
    }
}
