use std::{collections::HashMap, fmt::Write};

use crate::{
    diagnostic::Diagnostic,
    ir::{self, BinaryOp, BindingId, Expr, ExprKind, Program, Statement, StatementKind, UnaryOp},
    source::Span,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Bool,
    I64,
    F32,
    F64,
    Named(usize),
    Array {
        element: ArrayElementType,
        length: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrayElementType {
    Bool,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone)]
pub struct BytecodeProgram {
    pub type_definitions: Vec<TypeDefinition>,
    pub functions: Vec<BytecodeFunction>,
    pub slots: Vec<Slot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct BytecodeFunction {
    pub name: String,
    pub parameter_count: usize,
    pub return_type: ReturnType,
    pub slots: Vec<Slot>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnType {
    Void,
    Value(Type),
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone)]
pub struct FieldDefinition {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
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
    PushBool(bool),
    PushI64(i64),
    PushF32(f32),
    PushF64(f64),

    Load(usize),
    Store(usize),
    Assign(usize),
    Construct {
        type_id: usize,
        fields: Vec<ConstructField>,
    },
    FieldGet {
        type_id: usize,
        field_id: usize,
    },
    ConstructArray {
        element: ArrayElementType,
        length: usize,
    },
    Index {
        element: ArrayElementType,
        length: usize,
    },
    Call {
        function_id: usize,
        argument_count: usize,
    },
    Return {
        has_value: bool,
    },

    Add(Type),
    Subtract(Type),
    Multiply(Type),
    Divide(Type),

    Equal(Type),
    NotEqual(Type),
    Less(Type),
    LessEqual(Type),
    Greater(Type),
    GreaterEqual(Type),

    Negate(Type),
    Not,

    Print(Type),

    JumpIfFalse(usize),
    Jump(usize),

    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstructField {
    pub field_id: usize,
    pub origin: FieldOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldOrigin {
    Explicit,
    Default,
}

pub fn lower(program: &Program) -> Result<BytecodeProgram, Diagnostic> {
    let type_definitions = program
        .type_definitions
        .iter()
        .map(|definition| TypeDefinition {
            name: definition.name.clone(),
            fields: definition
                .fields
                .iter()
                .map(|field| FieldDefinition {
                    name: field.name.clone(),
                    ty: field.ty.into(),
                })
                .collect(),
        })
        .collect();
    let functions = program
        .function_definitions
        .iter()
        .map(|function| {
            let mut slots = Vec::new();
            let mut slot_map = HashMap::new();
            for parameter in &function.parameters {
                let slot = slots.len();
                slots.push(Slot {
                    name: parameter.name.clone(),
                    ty: parameter.ty.into(),
                    mutable: false,
                });
                slot_map.insert(parameter.id, slot);
            }
            collect_slots(&function.body, &mut slots, &mut slot_map);
            let mut compiler = Compiler {
                slot_map,
                instructions: Vec::new(),
                loops: Vec::new(),
            };
            let terminates = compiler.emit_statements(&function.body);
            if !terminates {
                compiler
                    .instructions
                    .push(Instruction::synthetic(InstructionKind::Return {
                        has_value: false,
                    }));
            }
            BytecodeFunction {
                name: function.name.clone(),
                parameter_count: function.parameters.len(),
                return_type: match function.return_type {
                    ir::ReturnType::Void => ReturnType::Void,
                    ir::ReturnType::Value(ty) => ReturnType::Value(ty.into()),
                },
                slots,
                instructions: compiler.instructions,
            }
        })
        .collect();

    let mut slots = Vec::new();
    let mut slot_map = HashMap::new();

    collect_slots(&program.statements, &mut slots, &mut slot_map);

    let mut compiler = Compiler {
        slot_map,
        instructions: Vec::new(),
        loops: Vec::new(),
    };

    compiler.emit_statements(&program.statements);

    compiler
        .instructions
        .push(Instruction::synthetic(InstructionKind::Halt));

    Ok(BytecodeProgram {
        type_definitions,
        functions,
        slots,
        instructions: compiler.instructions,
    })
}

pub fn format_program(program: &BytecodeProgram) -> String {
    let mut output = String::new();

    writeln!(output, "; Primer bytecode v0.1").unwrap();

    for (type_id, definition) in program.type_definitions.iter().enumerate() {
        writeln!(output, "\n.type {type_id} {}", definition.name).unwrap();
        for (field_id, field) in definition.fields.iter().enumerate() {
            writeln!(
                output,
                ".field {type_id}.{field_id} {} {}",
                field.name,
                type_name(field.ty, program)
            )
            .unwrap();
        }
    }

    for (function_id, function) in program.functions.iter().enumerate() {
        writeln!(output).unwrap();
        write!(output, ".function {function_id} {}(", function.name).unwrap();
        for parameter_index in 0..function.parameter_count {
            if parameter_index > 0 {
                output.push_str(", ");
            }
            let parameter = &function.slots[parameter_index];
            write!(
                output,
                "{parameter_index}:{}:{}",
                parameter.name,
                type_name(parameter.ty, program)
            )
            .unwrap();
        }
        match function.return_type {
            ReturnType::Void => writeln!(output, ") -> void").unwrap(),
            ReturnType::Value(ty) => {
                writeln!(output, ") -> {}", type_name(ty, program)).unwrap();
            }
        }

        for (index, slot) in function.slots.iter().enumerate() {
            write!(output, ".slot {index} ").unwrap();
            if slot.mutable {
                output.push_str("mut ");
            }
            writeln!(output, "{} {}", slot.name, type_name(slot.ty, program)).unwrap();
        }

        for (pc, instruction) in function.instructions.iter().enumerate() {
            write!(output, "{pc:04}  ").unwrap();
            format_instruction(&instruction.kind, program, &function.slots, &mut output);
        }
        writeln!(output, ".end").unwrap();
    }

    if !program.slots.is_empty() {
        writeln!(output).unwrap();

        for (index, slot) in program.slots.iter().enumerate() {
            write!(output, ".slot {index} ").unwrap();

            if slot.mutable {
                output.push_str("mut ");
            }

            writeln!(output, "{} {}", slot.name, type_name(slot.ty, program)).unwrap();
        }
    }

    writeln!(output).unwrap();

    for (pc, instruction) in program.instructions.iter().enumerate() {
        write!(output, "{pc:04}  ").unwrap();

        format_instruction(&instruction.kind, program, &program.slots, &mut output);
    }

    output
}

struct Compiler {
    slot_map: HashMap<BindingId, usize>,
    instructions: Vec<Instruction>,
    loops: Vec<LoopContext>,
}

struct LoopContext {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
}

impl Compiler {
    fn emit_statements(&mut self, statements: &[Statement]) -> bool {
        for statement in statements {
            if self.emit_statement(statement) {
                return true;
            }
        }

        false
    }

    fn emit_statement(&mut self, statement: &Statement) -> bool {
        match &statement.kind {
            StatementKind::Binding { id, value, .. } => {
                self.emit_expr(value);

                let slot = self
                    .slot_map
                    .get(id)
                    .copied()
                    .expect("binding must have a bytecode slot");

                self.emit_source(InstructionKind::Store(slot), statement.span);
                false
            }

            StatementKind::Assignment { id, value, .. } => {
                self.emit_expr(value);

                let slot = self
                    .slot_map
                    .get(id)
                    .copied()
                    .expect("assignment target must have a bytecode slot");

                self.emit_source(InstructionKind::Assign(slot), statement.span);
                false
            }

            StatementKind::Print { value } => {
                self.emit_expr(value);

                self.emit_source(InstructionKind::Print(value.ty.into()), statement.span);
                false
            }

            StatementKind::Call {
                function_id,
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.emit_expr(argument);
                }
                self.emit_source(
                    InstructionKind::Call {
                        function_id: function_id.0,
                        argument_count: arguments.len(),
                    },
                    statement.span,
                );
                false
            }

            StatementKind::Return { value } => {
                if let Some(value) = value {
                    self.emit_expr(value);
                }
                self.emit_source(
                    InstructionKind::Return {
                        has_value: value.is_some(),
                    },
                    statement.span,
                );
                true
            }

            StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                self.emit_expr(condition);
                let false_jump = self.instructions.len();
                self.emit_source(InstructionKind::JumpIfFalse(usize::MAX), condition.span);

                let then_terminates = self.emit_statements(then_body);

                if else_body.is_empty() {
                    let end = self.instructions.len();
                    self.patch_jump(false_jump, end);
                    false
                } else {
                    let end_jump = if then_terminates {
                        None
                    } else {
                        let index = self.instructions.len();
                        self.emit_source(InstructionKind::Jump(usize::MAX), statement.span);
                        Some(index)
                    };
                    let else_start = self.instructions.len();
                    self.patch_jump(false_jump, else_start);

                    let else_terminates = self.emit_statements(else_body);

                    let end = self.instructions.len();
                    if let Some(end_jump) = end_jump {
                        self.patch_jump(end_jump, end);
                    }

                    then_terminates && else_terminates
                }
            }

            StatementKind::While { condition, body } => {
                let condition_start = self.instructions.len();
                self.emit_expr(condition);
                let end_jump = self.instructions.len();
                self.emit_source(InstructionKind::JumpIfFalse(usize::MAX), condition.span);

                self.loops.push(LoopContext {
                    break_jumps: Vec::new(),
                    continue_jumps: Vec::new(),
                });
                let body_terminates = self.emit_statements(body);
                let loop_context = self.loops.pop().expect("while loop context must exist");

                if !body_terminates {
                    self.emit_source(InstructionKind::Jump(condition_start), statement.span);
                }
                for continue_jump in loop_context.continue_jumps {
                    self.patch_jump(continue_jump, condition_start);
                }
                let end = self.instructions.len();
                self.patch_jump(end_jump, end);
                for break_jump in loop_context.break_jumps {
                    self.patch_jump(break_jump, end);
                }

                false
            }

            StatementKind::For {
                initializer,
                condition,
                update,
                body,
            } => {
                self.emit_statement(initializer);

                let condition_start = self.instructions.len();
                self.emit_expr(condition);
                let end_jump = self.instructions.len();
                self.emit_source(InstructionKind::JumpIfFalse(usize::MAX), condition.span);

                self.loops.push(LoopContext {
                    break_jumps: Vec::new(),
                    continue_jumps: Vec::new(),
                });
                self.emit_statements(body);
                let loop_context = self.loops.pop().expect("for loop context must exist");

                let update_start = self.instructions.len();
                for continue_jump in loop_context.continue_jumps {
                    self.patch_jump(continue_jump, update_start);
                }
                self.emit_statement(update);
                self.emit_source(InstructionKind::Jump(condition_start), statement.span);

                let end = self.instructions.len();
                self.patch_jump(end_jump, end);
                for break_jump in loop_context.break_jumps {
                    self.patch_jump(break_jump, end);
                }

                false
            }

            StatementKind::Break => {
                let jump = self.instructions.len();
                self.emit_source(InstructionKind::Jump(usize::MAX), statement.span);
                self.loops
                    .last_mut()
                    .expect("semantic analysis rejects break outside a loop")
                    .break_jumps
                    .push(jump);
                true
            }

            StatementKind::Continue => {
                let jump = self.instructions.len();
                self.emit_source(InstructionKind::Jump(usize::MAX), statement.span);
                self.loops
                    .last_mut()
                    .expect("semantic analysis rejects continue outside a loop")
                    .continue_jumps
                    .push(jump);
                true
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Boolean(value) => {
                self.emit_source(InstructionKind::PushBool(*value), expr.span);
            }

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

                ir::Type::Bool => {
                    unreachable!("boolean cannot be emitted as float");
                }
                ir::Type::Named(_) | ir::Type::Array { .. } => {
                    unreachable!("a float literal cannot have an aggregate type");
                }
            },

            ExprKind::Variable { id, .. } => {
                let slot = self
                    .slot_map
                    .get(id)
                    .copied()
                    .expect("variable must have a bytecode slot");

                self.emit_source(InstructionKind::Load(slot), expr.span);
            }

            ExprKind::Construct {
                type_id, fields, ..
            } => {
                for field in fields {
                    self.emit_expr(&field.value);
                }
                self.emit_source(
                    InstructionKind::Construct {
                        type_id: type_id.0,
                        fields: fields
                            .iter()
                            .map(|field| ConstructField {
                                field_id: field.id.0,
                                origin: match field.origin {
                                    ir::FieldValueOrigin::Explicit { .. } => FieldOrigin::Explicit,
                                    ir::FieldValueOrigin::Default { .. } => FieldOrigin::Default,
                                },
                            })
                            .collect(),
                    },
                    expr.span,
                );
            }

            ExprKind::FieldAccess {
                type_id,
                field_id,
                base,
                ..
            } => {
                self.emit_expr(base);
                self.emit_source(
                    InstructionKind::FieldGet {
                        type_id: type_id.0,
                        field_id: field_id.0,
                    },
                    expr.span,
                );
            }

            ExprKind::Array(values) => {
                for value in values {
                    self.emit_expr(value);
                }
                let ir::Type::Array { element, length } = expr.ty else {
                    unreachable!("array expressions must have an array type")
                };
                self.emit_source(
                    InstructionKind::ConstructArray {
                        element: element.into(),
                        length,
                    },
                    expr.span,
                );
            }

            ExprKind::Index { base, index } => {
                self.emit_expr(base);
                self.emit_expr(index);
                let ir::Type::Array { element, length } = base.ty else {
                    unreachable!("indexed expressions must have an array base")
                };
                self.emit_source(
                    InstructionKind::Index {
                        element: element.into(),
                        length,
                    },
                    expr.span,
                );
            }

            ExprKind::Call {
                function_id,
                arguments,
                ..
            } => {
                for argument in arguments {
                    self.emit_expr(argument);
                }
                self.emit_source(
                    InstructionKind::Call {
                        function_id: function_id.0,
                        argument_count: arguments.len(),
                    },
                    expr.span,
                );
            }

            ExprKind::Unary { op, value } => {
                self.emit_expr(value);

                match *op {
                    UnaryOp::Negate => {
                        self.emit_source(InstructionKind::Negate(expr.ty.into()), expr.span);
                    }
                    UnaryOp::Not => {
                        self.emit_source(InstructionKind::Not, expr.span);
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
                    BinaryOp::Equal => InstructionKind::Equal(left.ty.into()),
                    BinaryOp::NotEqual => InstructionKind::NotEqual(left.ty.into()),
                    BinaryOp::Less => InstructionKind::Less(left.ty.into()),
                    BinaryOp::LessEqual => InstructionKind::LessEqual(left.ty.into()),
                    BinaryOp::Greater => InstructionKind::Greater(left.ty.into()),
                    BinaryOp::GreaterEqual => InstructionKind::GreaterEqual(left.ty.into()),
                };

                self.emit_source(instruction, expr.span);
            }
        }
    }

    fn emit_source(&mut self, kind: InstructionKind, span: Span) {
        self.instructions.push(Instruction::source(kind, span));
    }

    fn patch_jump(&mut self, index: usize, target: usize) {
        match &mut self.instructions[index].kind {
            InstructionKind::JumpIfFalse(current) | InstructionKind::Jump(current) => {
                *current = target;
            }
            _ => unreachable!("only jump instructions are patched"),
        }
    }
}

fn collect_slots(
    statements: &[Statement],
    slots: &mut Vec<Slot>,
    slot_map: &mut HashMap<BindingId, usize>,
) {
    for statement in statements {
        match &statement.kind {
            StatementKind::Binding {
                id,
                mutable,
                name,
                ty,
                ..
            } => {
                let index = slots.len();
                slots.push(Slot {
                    name: name.clone(),
                    ty: (*ty).into(),
                    mutable: *mutable,
                });
                slot_map.insert(*id, index);
            }
            StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                collect_slots(then_body, slots, slot_map);
                collect_slots(else_body, slots, slot_map);
            }
            StatementKind::While { body, .. } => {
                collect_slots(body, slots, slot_map);
            }
            StatementKind::For {
                initializer, body, ..
            } => {
                collect_slots(std::slice::from_ref(initializer), slots, slot_map);
                collect_slots(body, slots, slot_map);
            }
            StatementKind::Assignment { .. }
            | StatementKind::Print { .. }
            | StatementKind::Call { .. }
            | StatementKind::Return { .. }
            | StatementKind::Break
            | StatementKind::Continue => {}
        }
    }
}

impl From<ir::Type> for Type {
    fn from(value: ir::Type) -> Self {
        match value {
            ir::Type::Bool => Self::Bool,
            ir::Type::I64 => Self::I64,
            ir::Type::F32 => Self::F32,
            ir::Type::F64 => Self::F64,
            ir::Type::Named(id) => Self::Named(id.0),
            ir::Type::Array { element, length } => Self::Array {
                element: element.into(),
                length,
            },
        }
    }
}

impl From<ir::ArrayElementType> for ArrayElementType {
    fn from(value: ir::ArrayElementType) -> Self {
        match value {
            ir::ArrayElementType::Bool => Self::Bool,
            ir::ArrayElementType::I64 => Self::I64,
            ir::ArrayElementType::F32 => Self::F32,
            ir::ArrayElementType::F64 => Self::F64,
        }
    }
}

fn format_instruction(
    instruction: &InstructionKind,
    program: &BytecodeProgram,
    slots: &[Slot],
    output: &mut String,
) {
    match instruction {
        InstructionKind::PushBool(value) => {
            writeln!(output, "push.bool {value}").unwrap();
        }

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
            writeln!(output, "load {slot}        ; {}", slots[*slot].name).unwrap();
        }

        InstructionKind::Store(slot) => {
            writeln!(output, "store {slot}       ; {}", slots[*slot].name).unwrap();
        }

        InstructionKind::Assign(slot) => {
            writeln!(output, "assign {slot}      ; {}", slots[*slot].name).unwrap();
        }

        InstructionKind::Construct { type_id, fields } => {
            write!(
                output,
                "construct {} [",
                type_name(Type::Named(*type_id), program)
            )
            .unwrap();
            for (index, field) in fields.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                let definition = &program.type_definitions[*type_id];
                let origin = match field.origin {
                    FieldOrigin::Explicit => "explicit",
                    FieldOrigin::Default => "default",
                };
                write!(
                    output,
                    "%{}@{}:{origin}",
                    definition.fields[field.field_id].name, field.field_id
                )
                .unwrap();
            }
            writeln!(output, "]").unwrap();
        }

        InstructionKind::FieldGet { type_id, field_id } => {
            let definition = &program.type_definitions[*type_id];
            writeln!(
                output,
                "field.get %{}@{}.%{}@{}",
                definition.name, type_id, definition.fields[*field_id].name, field_id
            )
            .unwrap();
        }

        InstructionKind::ConstructArray { element, length } => {
            writeln!(
                output,
                "array.new {} {length}",
                array_element_name(*element)
            )
            .unwrap();
        }

        InstructionKind::Index { element, length } => {
            writeln!(
                output,
                "array.get {} {length}",
                array_element_name(*element)
            )
            .unwrap();
        }

        InstructionKind::Call {
            function_id,
            argument_count,
        } => {
            writeln!(
                output,
                "call {function_id} {argument_count}  ; {}",
                program.functions[*function_id].name
            )
            .unwrap();
        }

        InstructionKind::Return { has_value: true } => {
            writeln!(output, "return.value").unwrap();
        }

        InstructionKind::Return { has_value: false } => {
            writeln!(output, "return").unwrap();
        }

        InstructionKind::Add(ty) => {
            writeln!(output, "add.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::Subtract(ty) => {
            writeln!(output, "sub.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::Multiply(ty) => {
            writeln!(output, "mul.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::Divide(ty) => {
            writeln!(output, "div.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::Equal(ty) => {
            writeln!(output, "eq.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::NotEqual(ty) => {
            writeln!(output, "ne.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::Less(ty) => {
            writeln!(output, "lt.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::LessEqual(ty) => {
            writeln!(output, "le.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::Greater(ty) => {
            writeln!(output, "gt.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::GreaterEqual(ty) => {
            writeln!(output, "ge.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::Negate(ty) => {
            writeln!(output, "neg.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::Not => {
            writeln!(output, "not.bool").unwrap();
        }

        InstructionKind::Print(ty) => {
            writeln!(output, "print.{}", type_name(*ty, program),).unwrap();
        }

        InstructionKind::JumpIfFalse(target) => {
            writeln!(output, "jump-if-false {target:04}").unwrap();
        }

        InstructionKind::Jump(target) => {
            writeln!(output, "jump {target:04}").unwrap();
        }

        InstructionKind::Halt => {
            writeln!(output, "halt").unwrap();
        }
    }
}

fn type_name(ty: Type, program: &BytecodeProgram) -> String {
    match ty {
        Type::Bool => "bool".into(),
        Type::I64 => "i64".into(),
        Type::F32 => "f32".into(),
        Type::F64 => "f64".into(),
        Type::Named(id) => format!("%{}@{id}", program.type_definitions[id].name),
        Type::Array { element, length } => {
            format!("[{}; {length}]", array_element_name(element))
        }
    }
}

const fn array_element_name(element: ArrayElementType) -> &'static str {
    match element {
        ArrayElementType::Bool => "bool",
        ArrayElementType::I64 => "i64",
        ArrayElementType::F32 => "f32",
        ArrayElementType::F64 => "f64",
    }
}

#[cfg(test)]
mod tests {
    use crate::{compile_to_bytecode, compile_to_ir, source::Span};

    use super::{BytecodeProgram, InstructionKind, InstructionOrigin, Type, format_program, lower};

    #[test]
    fn emits_typed_bytecode() {
        let program = compile_to_ir("x: f32 = 0.1 + 0.2; print(x);").unwrap();

        let bytecode = lower(&program).unwrap();

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

    #[test]
    fn emits_all_typed_comparisons() {
        let program = compile_to_bytecode(
            "a: bool = 1 == 1; b: bool = 1 != 2; c: bool = 1 < 2;
             d: bool = 1 <= 2; e: bool = 2 > 1; f: bool = 2 >= 1;",
        )
        .unwrap();
        let text = format_program(&program);

        for instruction in ["eq.i64", "ne.i64", "lt.i64", "le.i64", "gt.i64", "ge.i64"] {
            assert!(text.contains(instruction));
        }
    }

    #[test]
    fn exposes_function_frames_calls_and_returns() {
        let program = compile_to_bytecode(
            "fn add(left: i64, right: i64) -> i64 { return left + right; }
             answer: i64 = add(20, 22);",
        )
        .unwrap();
        let text = format_program(&program);

        assert!(text.contains(".function 0 add(0:left:i64, 1:right:i64) -> i64"));
        assert!(text.contains("0003  return.value"));
        assert!(text.contains("call 0 2  ; add"));
    }
}
