use std::{collections::HashMap, fmt::Write};

use crate::ir::{self, BinaryOp, Expr, ExprKind, Program, Statement, StatementKind, UnaryOp};

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
pub enum Instruction {
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

    compiler.instructions.push(Instruction::Halt);

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

        format_instruction(instruction, program, &mut output);
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

                self.instructions.push(Instruction::Store(slot));
            }

            StatementKind::Print { value } => {
                self.emit_expr(value);

                self.instructions.push(Instruction::Print(value.ty.into()));
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Integer(value) => {
                self.instructions.push(Instruction::PushI64(*value));
            }

            ExprKind::Float { text } => match expr.ty {
                ir::Type::F32 => {
                    let value = text
                        .parse::<f32>()
                        .expect("validated floating-point literal");

                    self.instructions.push(Instruction::PushF32(value));
                }

                ir::Type::F64 => {
                    let value = text
                        .parse::<f64>()
                        .expect("validated floating-point literal");

                    self.instructions.push(Instruction::PushF64(value));
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

                self.instructions.push(Instruction::Load(slot));
            }

            ExprKind::Unary { op, value } => {
                self.emit_expr(value);

                match *op {
                    UnaryOp::Negate => {
                        self.instructions.push(Instruction::Negate(expr.ty.into()));
                    }
                }
            }

            ExprKind::Binary { op, left, right } => {
                self.emit_expr(left);
                self.emit_expr(right);

                let instruction = match *op {
                    BinaryOp::Add => Instruction::Add(expr.ty.into()),
                    BinaryOp::Subtract => Instruction::Subtract(expr.ty.into()),
                    BinaryOp::Multiply => Instruction::Multiply(expr.ty.into()),
                    BinaryOp::Divide => Instruction::Divide(expr.ty.into()),
                };

                self.instructions.push(instruction);
            }
        }
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

fn format_instruction(instruction: &Instruction, program: &BytecodeProgram, output: &mut String) {
    match instruction {
        Instruction::PushI64(value) => {
            writeln!(output, "push.i64 {value}").unwrap();
        }

        Instruction::PushF32(value) => {
            writeln!(output, "push.f32 {value}").unwrap();
        }

        Instruction::PushF64(value) => {
            writeln!(output, "push.f64 {value}").unwrap();
        }

        Instruction::Load(slot) => {
            writeln!(output, "load {slot}        ; {}", program.slots[*slot].name,).unwrap();
        }

        Instruction::Store(slot) => {
            writeln!(output, "store {slot}       ; {}", program.slots[*slot].name,).unwrap();
        }

        Instruction::Add(ty) => {
            writeln!(output, "add.{}", type_name(*ty),).unwrap();
        }

        Instruction::Subtract(ty) => {
            writeln!(output, "sub.{}", type_name(*ty),).unwrap();
        }

        Instruction::Multiply(ty) => {
            writeln!(output, "mul.{}", type_name(*ty),).unwrap();
        }

        Instruction::Divide(ty) => {
            writeln!(output, "div.{}", type_name(*ty),).unwrap();
        }

        Instruction::Negate(ty) => {
            writeln!(output, "neg.{}", type_name(*ty),).unwrap();
        }

        Instruction::Print(ty) => {
            writeln!(output, "print.{}", type_name(*ty),).unwrap();
        }

        Instruction::Halt => {
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
    use crate::compile_to_ir;

    use super::{format_program, lower};

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
}
