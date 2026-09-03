use std::fmt::Write;

use super::ir::{BinaryOp, CompareOp, Instruction, Module, Operand, PrintFormat, Temp, Type};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    // printf format strings.
    //
    // b 10 = '\n'
    // b 0  = '\0'
    output.push_str("data $fmt_i64 = { b \"%lld\", b 10, b 0 }\n");
    output.push_str("data $fmt_f32 = { b \"%.9g\", b 10, b 0 }\n");
    output.push_str("data $fmt_f64 = { b \"%.17g\", b 10, b 0 }\n");

    if uses_bool_print(module) {
        output.push_str("data $bool_false = { b \"false\", b 0 }\n");
        output.push_str("data $bool_true = { b \"true\", b 0 }\n");
        output.push_str("data $bool_texts = align 8 { l $bool_false, l $bool_true }\n");
    }

    output.push('\n');

    output.push_str("export function w $main() {\n");
    output.push_str("@start\n");

    for slot in &module.slots {
        writeln!(output, "  %slot_{} =l alloc8 8", slot.name).unwrap();
    }

    for instruction in &module.instructions {
        emit_instruction(instruction, module, &mut output);
    }

    output.push_str("  ret 0\n");
    output.push_str("}\n");

    output
}

fn emit_instruction(instruction: &Instruction, module: &Module, output: &mut String) {
    match instruction {
        Instruction::Label { id, name } => {
            writeln!(output, "@block{id} # {name}").unwrap();
        }

        Instruction::Branch {
            condition,
            then_label,
            else_label,
        } => {
            writeln!(
                output,
                "  jnz {}, @block{then_label}, @block{else_label}",
                operand(condition)
            )
            .unwrap();
        }

        Instruction::Jump(label) => {
            writeln!(output, "  jmp @block{label}").unwrap();
        }

        Instruction::Store { slot, ty, value } => {
            writeln!(
                output,
                "  {} {}, %slot_{}",
                store_name(*ty),
                operand(value),
                module.slots[*slot].name,
            )
            .unwrap();
        }

        Instruction::Load { dest, slot, ty } => {
            writeln!(
                output,
                "  {} ={} {} %slot_{}",
                temp(*dest),
                type_name(*ty),
                load_name(*ty),
                module.slots[*slot].name,
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

        Instruction::Not { dest, value } => {
            writeln!(output, "  {} =w ceqw {}, 0", temp(*dest), operand(value)).unwrap();
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

        Instruction::Compare {
            dest,
            op,
            operand_ty,
            left,
            right,
        } => {
            writeln!(
                output,
                "  {} =w {} {}, {}",
                temp(*dest),
                compare_name(*op, *operand_ty),
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

        Instruction::CallPrintBool {
            offset,
            scaled_offset,
            address,
            text,
            result,
            value,
        } => {
            writeln!(output, "  {} =l extsw {}", temp(*offset), operand(value)).unwrap();
            writeln!(
                output,
                "  {} =l mul {}, 8",
                temp(*scaled_offset),
                temp(*offset)
            )
            .unwrap();
            writeln!(
                output,
                "  {} =l add $bool_texts, {}",
                temp(*address),
                temp(*scaled_offset)
            )
            .unwrap();
            writeln!(output, "  {} =l loadl {}", temp(*text), temp(*address)).unwrap();
            writeln!(
                output,
                "  {} =w call $puts(l {})",
                temp(*result),
                temp(*text)
            )
            .unwrap();
        }
    }
}

fn type_name(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "w",
        Type::I64 => "l",
        Type::Single => "s",
        Type::Double => "d",
    }
}

fn store_name(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "storew",
        Type::I64 => "storel",
        Type::Single => "stores",
        Type::Double => "stored",
    }
}

fn load_name(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "loadw",
        Type::I64 => "loadl",
        Type::Single => "loads",
        Type::Double => "loadd",
    }
}

fn compare_name(op: CompareOp, ty: Type) -> &'static str {
    match (op, ty) {
        (CompareOp::Equal, Type::Bool) => "ceqw",
        (CompareOp::NotEqual, Type::Bool) => "cnew",

        (CompareOp::Equal, Type::I64) => "ceql",
        (CompareOp::NotEqual, Type::I64) => "cnel",
        (CompareOp::Less, Type::I64) => "csltl",
        (CompareOp::LessEqual, Type::I64) => "cslel",
        (CompareOp::Greater, Type::I64) => "csgtl",
        (CompareOp::GreaterEqual, Type::I64) => "csgel",

        (CompareOp::Equal, Type::Single) => "ceqs",
        (CompareOp::NotEqual, Type::Single) => "cnes",
        (CompareOp::Less, Type::Single) => "clts",
        (CompareOp::LessEqual, Type::Single) => "cles",
        (CompareOp::Greater, Type::Single) => "cgts",
        (CompareOp::GreaterEqual, Type::Single) => "cges",

        (CompareOp::Equal, Type::Double) => "ceqd",
        (CompareOp::NotEqual, Type::Double) => "cned",
        (CompareOp::Less, Type::Double) => "cltd",
        (CompareOp::LessEqual, Type::Double) => "cled",
        (CompareOp::Greater, Type::Double) => "cgtd",
        (CompareOp::GreaterEqual, Type::Double) => "cged",

        (
            CompareOp::Less | CompareOp::LessEqual | CompareOp::Greater | CompareOp::GreaterEqual,
            Type::Bool,
        ) => {
            unreachable!("semantic analysis rejects boolean ordering")
        }
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
        Operand::Boolean(value) => i32::from(*value).to_string(),
        Operand::Integer(value) => value.to_string(),
        Operand::Float32(text) => format!("s_{text}"),
        Operand::Float64(text) => format!("d_{text}"),
        Operand::Temp(temp) => self::temp(*temp),
    }
}

fn temp(temp: Temp) -> String {
    format!("%tmp{}", temp.0)
}

fn uses_bool_print(module: &Module) -> bool {
    module
        .instructions
        .iter()
        .any(|instruction| matches!(instruction, Instruction::CallPrintBool { .. }))
}
