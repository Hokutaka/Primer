use std::fmt::Write;

use super::ir::{
    BinaryOp, CompareOp, Function, Instruction, Label, Module, Operand, PrintFormat, Slot, SlotId,
    Temp, Type,
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
            output.push_str(&type_name(field, module));
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

    let array_types = array_types(module);
    if !array_types.is_empty() {
        output.push_str("declare void @llvm.trap()\n");
    }

    output.push('\n');

    for ty in &array_types {
        emit_array_get(ty, module, &mut output);
        output.push('\n');
    }

    for function in &module.functions {
        emit_function(function, module, &mut output);
        output.push('\n');
    }

    output.push_str("define i32 @main() {\nentry:\n");

    for slot in &module.slots {
        writeln!(
            output,
            "  %primer_{} = alloca {}",
            slot.name,
            type_name(&slot.ty, module),
        )
        .unwrap();
    }

    for instruction in &module.instructions {
        emit_instruction(instruction, &module.slots, module, &mut output);
    }

    if let Some(function_id) = module.explicit_main {
        let function = &module.functions[function_id];
        writeln!(output, "  call void @{}()", function_name(function)).unwrap();
    }

    output.push_str("  ret i32 0\n");
    output.push_str("}\n");

    output
}

fn emit_function(function: &Function, module: &Module, output: &mut String) {
    write!(
        output,
        "define {} @{}(",
        function
            .return_type
            .as_ref()
            .map_or_else(|| "void".into(), |ty| type_name(ty, module)),
        function_name(function)
    )
    .unwrap();
    for (index, parameter) in function.parameters.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write!(output, "{} %arg{index}", type_name(&parameter.ty, module)).unwrap();
    }
    output.push_str(") {\nentry:\n");

    for slot in &function.slots {
        writeln!(
            output,
            "  %primer_{} = alloca {}",
            slot.name,
            type_name(&slot.ty, module),
        )
        .unwrap();
    }
    for (index, parameter) in function.parameters.iter().enumerate() {
        writeln!(
            output,
            "  store {} %arg{index}, ptr %primer_{}",
            type_name(&parameter.ty, module),
            slot_by_id(&function.slots, parameter.slot).name,
        )
        .unwrap();
    }
    for instruction in &function.instructions {
        emit_instruction(instruction, &function.slots, module, output);
    }
    output.push_str("}\n");
}

fn emit_instruction(
    instruction: &Instruction,
    slots: &[Slot],
    module: &Module,
    output: &mut String,
) {
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
                type_name(ty, module),
                operand(*value),
                slot_by_id(slots, *slot).name,
            )
            .unwrap();
        }

        Instruction::Load { dest, ty, slot } => {
            writeln!(
                output,
                "  {} = load {}, ptr %primer_{}",
                temp(*dest),
                type_name(ty, module),
                slot_by_id(slots, *slot).name,
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
                type_name(ty, module),
                operand(*aggregate),
                type_name(value_ty, module),
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
                type_name(ty, module),
                operand(*aggregate),
                field,
            )
            .unwrap();
        }

        Instruction::ArrayGet {
            dest,
            element,
            length,
            array,
            index,
        } => {
            let array_ty = Type::Array {
                element: Box::new(element.clone()),
                length: *length,
            };
            writeln!(
                output,
                "  {} = call {} @{}({} {}, i64 {})",
                temp(*dest),
                type_name(element, module),
                array_get_name(element, *length, module),
                type_name(&array_ty, module),
                operand(*array),
                operand(*index),
            )
            .unwrap();
        }

        Instruction::Call {
            dest,
            function_id,
            return_type,
            arguments,
        } => {
            output.push_str("  ");
            if let Some(dest) = dest {
                write!(output, "{} = ", temp(*dest)).unwrap();
            }
            let function = &module.functions[*function_id];
            write!(
                output,
                "call {} @{}(",
                return_type
                    .as_ref()
                    .map_or_else(|| "void".into(), |ty| type_name(ty, module)),
                function_name(function)
            )
            .unwrap();
            for (index, (ty, argument)) in arguments.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                write!(output, "{} {}", type_name(ty, module), operand(*argument)).unwrap();
            }
            output.push_str(")\n");
        }

        Instruction::Return { value } => match value {
            Some((ty, value)) => {
                writeln!(
                    output,
                    "  ret {} {}",
                    type_name(ty, module),
                    operand(*value)
                )
                .unwrap();
            }
            None => output.push_str("  ret void\n"),
        },

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
                type_name(ty, module),
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
                compare_name(*op, operand_ty),
                type_name(operand_ty, module),
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
                type_name(ty, module),
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
                type_name(arg_ty, module),
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

