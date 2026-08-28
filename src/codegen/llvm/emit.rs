use std::fmt::Write;

use super::ir::{BinaryOp, Instruction, Module, Operand, PrintFormat, SlotId, Temp, Type};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    output.push_str("@.fmt_i64 = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n");
    output.push_str("@.fmt_f32 = private unnamed_addr constant [6 x i8] c\"%.9g\\0A\\00\"\n");
    output.push_str("@.fmt_f64 = private unnamed_addr constant [7 x i8] c\"%.17g\\0A\\00\"\n\n");

    output.push_str("declare i32 @printf(ptr, ...)\n\n");

    output.push_str("define i32 @main() {\n");
    output.push_str("entry:\n");

    for instruction in &module.instructions {
        emit_instruction(instruction, module, &mut output);
    }

    output.push_str("  ret i32 0\n");
    output.push_str("}\n");

    output
}

fn emit_instruction(instruction: &Instruction, module: &Module, output: &mut String) {
    match instruction {
        Instruction::Alloca { slot } => {
            let slot = slot_by_id(module, *slot);

            writeln!(
                output,
                "  %primer_{} = alloca {}",
                slot.name,
                type_name(slot.ty),
            )
            .unwrap();
        }

        Instruction::Store { ty, value, slot } => {
            writeln!(
                output,
                "  store {} {}, ptr %primer_{}",
                type_name(*ty),
                operand(*value),
                slot_by_id(module, *slot).name,
            )
            .unwrap();
        }

        Instruction::Load { dest, ty, slot } => {
            writeln!(
                output,
                "  {} = load {}, ptr %primer_{}",
                temp(*dest),
                type_name(*ty),
                slot_by_id(module, *slot).name,
            )
            .unwrap();
        }

        Instruction::Binary {
            dest,
            op,
            ty,
            left,
            right,
        } => {
            writeln!(
                output,
                "  {} = {} {} {}, {}",
                temp(*dest),
                binary_name(*op),
                type_name(*ty),
                operand(*left),
                operand(*right),
            )
            .unwrap();
        }

        Instruction::FNeg { dest, ty, value } => {
            writeln!(
                output,
                "  {} = fneg {} {}",
                temp(*dest),
                type_name(*ty),
                operand(*value),
            )
            .unwrap();
        }

        Instruction::FPExt { dest, value } => {
            writeln!(
                output,
                "  {} = fpext float {} to double",
                temp(*dest),
                operand(*value),
            )
            .unwrap();
        }

        Instruction::CallPrintf {
            format,
            arg_ty,
            value,
        } => {
            writeln!(
                output,
                "  call i32 (ptr, ...) @printf(ptr {}, {} {})",
                format_name(*format),
                type_name(*arg_ty),
                operand(*value),
            )
            .unwrap();
        }
    }
}

fn slot_by_id(module: &Module, id: SlotId) -> &super::ir::Slot {
    &module.slots[id.0]
}

fn temp(temp: Temp) -> String {
    format!("%tmp{}", temp.0)
}

fn type_name(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "i64",
        Type::Float => "float",
        Type::Double => "double",
    }
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Sub => "sub",
        BinaryOp::Mul => "mul",
        BinaryOp::SDiv => "sdiv",
        BinaryOp::FAdd => "fadd",
        BinaryOp::FSub => "fsub",
        BinaryOp::FMul => "fmul",
        BinaryOp::FDiv => "fdiv",
    }
}

fn format_name(format: PrintFormat) -> &'static str {
    match format {
        PrintFormat::I64 => "@.fmt_i64",
        PrintFormat::F32 => "@.fmt_f32",
        PrintFormat::F64 => "@.fmt_f64",
    }
}

fn operand(operand: Operand) -> String {
    match operand {
        Operand::Integer(value) => value.to_string(),

        Operand::Float32(bits) => {
            // LLVM 22 legacy hexadecimal syntax represents
            // float constants using the corresponding exact
            // double representation.
            let as_double = f32::from_bits(bits) as f64;

            format!("0x{:016X}", as_double.to_bits())
        }

        Operand::Float64(bits) => format!("0x{bits:016X}"),

        Operand::Temp(id) => temp(id),
    }
}
