use std::fmt::Write;

use super::ir::{BinaryOp, Instruction, Module, Operand, PrintFormat, Temp, Type};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    // printf format strings.
    //
    // b 10 = '\n'
    // b 0  = '\0'
    output.push_str("data $fmt_i64 = { b \"%lld\", b 10, b 0 }\n");
    output.push_str("data $fmt_f32 = { b \"%.9g\", b 10, b 0 }\n");
    output.push_str("data $fmt_f64 = { b \"%.17g\", b 10, b 0 }\n\n");

    output.push_str("export function w $main() {\n");
    output.push_str("@start\n");

    for instruction in &module.instructions {
        emit_instruction(instruction, &mut output);
    }

    output.push_str("  ret 0\n");
    output.push_str("}\n");

    output
}

fn emit_instruction(instruction: &Instruction, output: &mut String) {
    match instruction {
        Instruction::Copy { name, ty, value } => {
            writeln!(
                output,
                "  %primer_{name} ={} copy {}",
                type_name(*ty),
                operand(value),
            )
            .unwrap();
        }

        Instruction::Negate { dest, ty, value } => {
            writeln!(
                output,
                "  {} ={} neg {}",
                temp(*dest),
                type_name(*ty),
                operand(value),
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
                "  {} ={} {} {}, {}",
                temp(*dest),
                type_name(*ty),
                binary_name(*op),
                operand(left),
                operand(right),
            )
            .unwrap();
        }

        Instruction::ExtendSingleToDouble { dest, value } => {
            writeln!(output, "  {} =d exts {}", temp(*dest), operand(value)).unwrap();
        }

        Instruction::CallPrintf {
            dest,
            format,
            arg_ty,
            value,
        } => {
            writeln!(
                output,
                "  {} =w call $printf(l ${}, ..., {} {})",
                temp(*dest),
                format_name(*format),
                type_name(*arg_ty),
                operand(value),
            )
            .unwrap();
        }
    }
}

fn type_name(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "l",
        Type::Single => "s",
        Type::Double => "d",
    }
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Subtract => "sub",
        BinaryOp::Multiply => "mul",
        BinaryOp::Divide => "div",
    }
}

fn format_name(format: PrintFormat) -> &'static str {
    match format {
        PrintFormat::I64 => "fmt_i64",
        PrintFormat::F32 => "fmt_f32",
        PrintFormat::F64 => "fmt_f64",
    }
}

fn operand(value: &Operand) -> String {
    match value {
        Operand::Integer(value) => value.to_string(),
        Operand::Float32(text) => format!("s_{text}"),
        Operand::Float64(text) => format!("d_{text}"),
        Operand::Binding(name) => format!("%primer_{name}"),
        Operand::Temp(temp) => self::temp(*temp),
    }
}

fn temp(temp: Temp) -> String {
    format!("%tmp{}", temp.0)
}
