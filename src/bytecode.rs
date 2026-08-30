use std::{collections::HashMap, fmt::Write};

use crate::{
    ir::{self, BinaryOp, Expr, ExprKind, Program, Statement, StatementKind, UnaryOp},
    source::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone)]
pub struct BytecodeProgram {
    pub slots: Vec<Slot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub origin: InstructionOrigin,
}

impl Instruction {
    /// ソースコードに由来する命令を作ります。
    pub const fn source(kind: InstructionKind, span: Span) -> Self {
        Self {
            kind,
            origin: InstructionOrigin::Source(span),
        }
    }

    /// コンパイラが補助的に生成した命令を作ります。
    pub const fn synthetic(kind: InstructionKind) -> Self {
        Self {
            kind,
            origin: InstructionOrigin::Synthetic,
        }
    }
}

/// bytecode命令がどこから生成されたかを表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionOrigin {
    /// ソースコード中の構文要素から生成された命令です。
    Source(Span),

    /// ソースコード上に対応箇所を持たない、コンパイラ生成の命令です。
    Synthetic,
}

#[derive(Debug, Clone)]
pub enum InstructionKind {
    PushI64(i64),
    PushF32(f32),
    PushF64(f64),

    Load(usize),
    Store(usize),

    Add(Type),
    Subtract(Type),
    Multiply(Type),
    Divide(Type),

    Negate(Type),

    Print(Type),

    Halt,
}

pub fn lower(program: &Program) -> BytecodeProgram {
    let mut slots = Vec::new();
    let mut slot_map = HashMap::new();

    for statement in &program.statements {
        if let StatementKind::Binding { name, ty, .. } = &statement.kind {
            let index = slots.len();

            slots.push(Slot {
                name: name.clone(),
                ty: (*ty).into(),
            });

            slot_map.insert(name.clone(), index);
        }
    }

    let mut compiler = Compiler {
        slot_map,
        instructions: Vec::new(),
    };

    for statement in &program.statements {
        compiler.emit_statement(statement);
    }

    compiler
        .instructions
        .push(Instruction::synthetic(InstructionKind::Halt));

    BytecodeProgram {
        slots,
        instructions: compiler.instructions,
    }
}

pub fn format_program(program: &BytecodeProgram) -> String {
    let mut output = String::new();

    writeln!(output, "; Primer bytecode v0.1").unwrap();

    if !program.slots.is_empty() {
        writeln!(output).unwrap();

        for (index, slot) in program.slots.iter().enumerate() {
            writeln!(output, ".slot {index} {} {}", slot.name, type_name(slot.ty),).unwrap();
        }
    }

    writeln!(output).unwrap();

    for (pc, instruction) in program.instructions.iter().enumerate() {
        write!(output, "{pc:04}  ").unwrap();

        format_instruction(&instruction.kind, program, &mut output);
    }

    output
}

struct Compiler {
    slot_map: HashMap<String, usize>,
    instructions: Vec<Instruction>,
}

impl Compiler {
    fn emit_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Binding { name, value, .. } => {
                self.emit_expr(value);

                let slot = self
                    .slot_map
                    .get(name)
                    .copied()
                    .expect("binding must have a bytecode slot");

                self.emit_source(InstructionKind::Store(slot), statement.span);
            }

            StatementKind::Print { value } => {
                self.emit_expr(value);

                self.emit_source(InstructionKind::Print(value.ty.into()), statement.span);
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Integer(value) => {
                self.emit_source(InstructionKind::PushI64(*value), expr.span);
            }

            ExprKind::Float { text } => match expr.ty {
                ir::Type::F32 => {
                    let value = text
                        .parse::<f32>()
                        .expect("validated floating-point literal");

                    self.emit_source(InstructionKind::PushF32(value), expr.span);
                }

                ir::Type::F64 => {
                    let value = text
                        .parse::<f64>()
                        .expect("validated floating-point literal");

                    self.emit_source(InstructionKind::PushF64(value), expr.span);
                }

                ir::Type::I64 => {
                    unreachable!("integer cannot be emitted as float");
                }
            },

            ExprKind::Variable(name) => {
                let slot = self
                    .slot_map
                    .get(name)
                    .copied()
                    .expect("variable must have a bytecode slot");

                self.emit_source(InstructionKind::Load(slot), expr.span);
            }

            ExprKind::Unary { op, value } => {
                self.emit_expr(value);

                match *op {
                    UnaryOp::Negate => {
                        self.emit_source(InstructionKind::Negate(expr.ty.into()), expr.span);
                    }
                }
            }

            ExprKind::Binary { op, left, right } => {
                self.emit_expr(left);
                self.emit_expr(right);

                let instruction = match *op {
                    BinaryOp::Add => InstructionKind::Add(expr.ty.into()),
                    BinaryOp::Subtract => InstructionKind::Subtract(expr.ty.into()),
                    BinaryOp::Multiply => InstructionKind::Multiply(expr.ty.into()),
                    BinaryOp::Divide => InstructionKind::Divide(expr.ty.into()),
                };

                self.emit_source(instruction, expr.span);
            }
        }
    }

    fn emit_source(&mut self, kind: InstructionKind, span: Span) {
        self.instructions.push(Instruction::source(kind, span));
    }
}

