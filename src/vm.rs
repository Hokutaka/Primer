use crate::bytecode::{BytecodeProgram, Instruction, Type};

#[derive(Debug, Clone)]
enum Value {
    I64(i64),
    F32(f32),
    F64(f64),
}

pub fn run(program: &BytecodeProgram) -> Result<String, String> {
    let mut stack: Vec<Value> = Vec::new();

    let mut slots: Vec<Option<Value>> = vec![None; program.slots.len()];

    let mut output = String::new();

    let mut pc = 0;

    loop {
        let instruction = program
            .instructions
            .get(pc)
            .ok_or_else(|| format!("bytecode execution escaped program at pc {pc}"))?;

        match instruction {
            Instruction::PushI64(value) => {
                stack.push(Value::I64(*value));
            }

            Instruction::PushF32(value) => {
                stack.push(Value::F32(*value));
            }

            Instruction::PushF64(value) => {
                stack.push(Value::F64(*value));
            }

            Instruction::Load(slot) => {
                let value = slots
                    .get(*slot)
                    .ok_or_else(|| format!("invalid slot {slot}"))?
                    .clone()
                    .ok_or_else(|| format!("load from uninitialized slot {slot}"))?;

                stack.push(value);
            }

            Instruction::Store(slot) => {
                let value = pop_value(&mut stack)?;

                let destination = slots
                    .get_mut(*slot)
                    .ok_or_else(|| format!("invalid slot {slot}"))?;

                if destination.is_some() {
                    return Err(format!("slot {slot} already initialized"));
                }

                *destination = Some(value);
            }

            Instruction::Add(ty) => {
                binary(*ty, &mut stack, BinaryOperation::Add)?;
            }

            Instruction::Subtract(ty) => {
                binary(*ty, &mut stack, BinaryOperation::Subtract)?;
            }

            Instruction::Multiply(ty) => {
                binary(*ty, &mut stack, BinaryOperation::Multiply)?;
            }

            Instruction::Divide(ty) => {
                binary(*ty, &mut stack, BinaryOperation::Divide)?;
            }

            Instruction::Negate(ty) => {
                negate(*ty, &mut stack)?;
            }

            Instruction::Print(ty) => {
                let value = pop_value(&mut stack)?;

                let line = format_value(value, *ty)?;

                output.push_str(&line);

                output.push('\n');
            }

            Instruction::Halt => {
                break;
            }
        }

        pc += 1;
    }

    if !stack.is_empty() {
        return Err(format!(
            "VM halted with {} value(s) left on stack",
            stack.len(),
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

fn binary(ty: Type, stack: &mut Vec<Value>, operation: BinaryOperation) -> Result<(), String> {
    match ty {
        Type::I64 => {
            let right = pop_i64(stack)?;

            let left = pop_i64(stack)?;

            let value = match operation {
                BinaryOperation::Add => left.wrapping_add(right),

                BinaryOperation::Subtract => left.wrapping_sub(right),

                BinaryOperation::Multiply => left.wrapping_mul(right),

                BinaryOperation::Divide => left
                    .checked_div(right)
                    .ok_or_else(|| "invalid i64 division".to_owned())?,
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

fn negate(ty: Type, stack: &mut Vec<Value>) -> Result<(), String> {
    match ty {
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

fn pop_value(stack: &mut Vec<Value>) -> Result<Value, String> {
    stack.pop().ok_or_else(|| "VM stack underflow".to_owned())
}

fn pop_i64(stack: &mut Vec<Value>) -> Result<i64, String> {
    match pop_value(stack)? {
        Value::I64(value) => Ok(value),

        other => Err(format!("expected i64, found {other:?}")),
    }
}

fn pop_f32(stack: &mut Vec<Value>) -> Result<f32, String> {
    match pop_value(stack)? {
        Value::F32(value) => Ok(value),

        other => Err(format!("expected f32, found {other:?}")),
    }
}

fn pop_f64(stack: &mut Vec<Value>) -> Result<f64, String> {
    match pop_value(stack)? {
        Value::F64(value) => Ok(value),

        other => Err(format!("expected f64, found {other:?}")),
    }
}

fn format_value(value: Value, expected: Type) -> Result<String, String> {
    match (value, expected) {
        (Value::I64(value), Type::I64) => Ok(value.to_string()),

        (Value::F32(value), Type::F32) => Ok(trim_decimal(format!("{value:.9}"))),

        (Value::F64(value), Type::F64) => Ok(trim_decimal(format!("{value:.17}"))),

        (value, expected) => Err(format!(
            "print type mismatch: expected {expected:?}, found {value:?}"
        )),
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
    use crate::{bytecode, compile_to_ir};

    use super::run;

    #[test]
    fn executes_floating_point_program() {
        let program =
            compile_to_ir("a: f32 = 0.1 + 0.2; b: f64 = 0.1 + 0.2; print(a); print(b);").unwrap();

        let bytecode = bytecode::lower(&program);

        let output = run(&bytecode).unwrap();

        assert_eq!(output, "0.300000012\n0.30000000000000004\n");
    }
}