fn slot_by_id(slots: &[Slot], id: SlotId) -> &Slot {
    &slots[id.0]
}

fn function_name(function: &Function) -> String {
    format!("primer.fn.{}.{}", function.name, function.id)
}

fn temp(temp: Temp) -> String {
    format!("%tmp{}", temp.0)
}

fn label(label: Label) -> String {
    format!("block{}", label.0)
}

fn type_name(ty: &Type, module: &Module) -> String {
    match ty {
        Type::Bool => "i1".into(),
        Type::I64 => "i64".into(),
        Type::Float => "float".into(),
        Type::Double => "double".into(),
        Type::Named(id) => {
            let definition = &module.type_definitions[*id];
            format!("%primer.type.{}.{}", definition.name, id)
        }
        Type::Array { element, length } => {
            format!("[{length} x {}]", type_name(element, module))
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

fn compare_name(op: CompareOp, ty: &Type) -> &'static str {
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
        (_, Type::Named(_) | Type::Array { .. }) => {
            unreachable!("semantic analysis rejects aggregate comparison")
        }
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
        .chain(
            module
                .functions
                .iter()
                .flat_map(|function| function.instructions.iter()),
        )
        .any(|instruction| matches!(instruction, Instruction::CallPuts { .. }))
}

fn array_types(module: &Module) -> Vec<Type> {
    fn add(ty: &Type, result: &mut Vec<Type>) {
        let Type::Array { element, .. } = ty else {
            return;
        };
        add(element, result);
        if !result.contains(ty) {
            result.push(ty.clone());
        }
    }

    let mut result = Vec::new();
    for ty in module
        .slots
        .iter()
        .map(|slot| &slot.ty)
        .chain(module.functions.iter().flat_map(|function| {
            function
                .slots
                .iter()
                .map(|slot| &slot.ty)
                .chain(function.return_type.iter())
        }))
    {
        add(ty, &mut result);
    }
    for instruction in module.instructions.iter().chain(
        module
            .functions
            .iter()
            .flat_map(|function| function.instructions.iter()),
    ) {
        if let Instruction::ArrayGet {
            element, length, ..
        } = instruction
        {
            add(
                &Type::Array {
                    element: Box::new(element.clone()),
                    length: *length,
                },
                &mut result,
            );
        }
    }
    result
}

fn emit_array_get(ty: &Type, module: &Module, output: &mut String) {
    let Type::Array { element, length } = ty else {
        unreachable!("array getter requires an array type")
    };
    let element_ty = type_name(element, module);
    let array_ty = format!("[{length} x {element_ty}]");
    writeln!(
        output,
        "define internal {element_ty} @{}({array_ty} %value, i64 %index) {{",
        array_get_name(element, *length, module)
    )
    .unwrap();
    output.push_str("entry:\n");
    output.push_str("  %index.low = icmp slt i64 %index, 0\n");
    writeln!(output, "  %index.high = icmp sge i64 %index, {length}").unwrap();
    output.push_str("  %index.outside = or i1 %index.low, %index.high\n");
    output.push_str("  br i1 %index.outside, label %out_of_bounds, label %in_bounds\n");
    output.push_str("out_of_bounds:\n");
    output.push_str("  call void @llvm.trap()\n");
    output.push_str("  unreachable\n");
    output.push_str("in_bounds:\n");
    writeln!(output, "  %array = alloca {array_ty}").unwrap();
    writeln!(output, "  store {array_ty} %value, ptr %array").unwrap();
    writeln!(
        output,
        "  %element = getelementptr inbounds {array_ty}, ptr %array, i64 0, i64 %index"
    )
    .unwrap();
    writeln!(output, "  %result = load {element_ty}, ptr %element").unwrap();
    writeln!(output, "  ret {element_ty} %result").unwrap();
    output.push_str("}\n");
}

fn array_get_name(element: &Type, length: usize, module: &Module) -> String {
    format!(
        "primer.array.get.{}.{length}",
        array_element_name(element, module)
    )
}

fn array_element_name(element: &Type, module: &Module) -> String {
    match element {
        Type::Bool => "bool".into(),
        Type::I64 => "i64".into(),
        Type::Float => "f32".into(),
        Type::Double => "f64".into(),
        Type::Named(id) => {
            let definition = &module.type_definitions[*id];
            format!("type.{}.{}", definition.name, id)
        }
        Type::Array { element, length } => {
            format!("array.{}.{length}", array_element_name(element, module))
        }
    }
}