impl From<ir::Type> for Type {
    fn from(value: ir::Type) -> Self {
        match value {
            ir::Type::I64 => Self::I64,
            ir::Type::F32 => Self::F32,
            ir::Type::F64 => Self::F64,
        }
    }
}

fn format_instruction(
    instruction: &InstructionKind,
    program: &BytecodeProgram,
    output: &mut String,
) {
    match instruction {
        InstructionKind::PushI64(value) => {
            writeln!(output, "push.i64 {value}").unwrap();
        }

        InstructionKind::PushF32(value) => {
            writeln!(output, "push.f32 {value}").unwrap();
        }

        InstructionKind::PushF64(value) => {
            writeln!(output, "push.f64 {value}").unwrap();
        }

        InstructionKind::Load(slot) => {
            writeln!(output, "load {slot}        ; {}", program.slots[*slot].name,).unwrap();
        }

        InstructionKind::Store(slot) => {
            writeln!(output, "store {slot}       ; {}", program.slots[*slot].name,).unwrap();
        }

        InstructionKind::Add(ty) => {
            writeln!(output, "add.{}", type_name(*ty),).unwrap();
        }

        InstructionKind::Subtract(ty) => {
            writeln!(output, "sub.{}", type_name(*ty),).unwrap();
        }

        InstructionKind::Multiply(ty) => {
            writeln!(output, "mul.{}", type_name(*ty),).unwrap();
        }

        InstructionKind::Divide(ty) => {
            writeln!(output, "div.{}", type_name(*ty),).unwrap();
        }

        InstructionKind::Negate(ty) => {
            writeln!(output, "neg.{}", type_name(*ty),).unwrap();
        }

        InstructionKind::Print(ty) => {
            writeln!(output, "print.{}", type_name(*ty),).unwrap();
        }

        InstructionKind::Halt => {
            writeln!(output, "halt").unwrap();
        }
    }
}

fn type_name(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
    }
}

#[cfg(test)]
mod tests {
    use crate::{compile_to_bytecode, compile_to_ir, source::Span};

    use super::{BytecodeProgram, InstructionKind, InstructionOrigin, Type, format_program, lower};

    #[test]
    fn emits_typed_bytecode() {
        let program = compile_to_ir("x: f32 = 0.1 + 0.2; print(x);").unwrap();

        let bytecode = lower(&program);

        let text = format_program(&bytecode);

        assert!(text.contains("push.f32"));
        assert!(text.contains("add.f32"));
        assert!(text.contains("store 0"));
        assert!(text.contains("print.f32"));
        assert!(text.contains("halt"));
    }

    #[test]
    fn records_source_and_synthetic_instruction_origins() {
        let bytecode = compile_to_bytecode("print(1 / 0);").unwrap();

        assert!(matches!(
            bytecode.instructions[0].kind,
            InstructionKind::PushI64(1)
        ));
        assert!(matches!(
            bytecode.instructions[1].kind,
            InstructionKind::PushI64(0)
        ));
        assert!(matches!(
            bytecode.instructions[2].kind,
            InstructionKind::Divide(Type::I64)
        ));
        assert!(matches!(
            bytecode.instructions[3].kind,
            InstructionKind::Print(Type::I64)
        ));
        assert!(matches!(
            bytecode.instructions[4].kind,
            InstructionKind::Halt
        ));

        let origins: Vec<_> = bytecode
            .instructions
            .iter()
            .map(|instruction| instruction.origin)
            .collect();

        assert_eq!(
            origins,
            vec![
                InstructionOrigin::Source(Span::new(6, 7)),
                InstructionOrigin::Source(Span::new(10, 11)),
                InstructionOrigin::Source(Span::new(6, 11)),
                InstructionOrigin::Source(Span::new(0, 13)),
                InstructionOrigin::Synthetic,
            ]
        );
    }

    #[test]
    fn instruction_origins_are_deterministic() {
        let first = compile_to_bytecode("value: i64 = 1; print(value);").unwrap();
        let second = compile_to_bytecode("value: i64 = 1; print(value);").unwrap();

        let origins = |program: &BytecodeProgram| {
            program
                .instructions
                .iter()
                .map(|instruction| instruction.origin)
                .collect::<Vec<_>>()
        };

        assert_eq!(origins(&first), origins(&second));
    }
}
