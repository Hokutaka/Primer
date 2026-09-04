use std::fmt::Write;

use super::ir::{
    BinaryOp, CompareOp, Instruction, Label, Module, Operand, PrintFormat, SlotId, Temp, Type,
};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    output.push_str("@.fmt_i64 = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n");
    output.push_str("@.fmt_f32 = private unnamed_addr constant [6 x i8] c\"%.9g\\0A\\00\"\n");
    output.push_str("@.fmt_f64 = private unnamed_addr constant [7 x i8] c\"%.17g\\0A\\00\"\n");

    for definition in &module.type_definitions {
        write!(
            output,
            "%primer.type.{}.{} = type {{ ",
            definition.name, definition.id
        )
        .unwrap();
        for (index, field) in definition.fields.iter().enumerate() {
            if index > 0 {
                output.push_str(", ");
            }
            output.push_str(&type_name(*field, module));
        }
        output.push_str(" }\n");
    }

    if uses_bool_print(module) {
        output.push_str("@.bool_true = private unnamed_addr constant [5 x i8] c\"true\\00\"\n");
        output.push_str("@.bool_false = private unnamed_addr constant [6 x i8] c\"false\\00\"\n");
    }

    output.push('\n');

    output.push_str("declare i32 @printf(ptr, ...)\n");

    if uses_bool_print(module) {
        output.push_str("declare i32 @puts(ptr)\n");
    }

    output.push('\n');

    output.push_str("define i32 @main() {\n");
    output.push_str("entry:\n");

    for slot in &module.slots {
        writeln!(
            output,
            "  %primer_{} = alloca {}",
            slot.name,
            type_name(slot.ty, module),
        )
        .unwrap();
    }

    for instruction in &module.instructions {
        emit_instruction(instruction, module, &mut output);
    }

    output.push_str("  ret i32 0\n");
    output.push_str("}\n");

    output
}

fn emit_instruction(instruction: &Instruction, module: &Module, output: &mut String) {
    match instruction {
        Instruction::Label { id, name } => {
            writeln!(output, "{}: ; {name}", label(*id)).unwrap();
        }

        Instruction::Branch {
            condition,
            then_label,
            else_label,
        } => {
            writeln!(
                output,
                "  br i1 {}, label %{}, label %{}",
                operand(*condition),
                label(*then_label),
                label(*else_label),
            )
            .unwrap();
        }

        Instruction::Jump { label: target } => {
            writeln!(output, "  br label %{}", label(*target)).unwrap();
        }

        Instruction::Store { ty, value, slot } => {
            writeln!(
                output,
                "  store {} {}, ptr %primer_{}",
                type_name(*ty, module),
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
                type_name(*ty, module),
                slot_by_id(module, *slot).name,
            )
            .unwrap();
        }

        Instruction::InsertValue {
            dest,
            ty,
            aggregate,
            value_ty,
            value,
            field,
        } => {
            writeln!(
                output,
                "  {} = insertvalue {} {}, {} {}, {}",
                temp(*dest),
                type_name(*ty, module),
                operand(*aggregate),
                type_name(*value_ty, module),
                operand(*value),
                field,
            )
            .unwrap();
        }

        Instruction::ExtractValue {
            dest,
            ty,
            aggregate,
            field,
        } => {
            writeln!(
                output,
                "  {} = extractvalue {} {}, {}",
                temp(*dest),
                type_name(*ty, module),
                operand(*aggregate),
                field,
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
                type_name(*ty, module),
                operand(*left),
                operand(*right),
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
                "  {} = {} {} {}, {}",
                temp(*dest),
                compare_name(*op, *operand_ty),
                type_name(*operand_ty, module),
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
                type_name(*ty, module),
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
                type_name(*arg_ty, module),
                operand(*value),
            )
            .unwrap();
        }

        Instruction::SelectBoolText { dest, value } => {
            writeln!(
                output,
                "  {} = select i1 {}, ptr @.bool_true, ptr @.bool_false",
                temp(*dest),
                operand(*value),
            )
            .unwrap();
        }

        Instruction::CallPuts { value } => {
            writeln!(output, "  call i32 @puts(ptr {})", operand(*value)).unwrap();
        }
    }
}

fn slot_by_id(module: &Module, id: SlotId) -> &super::ir::Slot {
    &module.slots[id.0]
}

fn temp(temp: Temp) -> String {
    format!("%tmp{}", temp.0)
}

fn label(label: Label) -> String {
    format!("block{}", label.0)
}

fn type_name(ty: Type, module: &Module) -> String {
    match ty {
        Type::Bool => "i1".into(),
        Type::I64 => "i64".into(),
        Type::Float => "float".into(),
        Type::Double => "double".into(),
        Type::Named(id) => {
            let definition = &module.type_definitions[id];
            format!("%primer.type.{}.{}", definition.name, id)
        }
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
        BinaryOp::Xor => "xor",
    }
}

fn compare_name(op: CompareOp, ty: Type) -> &'static str {
    match (op, ty) {
        (CompareOp::Equal, Type::Bool | Type::I64) => "icmp eq",
        (CompareOp::NotEqual, Type::Bool | Type::I64) => "icmp ne",
        (CompareOp::Less, Type::I64) => "icmp slt",
        (CompareOp::LessEqual, Type::I64) => "icmp sle",
        (CompareOp::Greater, Type::I64) => "icmp sgt",
        (CompareOp::GreaterEqual, Type::I64) => "icmp sge",

        (CompareOp::Equal, Type::Float | Type::Double) => "fcmp oeq",
        (CompareOp::NotEqual, Type::Float | Type::Double) => "fcmp une",
        (CompareOp::Less, Type::Float | Type::Double) => "fcmp olt",
        (CompareOp::LessEqual, Type::Float | Type::Double) => "fcmp ole",
        (CompareOp::Greater, Type::Float | Type::Double) => "fcmp ogt",
        (CompareOp::GreaterEqual, Type::Float | Type::Double) => "fcmp oge",

        (
            CompareOp::Less | CompareOp::LessEqual | CompareOp::Greater | CompareOp::GreaterEqual,
            Type::Bool,
        ) => {
            unreachable!("semantic analysis rejects boolean ordering")
        }
        (_, Type::Named(_)) => unreachable!("semantic analysis rejects aggregate comparison"),
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
        Operand::Boolean(value) => i32::from(value).to_string(),

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
        Operand::Poison => "poison".into(),
    }
}

fn uses_bool_print(module: &Module) -> bool {
    module
        .instructions
        .iter()
        .any(|instruction| matches!(instruction, Instruction::CallPuts { .. }))
}
