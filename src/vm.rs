pub mod render;

use crate::bytecode::{ArrayAccess, BytecodeProgram, InstructionKind, ReturnType, Type};
use crate::types::IntegerType;

/// 整数の桁あふれを起こした演算を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerOperation {
    Add,
    Subtract,
    Multiply,
    Negate,
}

/// Primer VMの実行中に発生した問題の種類を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmErrorKind {
    /// 明示変換の入力を変換先の範囲で表せませんでした。
    IntegerConversionOutOfRange {
        from: IntegerType,
        to: IntegerType,
    },
    /// bytecodeが整数型の範囲外の値を持っていました。
    InvalidIntegerValue {
        ty: IntegerType,
    },
    InvalidNegationType {
        ty: IntegerType,
    },
    /// 実行位置がbytecodeの範囲外へ到達しました。
    InstructionOutOfBounds,

    /// 存在しないスロットへアクセスしました。
    InvalidSlot {
        slot: usize,
    },

    /// 存在しないproduct typeを使おうとしました。
    InvalidType {
        type_id: usize,
    },

    /// product typeに存在しないfieldを使おうとしました。
    InvalidField {
        type_id: usize,
        field_id: usize,
    },

    /// 存在しない関数を呼び出そうとしました。
    InvalidFunction {
        function_id: usize,
    },

    /// 関数へ渡された引数の数が定義と一致しませんでした。
    InvalidArgumentCount {
        expected: usize,
        actual: usize,
    },

    /// 関数の定義と一致しない方法で値を返そうとしました。
    InvalidReturn,

    /// 初期化前のスロットを読み取りました。
    UninitializedSlot {
        slot: usize,
    },

    /// 別の初期化命令が初期化済みのスロットへ書き込みました。
    SlotAlreadyInitialized {
        slot: usize,
    },

    /// 初期化前のスロットへ再代入しようとしました。
    AssignmentToUninitializedSlot {
        slot: usize,
    },

    /// 不変のスロットへ再代入しようとしました。
    AssignmentToImmutableSlot {
        slot: usize,
    },

    /// 必要な値がスタックにありませんでした。
    StackUnderflow,

    /// 命令が期待した型と実際の値の型が一致しませんでした。
    TypeMismatch {
        expected: Type,
        actual: Type,
    },

    /// その型では利用できない比較命令を実行しようとしました。
    InvalidComparisonType {
        ty: Type,
    },

    /// 整数をゼロで除算しようとしました。
    DivisionByZero,

    /// 整数除算の結果を指定された型で表現できませんでした。
    DivisionOverflow,

    /// 整数演算の結果を指定された型で表現できませんでした。
    IntegerOverflow {
        operation: IntegerOperation,
        ty: Type,
    },

    /// 配列の外側を読み取ろうとしました。
    ArrayIndexOutOfBounds {
        index: i64,
        length: usize,
    },

    /// VM停止時に未使用の値がスタックへ残っていました。
    UnusedStackValues {
        count: usize,
    },
}

/// Primer VMの実行エラーと発生したbytecode命令位置を表します。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmError {
    kind: VmErrorKind,
    instruction_index: usize,
    function_id: Option<usize>,
}

impl VmError {
    const fn new(kind: VmErrorKind, instruction_index: usize) -> Self {
        Self {
            kind,
            instruction_index,
            function_id: None,
        }
    }

    const fn in_function(mut self, function_id: usize) -> Self {
        if self.function_id.is_none() {
            self.function_id = Some(function_id);
        }
        self
    }

    /// エラーの種類を返します。
    pub fn kind(&self) -> VmErrorKind {
        self.kind.clone()
    }

    /// `emit-bytecode`の表示と対応する0から始まる命令番号を返します。
    pub const fn instruction_index(&self) -> usize {
        self.instruction_index
    }

    /// エラーが関数内で起きた場合、その関数番号を返します。
    pub const fn function_id(&self) -> Option<usize> {
        self.function_id
    }
}

#[derive(Debug, Clone)]
enum Value {
    Bool(bool),
    Integer(i64, IntegerType),
    F32(f32),
    F64(f64),
    Aggregate { type_id: usize, fields: Vec<Value> },
    Array { element: Type, values: Vec<Value> },
}

impl Value {
    fn ty(&self) -> Type {
        match self {
            Self::Bool(_) => Type::Bool,
            Self::Integer(_, ty) => Type::Integer(*ty),
            Self::F32(_) => Type::F32,
            Self::F64(_) => Type::F64,
            Self::Aggregate { type_id, .. } => Type::Named(*type_id),
            Self::Array { element, values } => Type::Array {
                element: Box::new(element.clone()),
                length: values.len(),
            },
        }
    }
}

type VmResult<T> = Result<T, VmErrorKind>;

fn at_instruction<T>(result: VmResult<T>, instruction_index: usize) -> Result<T, VmError> {
    result.map_err(|kind| VmError::new(kind, instruction_index))
}

