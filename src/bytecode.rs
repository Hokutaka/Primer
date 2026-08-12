use std::{collections::HashMap, fmt::Write};

use crate::ast::{BinaryOp, Expr, Program, Stmt, Type, UnaryOp};
use crate::semantic::{Bindings, type_of_expr};

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

pub fn compile(program: &Program, bindings: &Bindings) -> BytecodeProgram {
    let mut slots = Vec::new();
    let mut slot_map = HashMap::new();

    for statement in &program.statements {
        if let Stmt::Binding { name, .. } = statement {
            let ty = bindings
                .get(name)
                .copied()
                .expect("binding must have been resolved by type checker");

            let index = slots.len();

            slots.push(Slot {
                name: name.clone(),
                ty,
            });

            slot_map.insert(name.clone(), index);
        }
    }

    let mut compiler = Compiler {
        bindings,
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

struct Compiler<'a> {
    bindings: &'a Bindings,
    slot_map: HashMap<String, usize>,
    instructions: Vec<Instruction>,
}

impl Compiler<'_> {
    fn emit_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Binding { name, value, .. } => {
                let ty = self
                    .bindings
                    .get(name)
                    .copied()
                    .expect("binding must have been resolved by type checker");

                self.emit_expr(value, Some(ty));

                let slot = self
                    .slot_map
                    .get(name)
                    .copied()
                    .expect("binding must have a bytecode slot");

                self.instructions.push(Instruction::Store(slot));
            }

            Stmt::Print { value } => {
                let ty =
                    type_of_expr(value, self.bindings).expect("expression must have been checked");

                self.emit_expr(value, Some(ty));

                self.instructions.push(Instruction::Print(ty));
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr, expected: Option<Type>) {
        match expr {
            Expr::Integer(value) => {
                self.instructions.push(Instruction::PushI64(*value));
            }

            Expr::Float {
                text,
                explicit_type,
            } => {
                let ty = match explicit_type {
                    Some(ty) => *ty,

                    None => match expected {
                        Some(Type::F32) => Type::F32,

                        _ => Type::F64,
                    },
                };

                match ty {
                    Type::F32 => {
                        let value = text
                            .parse::<f32>()
                            .expect("validated floating-point literal");

                        self.instructions.push(Instruction::PushF32(value));
                    }

                    Type::F64 => {
                        let value = text
                            .parse::<f64>()
                            .expect("validated floating-point literal");

                        self.instructions.push(Instruction::PushF64(value));
                    }

                    Type::I64 => {
                        unreachable!("integer cannot be emitted as float");
                    }
                }
            }

            Expr::Variable(name) => {
                let slot = self
                    .slot_map
                    .get(name)
                    .copied()
                    .expect("variable must have a bytecode slot");

                self.instructions.push(Instruction::Load(slot));
            }

            Expr::Unary { op, value } => {
                let ty = expected.unwrap_or_else(|| {
                    type_of_expr(expr, self.bindings).expect("expression must have been checked")
                });

                self.emit_expr(value, Some(ty));

                match op {
                    UnaryOp::Negate => {
                        self.instructions.push(Instruction::Negate(ty));
                    }
                }
            }

            Expr::Binary { op, left, right } => {
                let ty = expected.unwrap_or_else(|| {
                    type_of_expr(expr, self.bindings).expect("expression must have been checked")
                });

                self.emit_expr(left, Some(ty));

                self.emit_expr(right, Some(ty));

                let instruction = match op {
                    BinaryOp::Add => Instruction::Add(ty),

                    BinaryOp::Subtract => Instruction::Subtract(ty),

                    BinaryOp::Multiply => Instruction::Multiply(ty),

                    BinaryOp::Divide => Instruction::Divide(ty),
                };

                self.instructions.push(instruction);
            }
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
    use crate::{lexer::lex, parser::parse, semantic::check};

    use super::{compile, format_program};

    #[test]
    fn emits_typed_bytecode() {
        let program = parse(
            lex("x: f32 = 0.1 + 0.2;
                 print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let bytecode = compile(&program, &bindings);

        let text = format_program(&bytecode);

        assert!(text.contains("push.f32"));

        assert!(text.contains("add.f32"));

        assert!(text.contains("store 0"));

        assert!(text.contains("print.f32"));

        assert!(text.contains("halt"));
    }
}
