pub mod render;

use crate::bytecode::{BytecodeProgram, InstructionKind, Type};

/// Primer VMの実行中に発生した問題の種類を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmErrorKind {
    /// 実行位置がbytecodeの範囲外へ到達しました。
    InstructionOutOfBounds,

    /// 存在しないスロットへアクセスしました。
    InvalidSlot { slot: usize },

    /// 初期化前のスロットを読み取りました。
    UninitializedSlot { slot: usize },

    /// 別の初期化命令が初期化済みのスロットへ書き込みました。
    SlotAlreadyInitialized { slot: usize },

    /// 初期化前のスロットへ再代入しようとしました。
    AssignmentToUninitializedSlot { slot: usize },

    /// 不変のスロットへ再代入しようとしました。
    AssignmentToImmutableSlot { slot: usize },

    /// 必要な値がスタックにありませんでした。
    StackUnderflow,

    /// 命令が期待した型と実際の値の型が一致しませんでした。
    TypeMismatch { expected: Type, actual: Type },

    /// その型では利用できない比較命令を実行しようとしました。
    InvalidComparisonType { ty: Type },

    /// 整数をゼロで除算しようとしました。
    DivisionByZero,

    /// 整数除算の結果を`i64`で表現できませんでした。
    DivisionOverflow,

    /// VM停止時に未使用の値がスタックへ残っていました。
    UnusedStackValues { count: usize },
}

/// Primer VMの実行エラーと発生したbytecode命令位置を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmError {
    kind: VmErrorKind,
    instruction_index: usize,
}

impl VmError {
    const fn new(kind: VmErrorKind, instruction_index: usize) -> Self {
        Self {
            kind,
            instruction_index,
        }
    }

    /// エラーの種類を返します。
    pub const fn kind(self) -> VmErrorKind {
        self.kind
    }

    /// `emit-bytecode`の表示と対応する0から始まる命令番号を返します。
    pub const fn instruction_index(self) -> usize {
        self.instruction_index
    }
}

