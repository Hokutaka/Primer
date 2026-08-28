use std::collections::HashMap;

use crate::ir as primer_ir;

use super::ir::{BinaryOp, FloatConstant, Instruction, Module, Type};

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
    };

    for statement in &program.statements {
        lowerer.lower_statement(statement);
    }

    Module {
        frame_size,
        float_constants: lowerer.float_constants,
        instructions: lowerer.instructions,
    }
}

struct Lowerer {
    binding_slots: HashMap<String, usize>,
    scratch_base: usize,
    float_id: usize,
    float_constants: Vec<FloatConstant>,
    instructions: Vec<Instruction>,
}

impl Lowerer {
    fn lower_statement(&mut self, statement: &primer_ir::Statement) {
        match statement {
            primer_ir::Statement::Binding { name, ty, value } => {
                self.lower_expr(value, 0);

                let offset = self.binding_offset(name);

                match Type::from(*ty) {
                    Type::I64 => {
                        self.instructions.push(Instruction::StoreI64ToStack(offset));
                    }

                    Type::F32 => {
                        self.instructions.push(Instruction::StoreF32ToStack(offset));
                    }

                    Type::F64 => {
                        self.instructions.push(Instruction::StoreF64ToStack(offset));
                    }
                }
            }

            primer_ir::Statement::Print { value } => {
                self.lower_expr(value, 0);

                self.lower_print(value.ty.into());
            }
        }
    }

    fn lower_expr(&mut self, expr: &primer_ir::Expr, depth: usize) -> Type {
        match &expr.kind {
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

                    Type::I64 => {
                        unreachable!("integer cannot be lowered as float");
                    }
                }

                ty
            }

            primer_ir::ExprKind::Variable(name) => {
                let ty = expr.ty.into();
                let offset = self.binding_offset(name);

                match ty {
                    Type::I64 => {
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
                }

                ty
            }

            primer_ir::ExprKind::Binary { op, left, right } => {
                let ty = expr.ty.into();

                // 左辺を計算。
                self.lower_expr(left, depth + 1);

                // 左辺をscratchへ退避。
                let scratch = self.scratch_offset(depth);

                match ty {
                    Type::I64 => {
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
                self.lower_expr(right, depth + 1);

                match ty {
                    Type::I64 => {
                        self.instructions.push(Instruction::MoveRaxToRcx);
                        self.instructions
                            .push(Instruction::LoadI64ScratchToRax(scratch));

                        let op = (*op).into();

                        match op {
                            BinaryOp::Divide => {
                                self.instructions.push(Instruction::SignExtendRax);
                                self.instructions.push(Instruction::DivideRaxByRcx);
                            }

                            _ => {
                                self.instructions.push(Instruction::I64Binary(op));
                            }
                        }
                    }

                    Type::F32 => {
                        self.instructions.push(Instruction::CopyXmm0ToXmm1F32);
                        self.instructions
                            .push(Instruction::LoadF32ScratchToXmm0(scratch));
                        self.instructions.push(Instruction::F32Binary((*op).into()));
                    }

                    Type::F64 => {
                        self.instructions.push(Instruction::CopyXmm0ToXmm1F64);
                        self.instructions
                            .push(Instruction::LoadF64ScratchToXmm0(scratch));
                        self.instructions.push(Instruction::F64Binary((*op).into()));
                    }
                }

                ty
            }
        }
    }

    fn lower_print(&mut self, ty: Type) {
        match ty {
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

            Type::I64 => {
                unreachable!("integer cannot be lowered as float");
            }
        }

        id
    }

    fn binding_offset(&self, name: &str) -> isize {
        let slot = self
            .binding_slots
            .get(name)
            .copied()
            .expect("binding must have a stack slot");

        slot_offset(slot)
    }

    fn scratch_offset(&self, depth: usize) -> isize {
        slot_offset(self.scratch_base + depth)
    }
}

fn assign_binding_slots(program: &primer_ir::Program) -> HashMap<String, usize> {
    let mut slots = HashMap::new();
    let mut next = 0;

    for statement in &program.statements {
        if let primer_ir::Statement::Binding { name, .. } = statement {
            slots.insert(name.clone(), next);
            next += 1;
        }
    }

    slots
}

fn count_program_expr_nodes(program: &primer_ir::Program) -> usize {
    program
        .statements
        .iter()
        .map(|statement| match statement {
            primer_ir::Statement::Binding { value, .. } | primer_ir::Statement::Print { value } => {
                count_expr_nodes(value)
            }
        })
        .sum()
}

fn count_expr_nodes(expr: &primer_ir::Expr) -> usize {
    match &expr.kind {
        primer_ir::ExprKind::Integer(_)
        | primer_ir::ExprKind::Float { .. }
        | primer_ir::ExprKind::Variable(_) => 1,

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
        }
    }
}