#[derive(Clone, Copy)]
enum Frame {
    Entry,
    Function(usize),
}

pub fn run(program: &BytecodeProgram) -> Result<String, VmError> {
    let mut output = String::new();
    execute_frame(program, Frame::Entry, Vec::new(), &mut output)?;
    Ok(output)
}

fn execute_frame(
    program: &BytecodeProgram,
    frame: Frame,
    arguments: Vec<Value>,
    output: &mut String,
) -> Result<Option<Value>, VmError> {
    let result = execute_frame_inner(program, frame, arguments, output);
    match (frame, result) {
        (Frame::Function(function_id), Err(error)) => Err(error.in_function(function_id)),
        (_, result) => result,
    }
}

fn execute_frame_inner(
    program: &BytecodeProgram,
    frame: Frame,
    arguments: Vec<Value>,
    output: &mut String,
) -> Result<Option<Value>, VmError> {
    let (instructions, slot_info, parameter_count, return_type) = match frame {
        Frame::Entry => (
            program.instructions.as_slice(),
            program.slots.as_slice(),
            0,
            ReturnType::Void,
        ),
        Frame::Function(function_id) => {
            let function = program
                .functions
                .get(function_id)
                .ok_or_else(|| VmError::new(VmErrorKind::InvalidFunction { function_id }, 0))?;
            (
                function.instructions.as_slice(),
                function.slots.as_slice(),
                function.parameter_count,
                function.return_type.clone(),
            )
        }
    };

    if arguments.len() != parameter_count {
        return Err(VmError::new(
            VmErrorKind::InvalidArgumentCount {
                expected: parameter_count,
                actual: arguments.len(),
            },
            0,
        ));
    }

    let mut stack: Vec<Value> = Vec::new();
    let mut slots: Vec<Option<Value>> = vec![None; slot_info.len()];
    let mut initializers: Vec<Option<usize>> = vec![None; slot_info.len()];

    for (index, argument) in arguments.into_iter().enumerate() {
        let expected = slot_info[index].ty.clone();
        if argument.ty() != expected {
            return Err(VmError::new(
                VmErrorKind::TypeMismatch {
                    expected,
                    actual: argument.ty(),
                },
                0,
            ));
        }
        slots[index] = Some(argument);
        initializers[index] = Some(usize::MAX);
    }

    let mut pc = 0;

    loop {
        let instruction = instructions
            .get(pc)
            .ok_or_else(|| VmError::new(VmErrorKind::InstructionOutOfBounds, pc))?;

        match &instruction.kind {
            InstructionKind::ConvertInteger { from, to } => {
                let value = at_instruction(pop_integer(&mut stack, *from), pc)?;
                if !to.contains(value) {
                    return Err(VmError::new(
                        VmErrorKind::IntegerConversionOutOfRange {
                            from: *from,
                            to: *to,
                        },
                        pc,
                    ));
                }
                stack.push(Value::Integer(value, *to));
            }
            InstructionKind::PushBool(value) => {
                stack.push(Value::Bool(*value));
            }

            InstructionKind::PushInteger(value, ty) => {
                if !ty.contains(*value) {
                    return Err(VmError::new(
                        VmErrorKind::InvalidIntegerValue { ty: *ty },
                        pc,
                    ));
                }
                stack.push(Value::Integer(*value, *ty));
            }

            InstructionKind::PushF32(value) => {
                stack.push(Value::F32(*value));
            }

            InstructionKind::PushF64(value) => {
                stack.push(Value::F64(*value));
            }

            InstructionKind::Load(slot) => {
                let value = slots
                    .get(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?
                    .clone()
                    .ok_or_else(|| {
                        VmError::new(VmErrorKind::UninitializedSlot { slot: *slot }, pc)
                    })?;

                stack.push(value);
            }

            InstructionKind::Store(slot) => {
                let value = at_instruction(pop_value(&mut stack), pc)?;

                let expected = slot_info
                    .get(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?
                    .ty
                    .clone();

                if value.ty() != expected {
                    return Err(VmError::new(
                        VmErrorKind::TypeMismatch {
                            expected,
                            actual: value.ty(),
                        },
                        pc,
                    ));
                }

                let destination = slots
                    .get_mut(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?;

                let initializer = initializers
                    .get_mut(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?;

                if initializer.is_some_and(|initializer| initializer != pc) {
                    return Err(VmError::new(
                        VmErrorKind::SlotAlreadyInitialized { slot: *slot },
                        pc,
                    ));
                }

                *initializer = Some(pc);
                *destination = Some(value);
            }

            InstructionKind::Assign(slot) => {
                let value = at_instruction(pop_value(&mut stack), pc)?;

                let slot_definition = slot_info
                    .get(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?;

                if !slot_definition.mutable {
                    return Err(VmError::new(
                        VmErrorKind::AssignmentToImmutableSlot { slot: *slot },
                        pc,
                    ));
                }

                let expected = slot_definition.ty.clone();

                if value.ty() != expected {
                    return Err(VmError::new(
                        VmErrorKind::TypeMismatch {
                            expected,
                            actual: value.ty(),
                        },
                        pc,
                    ));
                }

                let destination = slots
                    .get_mut(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?;

                if destination.is_none() {
                    return Err(VmError::new(
                        VmErrorKind::AssignmentToUninitializedSlot { slot: *slot },
                        pc,
                    ));
                }

                *destination = Some(value);
            }

            InstructionKind::ArrayAssign { slot, path } => {
                let replacement = at_instruction(pop_value(&mut stack), pc)?;
                let mut indices = Vec::with_capacity(path.len());
                for _ in 0..path.len() {
                    indices.push(at_instruction(pop_i64(&mut stack), pc)?);
                }
                indices.reverse();

                let slot_definition = slot_info
                    .get(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?;
                if !slot_definition.mutable {
                    return Err(VmError::new(
                        VmErrorKind::AssignmentToImmutableSlot { slot: *slot },
                        pc,
                    ));
                }

                let destination = slots
                    .get_mut(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?;
                let Some(current) = destination else {
                    return Err(VmError::new(
                        VmErrorKind::AssignmentToUninitializedSlot { slot: *slot },
                        pc,
                    ));
                };

                let mut updated = current.clone();
                at_instruction(
                    assign_array_path(&mut updated, path, &indices, replacement),
                    pc,
                )?;
                *current = updated;
            }

            InstructionKind::ArrayCheck { slot, path } => {
                let slot_definition = slot_info
                    .get(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?;
                if !slot_definition.mutable {
                    return Err(VmError::new(
                        VmErrorKind::AssignmentToImmutableSlot { slot: *slot },
                        pc,
                    ));
                }
                let current = slots
                    .get(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?
                    .as_ref()
                    .ok_or_else(|| {
                        VmError::new(
                            VmErrorKind::AssignmentToUninitializedSlot { slot: *slot },
                            pc,
                        )
                    })?;
                if stack.len() < path.len() {
                    return Err(VmError::new(VmErrorKind::StackUnderflow, pc));
                }
                let indices = &stack[stack.len() - path.len()..];
                at_instruction(check_array_path(current, path, indices), pc)?;
            }

            InstructionKind::Construct { type_id, fields } => {
                let definition = program.type_definitions.get(*type_id).ok_or_else(|| {
                    VmError::new(VmErrorKind::InvalidType { type_id: *type_id }, pc)
                })?;
                let mut values = vec![None; definition.fields.len()];

                for field in fields.iter().rev() {
                    let value = at_instruction(pop_value(&mut stack), pc)?;
                    let expected = definition
                        .fields
                        .get(field.field_id)
                        .ok_or_else(|| {
                            VmError::new(
                                VmErrorKind::InvalidField {
                                    type_id: *type_id,
                                    field_id: field.field_id,
                                },
                                pc,
                            )
                        })?
                        .ty
                        .clone();
                    if value.ty() != expected {
                        return Err(VmError::new(
                            VmErrorKind::TypeMismatch {
                                expected,
                                actual: value.ty(),
                            },
                            pc,
                        ));
                    }
                    values[field.field_id] = Some(value);
                }

                let values = values
                    .into_iter()
                    .enumerate()
                    .map(|(field_id, value)| {
                        value.ok_or_else(|| {
                            VmError::new(
                                VmErrorKind::InvalidField {
                                    type_id: *type_id,
                                    field_id,
                                },
                                pc,
                            )
                        })
                    })
                    .collect::<Result<_, _>>()?;
                stack.push(Value::Aggregate {
                    type_id: *type_id,
                    fields: values,
                });
            }

            InstructionKind::FieldGet { type_id, field_id } => {
                let value = at_instruction(pop_value(&mut stack), pc)?;
                let actual_type = value.ty();
                let Value::Aggregate {
                    type_id: aggregate_type,
                    fields,
                } = value
                else {
                    return Err(VmError::new(
                        VmErrorKind::TypeMismatch {
                            expected: Type::Named(*type_id),
                            actual: actual_type,
                        },
                        pc,
                    ));
                };
                if aggregate_type != *type_id {
                    return Err(VmError::new(
                        VmErrorKind::TypeMismatch {
                            expected: Type::Named(*type_id),
                            actual: Type::Named(aggregate_type),
                        },
                        pc,
                    ));
                }
                let field = fields.get(*field_id).cloned().ok_or_else(|| {
                    VmError::new(
                        VmErrorKind::InvalidField {
                            type_id: *type_id,
                            field_id: *field_id,
                        },
                        pc,
                    )
                })?;
                stack.push(field);
            }

            InstructionKind::ConstructArray { element, length } => {
                let mut values = Vec::with_capacity(*length);
                for _ in 0..*length {
                    let value = at_instruction(pop_value(&mut stack), pc)?;
                    if value.ty() != *element {
                        return Err(VmError::new(
                            VmErrorKind::TypeMismatch {
                                expected: element.clone(),
                                actual: value.ty(),
                            },
                            pc,
                        ));
                    }
                    values.push(value);
                }
                values.reverse();
                stack.push(Value::Array {
                    element: element.clone(),
                    values,
                });
            }

            InstructionKind::Index { element, length } => {
                let index = at_instruction(pop_i64(&mut stack), pc)?;
                let value = at_instruction(pop_value(&mut stack), pc)?;
                let expected = Type::Array {
                    element: Box::new(element.clone()),
                    length: *length,
                };
                let actual = value.ty();
                let Value::Array {
                    element: actual_element,
                    values,
                } = value
                else {
                    return Err(VmError::new(
                        VmErrorKind::TypeMismatch { expected, actual },
                        pc,
                    ));
                };
                if actual_element != *element || values.len() != *length {
                    return Err(VmError::new(
                        VmErrorKind::TypeMismatch { expected, actual },
                        pc,
                    ));
                }
                let value = usize::try_from(index)
                    .ok()
                    .and_then(|index| values.get(index).cloned())
                    .ok_or_else(|| {
                        VmError::new(
                            VmErrorKind::ArrayIndexOutOfBounds {
                                index,
                                length: *length,
                            },
                            pc,
                        )
                    })?;
                stack.push(value);
            }

            InstructionKind::Call {
                function_id,
                argument_count,
            } => {
                let function = program.functions.get(*function_id).ok_or_else(|| {
                    VmError::new(
                        VmErrorKind::InvalidFunction {
                            function_id: *function_id,
                        },
                        pc,
                    )
                })?;
                if *argument_count != function.parameter_count {
                    return Err(VmError::new(
                        VmErrorKind::InvalidArgumentCount {
                            expected: function.parameter_count,
                            actual: *argument_count,
                        },
                        pc,
                    ));
                }

                let mut arguments = Vec::with_capacity(*argument_count);
                for _ in 0..*argument_count {
                    arguments.push(at_instruction(pop_value(&mut stack), pc)?);
                }
                arguments.reverse();

                if let Some(value) =
                    execute_frame(program, Frame::Function(*function_id), arguments, output)?
                {
                    stack.push(value);
                }
            }

            InstructionKind::Return { has_value } => {
                if matches!(frame, Frame::Entry) {
                    return Err(VmError::new(VmErrorKind::InvalidReturn, pc));
                }

                let value = match (&return_type, *has_value) {
                    (ReturnType::Void, false) => None,
                    (ReturnType::Value(expected), true) => {
                        let value = at_instruction(pop_value(&mut stack), pc)?;
                        if value.ty() != *expected {
                            return Err(VmError::new(
                                VmErrorKind::TypeMismatch {
                                    expected: expected.clone(),
                                    actual: value.ty(),
                                },
                                pc,
                            ));
                        }
                        Some(value)
                    }
                    _ => return Err(VmError::new(VmErrorKind::InvalidReturn, pc)),
                };

                if !stack.is_empty() {
                    return Err(VmError::new(
                        VmErrorKind::UnusedStackValues { count: stack.len() },
                        pc,
                    ));
                }
                return Ok(value);
            }

            InstructionKind::Add(ty) => {
                at_instruction(binary(ty.clone(), &mut stack, BinaryOperation::Add), pc)?;
            }

            InstructionKind::Subtract(ty) => {
                at_instruction(
                    binary(ty.clone(), &mut stack, BinaryOperation::Subtract),
                    pc,
                )?;
            }

            InstructionKind::Multiply(ty) => {
                at_instruction(
                    binary(ty.clone(), &mut stack, BinaryOperation::Multiply),
                    pc,
                )?;
            }

            InstructionKind::Divide(ty) => {
                at_instruction(binary(ty.clone(), &mut stack, BinaryOperation::Divide), pc)?;
            }

            InstructionKind::Equal(ty) => {
                at_instruction(compare(ty.clone(), &mut stack, Comparison::Equal), pc)?;
            }

            InstructionKind::NotEqual(ty) => {
                at_instruction(compare(ty.clone(), &mut stack, Comparison::NotEqual), pc)?;
            }

            InstructionKind::Less(ty) => {
                at_instruction(compare(ty.clone(), &mut stack, Comparison::Less), pc)?;
            }

            InstructionKind::LessEqual(ty) => {
                at_instruction(compare(ty.clone(), &mut stack, Comparison::LessEqual), pc)?;
            }

            InstructionKind::Greater(ty) => {
                at_instruction(compare(ty.clone(), &mut stack, Comparison::Greater), pc)?;
            }

            InstructionKind::GreaterEqual(ty) => {
                at_instruction(
                    compare(ty.clone(), &mut stack, Comparison::GreaterEqual),
                    pc,
                )?;
            }

            InstructionKind::Negate(ty) => {
                at_instruction(negate(ty.clone(), &mut stack), pc)?;
            }

            InstructionKind::Not => {
                let value = at_instruction(pop_bool(&mut stack), pc)?;
                stack.push(Value::Bool(!value));
            }

            InstructionKind::Print(ty) => {
                let value = at_instruction(pop_value(&mut stack), pc)?;

                let line = at_instruction(format_value(value, ty.clone()), pc)?;

                output.push_str(&line);

                output.push('\n');
            }

            InstructionKind::JumpIfFalse(target) => {
                let condition = at_instruction(pop_bool(&mut stack), pc)?;

                if !condition {
                    pc = *target;
                    continue;
                }
            }

            InstructionKind::Jump(target) => {
                pc = *target;
                continue;
            }

            InstructionKind::Halt => {
                if matches!(frame, Frame::Function(_)) {
                    return Err(VmError::new(VmErrorKind::InvalidReturn, pc));
                }
                break;
            }
        }

        pc += 1;
    }

    if !stack.is_empty() {
        return Err(VmError::new(
            VmErrorKind::UnusedStackValues { count: stack.len() },
            pc,
        ));
    }

    Ok(None)
}

#[derive(Clone, Copy)]
enum BinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

fn binary(ty: Type, stack: &mut Vec<Value>, operation: BinaryOperation) -> VmResult<()> {
    match ty {
        Type::Bool => {
            return Err(VmErrorKind::TypeMismatch {
                expected: Type::Integer(IntegerType::I64),
                actual: Type::Bool,
            });
        }

        Type::Integer(integer) => {
            let right = pop_integer(stack, integer)? as i128;
            let left = pop_integer(stack, integer)? as i128;
            let value = match operation {
                BinaryOperation::Add => left + right,
                BinaryOperation::Subtract => left - right,
                BinaryOperation::Multiply => left * right,
                BinaryOperation::Divide => {
                    if right == 0 {
                        return Err(VmErrorKind::DivisionByZero);
                    }
                    left / right
                }
            };
            if value < integer.minimum() as i128 || value > integer.maximum() as i128 {
                let operation = match operation {
                    BinaryOperation::Add => IntegerOperation::Add,
                    BinaryOperation::Subtract => IntegerOperation::Subtract,
                    BinaryOperation::Multiply => IntegerOperation::Multiply,
                    BinaryOperation::Divide => return Err(VmErrorKind::DivisionOverflow),
                };
                return Err(VmErrorKind::IntegerOverflow { operation, ty });
            }
            stack.push(Value::Integer(value as i64, integer));
        }

        Type::F32 => {
            let right = pop_f32(stack)?;

            let left = pop_f32(stack)?;

            let value = match operation {
                BinaryOperation::Add => left + right,

                BinaryOperation::Subtract => left - right,

                BinaryOperation::Multiply => left * right,

                BinaryOperation::Divide => left / right,
            };

            stack.push(Value::F32(value));
        }

        Type::F64 => {
            let right = pop_f64(stack)?;

            let left = pop_f64(stack)?;

            let value = match operation {
                BinaryOperation::Add => left + right,

                BinaryOperation::Subtract => left - right,

                BinaryOperation::Multiply => left * right,

                BinaryOperation::Divide => left / right,
            };

            stack.push(Value::F64(value));
        }

        Type::Named(_) | Type::Array { .. } => {
            return Err(VmErrorKind::TypeMismatch {
                expected: Type::Integer(IntegerType::I64),
                actual: ty,
            });
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
enum Comparison {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

fn compare(ty: Type, stack: &mut Vec<Value>, comparison: Comparison) -> VmResult<()> {
    let result = match ty {
        Type::Bool => {
            let right = pop_bool(stack)?;
            let left = pop_bool(stack)?;

            match comparison {
                Comparison::Equal => left == right,
                Comparison::NotEqual => left != right,
                Comparison::Less
                | Comparison::LessEqual
                | Comparison::Greater
                | Comparison::GreaterEqual => {
                    return Err(VmErrorKind::InvalidComparisonType { ty });
                }
            }
        }
        Type::Integer(integer) => {
            let right = pop_integer(stack, integer)?;
            let left = pop_integer(stack, integer)?;
            compare_values(left, right, comparison)
        }
        Type::F32 => {
            let right = pop_f32(stack)?;
            let left = pop_f32(stack)?;
            compare_values(left, right, comparison)
        }
        Type::F64 => {
            let right = pop_f64(stack)?;
            let left = pop_f64(stack)?;
            compare_values(left, right, comparison)
        }
        Type::Named(_) | Type::Array { .. } => {
            return Err(VmErrorKind::InvalidComparisonType { ty });
        }
    };

    stack.push(Value::Bool(result));
    Ok(())
}

fn compare_values<T: PartialEq + PartialOrd>(left: T, right: T, comparison: Comparison) -> bool {
    match comparison {
        Comparison::Equal => left == right,
        Comparison::NotEqual => left != right,
        Comparison::Less => left < right,
        Comparison::LessEqual => left <= right,
        Comparison::Greater => left > right,
        Comparison::GreaterEqual => left >= right,
    }
}

fn negate(ty: Type, stack: &mut Vec<Value>) -> VmResult<()> {
    match ty {
        Type::Bool => {
            return Err(VmErrorKind::TypeMismatch {
                expected: Type::Integer(IntegerType::I64),
                actual: Type::Bool,
            });
        }

        Type::Integer(integer) => {
            if !integer.is_signed() {
                return Err(VmErrorKind::InvalidNegationType { ty: integer });
            }
            let value = pop_integer(stack, integer)?;
            let value = value
                .checked_neg()
                .filter(|value| integer.contains(*value))
                .ok_or(VmErrorKind::IntegerOverflow {
                    operation: IntegerOperation::Negate,
                    ty,
                })?;
            stack.push(Value::Integer(value, integer));
        }

        Type::F32 => {
            let value = pop_f32(stack)?;

            stack.push(Value::F32(-value));
        }

        Type::F64 => {
            let value = pop_f64(stack)?;

            stack.push(Value::F64(-value));
        }

        Type::Named(_) | Type::Array { .. } => {
            return Err(VmErrorKind::TypeMismatch {
                expected: Type::Integer(IntegerType::I64),
                actual: ty,
            });
        }
    }

    Ok(())
}

fn pop_value(stack: &mut Vec<Value>) -> VmResult<Value> {
    stack.pop().ok_or(VmErrorKind::StackUnderflow)
}

fn pop_i64(stack: &mut Vec<Value>) -> VmResult<i64> {
    pop_integer(stack, IntegerType::I64)
}

fn pop_integer(stack: &mut Vec<Value>, ty: IntegerType) -> VmResult<i64> {
    match pop_value(stack)? {
        Value::Integer(value, actual) if actual == ty => Ok(value),

        other => Err(VmErrorKind::TypeMismatch {
            expected: Type::Integer(ty),
            actual: other.ty(),
        }),
    }
}

fn assign_array_path(
    current: &mut Value,
    path: &[ArrayAccess],
    indices: &[i64],
    replacement: Value,
) -> VmResult<()> {
    let Some((access, remaining_path)) = path.split_first() else {
        return Err(VmErrorKind::TypeMismatch {
            expected: current.ty(),
            actual: replacement.ty(),
        });
    };
    let Some((&index, remaining_indices)) = indices.split_first() else {
        return Err(VmErrorKind::StackUnderflow);
    };

    let expected = Type::Array {
        element: Box::new(access.element.clone()),
        length: access.length,
    };
    let actual = current.ty();
    let Value::Array { element, values } = current else {
        return Err(VmErrorKind::TypeMismatch { expected, actual });
    };
    if *element != access.element || values.len() != access.length {
        return Err(VmErrorKind::TypeMismatch { expected, actual });
    }

    let index = usize::try_from(index)
        .ok()
        .filter(|index| *index < values.len())
        .ok_or(VmErrorKind::ArrayIndexOutOfBounds {
            index,
            length: access.length,
        })?;

    if remaining_path.is_empty() {
        if replacement.ty() != access.element {
            return Err(VmErrorKind::TypeMismatch {
                expected: access.element.clone(),
                actual: replacement.ty(),
            });
        }
        values[index] = replacement;
        Ok(())
    } else {
        assign_array_path(
            &mut values[index],
            remaining_path,
            remaining_indices,
            replacement,
        )
    }
}

fn check_array_path(current: &Value, path: &[ArrayAccess], indices: &[Value]) -> VmResult<()> {
    let mut current = current;
    for (access, index) in path.iter().zip(indices) {
        let Value::Integer(index, IntegerType::I64) = index else {
            return Err(VmErrorKind::TypeMismatch {
                expected: Type::Integer(IntegerType::I64),
                actual: index.ty(),
            });
        };
        let expected = Type::Array {
            element: Box::new(access.element.clone()),
            length: access.length,
        };
        let actual = current.ty();
        let Value::Array { element, values } = current else {
            return Err(VmErrorKind::TypeMismatch { expected, actual });
        };
        if *element != access.element || values.len() != access.length {
            return Err(VmErrorKind::TypeMismatch { expected, actual });
        }
        current = usize::try_from(*index)
            .ok()
            .and_then(|index| values.get(index))
            .ok_or(VmErrorKind::ArrayIndexOutOfBounds {
                index: *index,
                length: access.length,
            })?;
    }
    Ok(())
}

fn pop_bool(stack: &mut Vec<Value>) -> VmResult<bool> {
    match pop_value(stack)? {
        Value::Bool(value) => Ok(value),

        other => Err(VmErrorKind::TypeMismatch {
            expected: Type::Bool,
            actual: other.ty(),
        }),
    }
}

fn pop_f32(stack: &mut Vec<Value>) -> VmResult<f32> {
    match pop_value(stack)? {
        Value::F32(value) => Ok(value),

        other => Err(VmErrorKind::TypeMismatch {
            expected: Type::F32,
            actual: other.ty(),
        }),
    }
}

fn pop_f64(stack: &mut Vec<Value>) -> VmResult<f64> {
    match pop_value(stack)? {
        Value::F64(value) => Ok(value),

        other => Err(VmErrorKind::TypeMismatch {
            expected: Type::F64,
            actual: other.ty(),
        }),
    }
}

fn format_value(value: Value, expected: Type) -> VmResult<String> {
    match (value, expected) {
        (Value::Bool(value), Type::Bool) => Ok(value.to_string()),

        (Value::Integer(value, actual), Type::Integer(expected)) if actual == expected => {
            Ok(value.to_string())
        }

        (Value::F32(value), Type::F32) => Ok(trim_decimal(format!("{value:.9}"))),

        (Value::F64(value), Type::F64) => Ok(trim_decimal(format!("{value:.17}"))),

        (value, expected) => Err(VmErrorKind::TypeMismatch {
            expected,
            actual: value.ty(),
        }),
    }
}

fn trim_decimal(mut text: String) -> String {
    if !text.contains('.') {
        return text;
    }

    while text.ends_with('0') {
        text.pop();
    }

    if text.ends_with('.') {
        text.pop();
    }

    if text == "-0" {
        return "0".to_owned();
    }

    text
}

#[cfg(test)]
mod tests {
    use crate::types::IntegerType;
    use crate::{
        bytecode::{self, BytecodeProgram, Instruction, InstructionKind, Slot, Type},
        compile_to_bytecode, compile_to_ir,
    };

    use super::{IntegerOperation, VmErrorKind, run};

    #[test]
    fn executes_floating_point_program() {
        let program =
            compile_to_ir("a: f32 = 0.1 + 0.2; b: f64 = 0.1 + 0.2; print(a); print(b);").unwrap();

        let bytecode = bytecode::lower(&program).unwrap();

        let output = run(&bytecode).unwrap();

        assert_eq!(output, "0.300000012\n0.30000000000000004\n");
    }

    #[test]
    fn reports_integer_division_by_zero_at_instruction() {
        let program = compile_to_bytecode("print(1 / 0);").unwrap();

        let error = run(&program).unwrap_err();

        assert_eq!(error.kind(), VmErrorKind::DivisionByZero);
        assert_eq!(error.instruction_index(), 2);
    }

    #[test]
    fn distinguishes_integer_division_overflow() {
        let program = BytecodeProgram {
            type_definitions: Vec::new(),
            functions: Vec::new(),
            slots: Vec::new(),
            instructions: vec![
                Instruction::synthetic(InstructionKind::PushInteger(i64::MIN, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::PushInteger(-1, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::Divide(Type::Integer(IntegerType::I64))),
                Instruction::synthetic(InstructionKind::Halt),
            ],
        };

        let error = run(&program).unwrap_err();

        assert_eq!(error.kind(), VmErrorKind::DivisionOverflow);
        assert_eq!(error.instruction_index(), 2);
    }

    #[test]
    fn reports_integer_addition_overflow_at_instruction() {
        let program = compile_to_bytecode("print(9223372036854775807 + 1);").unwrap();

        let error = run(&program).unwrap_err();

        assert_eq!(
            error.kind(),
            VmErrorKind::IntegerOverflow {
                operation: IntegerOperation::Add,
                ty: Type::Integer(IntegerType::I64),
            }
        );
        assert_eq!(error.instruction_index(), 2);
    }

    #[test]
    fn reports_integer_subtraction_overflow_at_instruction() {
        let program = BytecodeProgram {
            type_definitions: Vec::new(),
            functions: Vec::new(),
            slots: Vec::new(),
            instructions: vec![
                Instruction::synthetic(InstructionKind::PushInteger(i64::MIN, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::PushInteger(1, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::Subtract(Type::Integer(IntegerType::I64))),
                Instruction::synthetic(InstructionKind::Halt),
            ],
        };

        let error = run(&program).unwrap_err();

        assert_eq!(
            error.kind(),
            VmErrorKind::IntegerOverflow {
                operation: IntegerOperation::Subtract,
                ty: Type::Integer(IntegerType::I64),
            }
        );
        assert_eq!(error.instruction_index(), 2);
    }

    #[test]
    fn reports_integer_multiplication_overflow_at_instruction() {
        let program = compile_to_bytecode("print(9223372036854775807 * 2);").unwrap();

        let error = run(&program).unwrap_err();

        assert_eq!(
            error.kind(),
            VmErrorKind::IntegerOverflow {
                operation: IntegerOperation::Multiply,
                ty: Type::Integer(IntegerType::I64),
            }
        );
        assert_eq!(error.instruction_index(), 2);
    }

    #[test]
    fn reports_integer_negation_overflow_at_instruction() {
        let program = BytecodeProgram {
            type_definitions: Vec::new(),
            functions: Vec::new(),
            slots: Vec::new(),
            instructions: vec![
                Instruction::synthetic(InstructionKind::PushInteger(i64::MIN, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::Negate(Type::Integer(IntegerType::I64))),
                Instruction::synthetic(InstructionKind::Halt),
            ],
        };

        let error = run(&program).unwrap_err();

        assert_eq!(
            error.kind(),
            VmErrorKind::IntegerOverflow {
                operation: IntegerOperation::Negate,
                ty: Type::Integer(IntegerType::I64),
            }
        );
        assert_eq!(error.instruction_index(), 1);
    }

    #[test]
    fn reports_overflow_when_negating_the_minimum_i64_literal() {
        let program = compile_to_bytecode("print(--9223372036854775808);").unwrap();

        let error = run(&program).unwrap_err();

        assert_eq!(
            error.kind(),
            VmErrorKind::IntegerOverflow {
                operation: IntegerOperation::Negate,
                ty: Type::Integer(IntegerType::I64),
            }
        );
        assert_eq!(error.instruction_index(), 1);
    }

    #[test]
    fn rejects_bytecode_assignment_to_immutable_slot() {
        let program = BytecodeProgram {
            type_definitions: Vec::new(),
            functions: Vec::new(),
            slots: vec![Slot {
                name: "value".into(),
                ty: Type::Integer(IntegerType::I64),
                mutable: false,
            }],
            instructions: vec![
                Instruction::synthetic(InstructionKind::PushInteger(1, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::Store(0)),
                Instruction::synthetic(InstructionKind::PushInteger(2, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::Assign(0)),
                Instruction::synthetic(InstructionKind::Halt),
            ],
        };

        let error = run(&program).unwrap_err();

        assert_eq!(
            error.kind(),
            VmErrorKind::AssignmentToImmutableSlot { slot: 0 }
        );
        assert_eq!(error.instruction_index(), 3);
    }

    #[test]
    fn rejects_distinct_initializers_for_the_same_slot() {
        let program = BytecodeProgram {
            type_definitions: Vec::new(),
            functions: Vec::new(),
            slots: vec![Slot {
                name: "value".into(),
                ty: Type::Integer(IntegerType::I64),
                mutable: false,
            }],
            instructions: vec![
                Instruction::synthetic(InstructionKind::PushInteger(1, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::Store(0)),
                Instruction::synthetic(InstructionKind::PushInteger(2, IntegerType::I64)),
                Instruction::synthetic(InstructionKind::Store(0)),
                Instruction::synthetic(InstructionKind::Halt),
            ],
        };

        let error = run(&program).unwrap_err();

        assert_eq!(
            error.kind(),
            VmErrorKind::SlotAlreadyInitialized { slot: 0 }
        );
        assert_eq!(error.instruction_index(), 3);
    }

    #[test]
    fn reports_missing_instruction_without_panicking() {
        let program = BytecodeProgram {
            type_definitions: Vec::new(),
            functions: Vec::new(),
            slots: Vec::new(),
            instructions: Vec::new(),
        };

        let error = run(&program).unwrap_err();

        assert_eq!(error.kind(), VmErrorKind::InstructionOutOfBounds);
        assert_eq!(error.instruction_index(), 0);
    }

    #[test]
    fn executes_else_and_skips_if_without_else() {
        let program = compile_to_bytecode(
            "if false { print(1); } else { print(2); }
             if false { print(3); }
             print(4);",
        )
        .unwrap();

        assert_eq!(run(&program).unwrap(), "2\n4\n");
    }

    #[test]
    fn executes_while_and_rechecks_its_condition() {
        let program = compile_to_bytecode(
            "mut count: i64 = 0;
             while count < 3 {
                 print(count);
                 count = count + 1;
             }
             print(count);",
        )
        .unwrap();

        assert_eq!(run(&program).unwrap(), "0\n1\n2\n3\n");
    }

    #[test]
    fn executes_break_and_continue() {
        let program = compile_to_bytecode(
            "mut value: i64 = 0;
             mut sum: i64 = 0;
             while value < 10 {
                 value = value + 1;
                 if value < 3 { continue; }
                 if value > 5 { break; }
                 sum = sum + value;
             }
             print(sum);
             print(value);",
        )
        .unwrap();

        assert_eq!(run(&program).unwrap(), "12\n6\n");
    }

    #[test]
    fn break_targets_only_the_innermost_loop() {
        let program = compile_to_bytecode(
            "mut outer: i64 = 0;
             mut hits: i64 = 0;
             while outer < 2 {
                 mut inner: i64 = 0;
                 while inner < 3 {
                     inner = inner + 1;
                     if inner == 2 { break; }
                     hits = hits + 1;
                 }
                 outer = outer + 1;
             }
             print(hits);",
        )
        .unwrap();

        assert_eq!(run(&program).unwrap(), "2\n");
    }
}