#[derive(Debug, Clone)]
enum Value {
    Bool(bool),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Value {
    const fn ty(&self) -> Type {
        match self {
            Self::Bool(_) => Type::Bool,
            Self::I64(_) => Type::I64,
            Self::F32(_) => Type::F32,
            Self::F64(_) => Type::F64,
        }
    }
}

type VmResult<T> = Result<T, VmErrorKind>;

fn at_instruction<T>(result: VmResult<T>, instruction_index: usize) -> Result<T, VmError> {
    result.map_err(|kind| VmError::new(kind, instruction_index))
}

pub fn run(program: &BytecodeProgram) -> Result<String, VmError> {
    let mut stack: Vec<Value> = Vec::new();

    let mut slots: Vec<Option<Value>> = vec![None; program.slots.len()];
    let mut initializers: Vec<Option<usize>> = vec![None; program.slots.len()];

    let mut output = String::new();

    let mut pc = 0;

    loop {
        let instruction = program
            .instructions
            .get(pc)
            .ok_or_else(|| VmError::new(VmErrorKind::InstructionOutOfBounds, pc))?;

        match &instruction.kind {
            InstructionKind::PushBool(value) => {
                stack.push(Value::Bool(*value));
            }

            InstructionKind::PushI64(value) => {
                stack.push(Value::I64(*value));
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

                let expected = program
                    .slots
                    .get(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?
                    .ty;

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

                let slot_info = program
                    .slots
                    .get(*slot)
                    .ok_or_else(|| VmError::new(VmErrorKind::InvalidSlot { slot: *slot }, pc))?;

                if !slot_info.mutable {
                    return Err(VmError::new(
                        VmErrorKind::AssignmentToImmutableSlot { slot: *slot },
                        pc,
                    ));
                }

                let expected = slot_info.ty;

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

            InstructionKind::Add(ty) => {
                at_instruction(binary(*ty, &mut stack, BinaryOperation::Add), pc)?;
            }

            InstructionKind::Subtract(ty) => {
                at_instruction(binary(*ty, &mut stack, BinaryOperation::Subtract), pc)?;
            }

            InstructionKind::Multiply(ty) => {
                at_instruction(binary(*ty, &mut stack, BinaryOperation::Multiply), pc)?;
            }

            InstructionKind::Divide(ty) => {
                at_instruction(binary(*ty, &mut stack, BinaryOperation::Divide), pc)?;
            }

            InstructionKind::Equal(ty) => {
                at_instruction(compare(*ty, &mut stack, Comparison::Equal), pc)?;
            }

            InstructionKind::NotEqual(ty) => {
                at_instruction(compare(*ty, &mut stack, Comparison::NotEqual), pc)?;
            }

            InstructionKind::Less(ty) => {
                at_instruction(compare(*ty, &mut stack, Comparison::Less), pc)?;
            }

            InstructionKind::LessEqual(ty) => {
                at_instruction(compare(*ty, &mut stack, Comparison::LessEqual), pc)?;
            }

            InstructionKind::Greater(ty) => {
                at_instruction(compare(*ty, &mut stack, Comparison::Greater), pc)?;
            }

            InstructionKind::GreaterEqual(ty) => {
                at_instruction(compare(*ty, &mut stack, Comparison::GreaterEqual), pc)?;
            }

            InstructionKind::Negate(ty) => {
                at_instruction(negate(*ty, &mut stack), pc)?;
            }

            InstructionKind::Not => {
                let value = at_instruction(pop_bool(&mut stack), pc)?;
                stack.push(Value::Bool(!value));
            }

            InstructionKind::Print(ty) => {
                let value = at_instruction(pop_value(&mut stack), pc)?;

                let line = at_instruction(format_value(value, *ty), pc)?;

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

    Ok(output)
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
                expected: Type::I64,
                actual: Type::Bool,
            });
        }

        Type::I64 => {
            let right = pop_i64(stack)?;

            let left = pop_i64(stack)?;

            let value = match operation {
                BinaryOperation::Add => left.wrapping_add(right),

                BinaryOperation::Subtract => left.wrapping_sub(right),

                BinaryOperation::Multiply => left.wrapping_mul(right),

                BinaryOperation::Divide => {
                    if right == 0 {
                        return Err(VmErrorKind::DivisionByZero);
                    }

                    left.checked_div(right)
                        .ok_or(VmErrorKind::DivisionOverflow)?
                }
            };

            stack.push(Value::I64(value));
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
        Type::I64 => {
            let right = pop_i64(stack)?;
            let left = pop_i64(stack)?;
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
                expected: Type::I64,
                actual: Type::Bool,
            });
        }

        Type::I64 => {
            let value = pop_i64(stack)?;

            stack.push(Value::I64(value.wrapping_neg()));
        }

        Type::F32 => {
            let value = pop_f32(stack)?;

            stack.push(Value::F32(-value));
        }

        Type::F64 => {
            let value = pop_f64(stack)?;

            stack.push(Value::F64(-value));
        }
    }

    Ok(())
}

fn pop_value(stack: &mut Vec<Value>) -> VmResult<Value> {
    stack.pop().ok_or(VmErrorKind::StackUnderflow)
}

fn pop_i64(stack: &mut Vec<Value>) -> VmResult<i64> {
    match pop_value(stack)? {
        Value::I64(value) => Ok(value),

        other => Err(VmErrorKind::TypeMismatch {
            expected: Type::I64,
            actual: other.ty(),
        }),
    }
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

        (Value::I64(value), Type::I64) => Ok(value.to_string()),

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
    use crate::{
        bytecode::{self, BytecodeProgram, Instruction, InstructionKind, Slot, Type},
        compile_to_bytecode, compile_to_ir,
    };

    use super::{VmErrorKind, run};

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
            slots: Vec::new(),
            instructions: vec![
                Instruction::synthetic(InstructionKind::PushI64(i64::MIN)),
                Instruction::synthetic(InstructionKind::PushI64(-1)),
                Instruction::synthetic(InstructionKind::Divide(Type::I64)),
                Instruction::synthetic(InstructionKind::Halt),
            ],
        };

        let error = run(&program).unwrap_err();

        assert_eq!(error.kind(), VmErrorKind::DivisionOverflow);
        assert_eq!(error.instruction_index(), 2);
    }

    #[test]
    fn rejects_bytecode_assignment_to_immutable_slot() {
        let program = BytecodeProgram {
            slots: vec![Slot {
                name: "value".into(),
                ty: Type::I64,
                mutable: false,
            }],
            instructions: vec![
                Instruction::synthetic(InstructionKind::PushI64(1)),
                Instruction::synthetic(InstructionKind::Store(0)),
                Instruction::synthetic(InstructionKind::PushI64(2)),
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
            slots: vec![Slot {
                name: "value".into(),
                ty: Type::I64,
                mutable: false,
            }],
            instructions: vec![
                Instruction::synthetic(InstructionKind::PushI64(1)),
                Instruction::synthetic(InstructionKind::Store(0)),
                Instruction::synthetic(InstructionKind::PushI64(2)),
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
