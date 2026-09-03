use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{BinaryOp, CompareOp, FloatConstant, Instruction, Module, Type};

pub fn lower(program: &primer_ir::Program) -> Module {
    let binding_slots = assign_binding_slots(program);

    // 少し多めにscratch領域を確保。
    // Primer v0.1は小さいので、まず単純さ優先。
    let scratch_count = count_program_expr_nodes(program).max(1);

    let scratch_base = binding_slots.len();

    // Windows x64 ABIではcall時に32-byte shadow spaceが必要。
    let local_bytes = 8 * (binding_slots.len() + scratch_count);
    let frame_size = align16(32 + local_bytes);

    let mut lowerer = Lowerer {
        binding_slots,
        scratch_base,
        float_id: 0,
        float_constants: Vec::new(),
        instructions: Vec::new(),
        label: 0,
        loops: Vec::new(),
    };

    lowerer.lower_statements(&program.statements);

    Module {
        frame_size,
        float_constants: lowerer.float_constants,
        instructions: lowerer.instructions,
    }
}

struct Lowerer {
    binding_slots: HashMap<primer_ir::BindingId, usize>,
    scratch_base: usize,
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
                self.lower_expr(value, 0);

                let offset = self.binding_offset(*id);

                match Type::from(*ty) {
                    Type::Bool | Type::I64 => {
                        self.instructions.push(Instruction::StoreI64ToStack(offset));
                    }

                    Type::F32 => {
                        self.instructions.push(Instruction::StoreF32ToStack(offset));
                    }

                    Type::F64 => {
                        self.instructions.push(Instruction::StoreF64ToStack(offset));
                    }
                }
                false
            }

            primer_ir::StatementKind::Assignment { id, ty, value, .. } => {
                self.lower_expr(value, 0);

                let offset = self.binding_offset(*id);

                match Type::from(*ty) {
                    Type::Bool | Type::I64 => {
                        self.instructions.push(Instruction::StoreI64ToStack(offset));
                    }

                    Type::F32 => {
                        self.instructions.push(Instruction::StoreF32ToStack(offset));
                    }

                    Type::F64 => {
                        self.instructions.push(Instruction::StoreF64ToStack(offset));
                    }
                }
                false
            }

            primer_ir::StatementKind::Print { value } => {
                self.lower_expr(value, 0);

                self.lower_print(value.ty.into());
                false
            }

            primer_ir::StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                self.lower_expr(condition, 0);
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
                self.lower_expr(condition, 0);
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

    fn lower_expr(&mut self, expr: &primer_ir::Expr, depth: usize) -> Type {
        match &expr.kind {
            primer_ir::ExprKind::Boolean(value) => {
                self.instructions
                    .push(Instruction::MovI64ImmediateToRax(i64::from(*value)));

                Type::Bool
            }

            primer_ir::ExprKind::Integer(value) => {
                self.instructions
                    .push(Instruction::MovI64ImmediateToRax(*value));

                Type::I64
            }

            primer_ir::ExprKind::Float { text } => {
                let ty = expr.ty.into();
                let id = self.add_float_constant(text, ty);

                match ty {
                    Type::F32 => {
                        self.instructions.push(Instruction::LoadF32Constant(id));
                    }

                    Type::F64 => {
                        self.instructions.push(Instruction::LoadF64Constant(id));
                    }

                    Type::Bool | Type::I64 => {
                        unreachable!("integer cannot be lowered as float");
                    }
                }

                ty
            }

            primer_ir::ExprKind::Variable { id, .. } => {
                let ty = expr.ty.into();
                let offset = self.binding_offset(*id);

                match ty {
                    Type::Bool | Type::I64 => {
                        self.instructions
                            .push(Instruction::LoadI64FromStack(offset));
                    }

                    Type::F32 => {
                        self.instructions
                            .push(Instruction::LoadF32FromStack(offset));
                    }

                    Type::F64 => {
                        self.instructions
                            .push(Instruction::LoadF64FromStack(offset));
                    }
                }

                ty
            }

            primer_ir::ExprKind::Unary { op, value } => {
                let ty = expr.ty.into();

                self.lower_expr(value, depth);

                match (*op, ty) {
                    (primer_ir::UnaryOp::Negate, Type::I64) => {
                        self.instructions.push(Instruction::NegI64);
                    }

                    (primer_ir::UnaryOp::Negate, Type::F32) => {
                        self.instructions.push(Instruction::NegF32);
                    }

                    (primer_ir::UnaryOp::Negate, Type::F64) => {
                        self.instructions.push(Instruction::NegF64);
                    }

                    (primer_ir::UnaryOp::Not, Type::Bool) => {
                        self.instructions.push(Instruction::NotBool);
                    }

                    (primer_ir::UnaryOp::Negate, Type::Bool)
                    | (primer_ir::UnaryOp::Not, Type::I64 | Type::F32 | Type::F64) => {
                        unreachable!("semantic analysis rejects invalid unary operands");
                    }
                }

                ty
            }

            primer_ir::ExprKind::Binary { op, left, right } => {
                let operand_ty = self.lower_expr(left, depth + 1);

                // 左辺を計算。
                // 左辺をscratchへ退避。
                let scratch = self.scratch_offset(depth);

                match operand_ty {
                    Type::Bool | Type::I64 => {
                        self.instructions
                            .push(Instruction::StoreI64ToStack(scratch));
                    }

                    Type::F32 => {
                        self.instructions
                            .push(Instruction::StoreF32ToStack(scratch));
                    }

                    Type::F64 => {
                        self.instructions
                            .push(Instruction::StoreF64ToStack(scratch));
                    }
                }

                // 右辺を計算。
                let right_ty = self.lower_expr(right, depth + 1);
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

                expr.ty.into()
            }
        }
    }

    fn lower_print(&mut self, ty: Type) {
        match ty {
            Type::Bool => {
                self.instructions.push(Instruction::CallPrintBool);
            }

            Type::I64 => {
                self.instructions.push(Instruction::MoveRaxToRdx);
                self.instructions.push(Instruction::LoadFormatI64ToRcx);
                self.instructions.push(Instruction::CallPrintf);
            }

            Type::F32 => {
                // C varargs:
                // float -> double
                self.instructions.push(Instruction::ConvertF32ToF64Argument);

                // Windows x64 varargs requires
                // floating-point arg duplicated
                // into the corresponding GP register.
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

            Type::Bool | Type::I64 => {
                unreachable!("integer cannot be lowered as float");
            }
        }

        id
    }

    fn binding_offset(&self, id: primer_ir::BindingId) -> isize {
        let slot = self
            .binding_slots
            .get(&id)
            .copied()
            .expect("binding must have a stack slot");

        slot_offset(slot)
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

fn assign_binding_slots(program: &primer_ir::Program) -> HashMap<primer_ir::BindingId, usize> {
    let mut slots = HashMap::new();
    let mut next = 0;
    collect_binding_slots(&program.statements, &mut slots, &mut next);

    slots
}

fn collect_binding_slots(
    statements: &[primer_ir::Statement],
    slots: &mut HashMap<primer_ir::BindingId, usize>,
    next: &mut usize,
) {
    for statement in statements {
        match &statement.kind {
            primer_ir::StatementKind::Binding { id, .. } => {
                slots.insert(*id, *next);
                *next += 1;
            }
            primer_ir::StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_binding_slots(then_body, slots, next);
                collect_binding_slots(else_body, slots, next);
            }
            primer_ir::StatementKind::While { body, .. } => {
                collect_binding_slots(body, slots, next);
            }
            primer_ir::StatementKind::Assignment { .. }
            | primer_ir::StatementKind::Print { .. }
            | primer_ir::StatementKind::Break
            | primer_ir::StatementKind::Continue => {}
        }
    }
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
    }
}

fn slot_offset(slot: usize) -> isize {
    -8 * (slot as isize + 1)
}

fn align16(value: usize) -> usize {
    (value + 15) & !15
}

impl From<primer_ir::Type> for Type {
    fn from(value: primer_ir::Type) -> Self {
        match value {
            primer_ir::Type::Bool => Self::Bool,
            primer_ir::Type::I64 => Self::I64,
            primer_ir::Type::F32 => Self::F32,
            primer_ir::Type::F64 => Self::F64,
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
