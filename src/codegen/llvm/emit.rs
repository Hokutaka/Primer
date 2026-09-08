use std::fmt::Write;

use super::ir::{
    BinaryOp, CompareOp, Function, Instruction, Label, Module, Operand, Origin, PrintFormat, Slot,
    SlotId, Temp, Type,
};

pub fn emit(module: &Module) -> String {
    emit_with_origins(module, false)
}

pub fn emit_with_origins(module: &Module, annotate_origins: bool) -> String {
    let mut output = String::new();
    if annotate_origins {
        output.push_str("; primer-origins v1: UTF-8 byte ranges, end exclusive\n");
    }
    emit_origin(Origin::Synthetic, annotate_origins, &mut output);
    let i64_operations = i64_operations(module);
    if let Some(target) = module.target {
        writeln!(output, "target triple = \"{}\"\n", target.triple()).unwrap();
    }
    if module.uses_strings {
        super::string::emit_data(module, &mut output);
    }

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
    let array_set_types = array_set_types(module);
    if !array_types.is_empty() || i64_operations.any() {
        output.push_str("declare void @llvm.trap()\n");
    }

    if i64_operations.add {
        output.push_str("declare { i64, i1 } @llvm.sadd.with.overflow.i64(i64, i64)\n");
    }
    if i64_operations.subtract {
        output.push_str("declare { i64, i1 } @llvm.ssub.with.overflow.i64(i64, i64)\n");
    }
    if i64_operations.multiply {
        output.push_str("declare { i64, i1 } @llvm.smul.with.overflow.i64(i64, i64)\n");
    }

    output.push('\n');

    emit_i64_operation_support(i64_operations, &mut output);
    if module.uses_strings {
        super::string::emit_support(module, &mut output);
    }

    for ty in &array_types {
        emit_array_get(ty, module, &mut output);
        output.push('\n');
        if array_set_types.contains(ty) {
            emit_array_set(ty, module, &mut output);
            output.push('\n');
        }
    }

    for function in &module.functions {
        emit_function(function, module, annotate_origins, &mut output);
        output.push('\n');
    }

    emit_origin(Origin::Synthetic, annotate_origins, &mut output);
    output.push_str("define i32 @main() {\nentry:\n");
    if module.uses_strings && module.target == Some(super::Target::X86_64PcWindowsMsvc) {
        // CRTの標準出力(記述子1)を、最初の出力より前にバイナリモードにします。
        output.push_str("  %stdout.mode = call i32 @_setmode(i32 1, i32 32768)\n");
        output.push_str("  %stdout.failed = icmp eq i32 %stdout.mode, -1\n");
        output.push_str("  br i1 %stdout.failed, label %stdout_error, label %stdout_ready\n");
        output.push_str("stdout_error:\n  ret i32 1\nstdout_ready:\n");
    }

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
        emit_origin(instruction.origin, annotate_origins, &mut output);
        emit_instruction(&instruction.instruction, &module.slots, module, &mut output);
    }

    emit_origin(Origin::Synthetic, annotate_origins, &mut output);
    if let Some(function_id) = module.explicit_main {
        let function = &module.functions[function_id];
        writeln!(output, "  call void @{}()", function_name(function)).unwrap();
    }

    output.push_str("  ret i32 0\n");
    output.push_str("}\n");

    output
}

fn emit_function(
    function: &Function,
    module: &Module,
    annotate_origins: bool,
    output: &mut String,
) {
    emit_origin(Origin::Synthetic, annotate_origins, output);
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
        emit_origin(instruction.origin, annotate_origins, output);
        emit_instruction(&instruction.instruction, &function.slots, module, output);
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
        Instruction::PrintString { value } => {
            writeln!(
                output,
                "  call void @primer.print.string(%primer.string {})",
                operand(*value)
            )
            .unwrap();
        }
        Instruction::ConvertNumeric {
            dest,
            value,
            conversion,
        } => {
            writeln!(
                output,
                "  {} = call {} @{}({} {})",
                temp(*dest),
                super::conversion::type_name(conversion.to),
                conversion.helper(),
                super::conversion::type_name(conversion.from),
                operand(*value)
            )
            .unwrap();
        }
        Instruction::IntegerBinary {
            dest,
            op,
            ty,
            left,
            right,
        } => {
            writeln!(
                output,
                "  {} = call i64 @{}(i64 {}, i64 {})",
                temp(*dest),
                op.helper(*ty),
                operand(*left),
                operand(*right)
            )
            .unwrap();
        }
        Instruction::CheckIntegerRange { dest, value, ty } => {
            writeln!(
                output,
                "  {} = call i64 @primer_check_{}(i64 {})",
                temp(*dest),
                ty.name(),
                operand(*value)
            )
            .unwrap();
        }
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

        Instruction::ArraySet {
            dest,
            element,
            length,
            array,
            index,
            value,
        } => {
            let array_ty = Type::Array {
                element: Box::new(element.clone()),
                length: *length,
            };
            writeln!(
                output,
                "  {} = call {} @{}({} {}, i64 {}, {} {})",
                temp(*dest),
                type_name(&array_ty, module),
                array_set_name(element, *length, module),
                type_name(&array_ty, module),
                operand(*array),
                operand(*index),
                type_name(element, module),
                operand(*value),
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
            if let Some(helper) = checked_i64_helper(*op) {
                writeln!(
                    output,
                    "  {} = call i64 @{helper}(i64 {}, i64 {})",
                    temp(*dest),
                    operand(*left),
                    operand(*right),
                )
                .unwrap();
            } else {
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
        }

        Instruction::Compare {
            dest,
            op,
            operand_ty,
            left,
            right,
        } => {
            if *operand_ty == Type::String {
                let name = temp(*dest);
                let result = if *op == CompareOp::NotEqual {
                    format!("{name}.equal")
                } else {
                    assert_eq!(*op, CompareOp::Equal);
                    name.clone()
                };
                writeln!(output, "  {result} = call i1 @primer.string.equal(%primer.string {}, %primer.string {})", operand(*left), operand(*right)).unwrap();
                if *op == CompareOp::NotEqual {
                    writeln!(output, "  {name} = xor i1 {result}, true").unwrap();
                }
                return;
            }
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
        Type::String => "%primer.string".into(),
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
        BinaryOp::CheckedI64Add
        | BinaryOp::CheckedI64Sub
        | BinaryOp::CheckedI64Mul
        | BinaryOp::CheckedI64Div => {
            unreachable!("checked integer operations are emitted as helper calls")
        }
        BinaryOp::FAdd => "fadd",
        BinaryOp::FSub => "fsub",
        BinaryOp::FMul => "fmul",
        BinaryOp::FDiv => "fdiv",
        BinaryOp::Xor => "xor",
    }
}

fn checked_i64_helper(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::CheckedI64Add => Some("primer_i64_add"),
        BinaryOp::CheckedI64Sub => Some("primer_i64_sub"),
        BinaryOp::CheckedI64Mul => Some("primer_i64_mul"),
        BinaryOp::CheckedI64Div => Some("primer_i64_div"),
        BinaryOp::FAdd | BinaryOp::FSub | BinaryOp::FMul | BinaryOp::FDiv | BinaryOp::Xor => None,
    }
}

#[derive(Clone, Default)]
struct I64Operations {
    conversions: std::collections::BTreeSet<crate::codegen::NumericConversion>,
    integer_binary:
        std::collections::BTreeSet<(crate::codegen::IntegerBinaryOp, crate::types::IntegerType)>,
    range_checks: std::collections::BTreeSet<crate::types::IntegerType>,
    add: bool,
    subtract: bool,
    multiply: bool,
    divide: bool,
}

impl I64Operations {
    fn any(&self) -> bool {
        !self.conversions.is_empty()
            || !self.integer_binary.is_empty()
            || !self.range_checks.is_empty()
            || self.add
            || self.subtract
            || self.multiply
            || self.divide
    }

    fn include(&mut self, instruction: &Instruction) {
        if let Instruction::ConvertNumeric { conversion, .. } = instruction {
            self.conversions.insert(*conversion);
        }
        if let Instruction::IntegerBinary { op, ty, .. } = instruction {
            self.integer_binary.insert((*op, *ty));
        }
        if let Instruction::CheckIntegerRange { ty, .. } = instruction
            && *ty != crate::types::IntegerType::I64
        {
            self.range_checks.insert(*ty);
        }
        let Instruction::Binary { op, .. } = instruction else {
            return;
        };
        match op {
            BinaryOp::CheckedI64Add => self.add = true,
            BinaryOp::CheckedI64Sub => self.subtract = true,
            BinaryOp::CheckedI64Mul => self.multiply = true,
            BinaryOp::CheckedI64Div => self.divide = true,
            BinaryOp::FAdd | BinaryOp::FSub | BinaryOp::FMul | BinaryOp::FDiv | BinaryOp::Xor => {}
        }
    }
}

fn i64_operations(module: &Module) -> I64Operations {
    let mut operations = I64Operations::default();
    for instruction in &module.instructions {
        operations.include(&instruction.instruction);
    }
    for function in &module.functions {
        for instruction in &function.instructions {
            operations.include(&instruction.instruction);
        }
    }
    operations
}

fn emit_i64_operation_support(operations: I64Operations, output: &mut String) {
    for &conversion in &operations.conversions {
        super::conversion::emit_support(conversion, output);
    }
    for &(op, ty) in &operations.integer_binary {
        super::integer::emit_support(op, ty, output);
    }
    for ty in &operations.range_checks {
        output.push_str(&format!("define internal i64 @primer_check_{}(i64 %value) {{\nentry:\n  %below = icmp slt i64 %value, {}\n  %above = icmp sgt i64 %value, {}\n  %bad = or i1 %below, %above\n  br i1 %bad, label %trap, label %ok\ntrap:\n  call void @llvm.trap()\n  unreachable\nok:\n  ret i64 %value\n}}\n\n", ty.name(), ty.minimum(), ty.maximum()));
    }

    for (enabled, name, intrinsic) in [
        (operations.add, "add", "sadd"),
        (operations.subtract, "sub", "ssub"),
        (operations.multiply, "mul", "smul"),
    ] {
        if !enabled {
            continue;
        }
        writeln!(
            output,
            "define internal i64 @primer_i64_{name}(i64 %left, i64 %right) {{"
        )
        .unwrap();
        output.push_str("entry:\n");
        writeln!(
            output,
            "  %checked = call {{ i64, i1 }} @llvm.{intrinsic}.with.overflow.i64(i64 %left, i64 %right)"
        )
        .unwrap();
        output.push_str("  %result = extractvalue { i64, i1 } %checked, 0\n");
        output.push_str("  %overflow = extractvalue { i64, i1 } %checked, 1\n");
        output.push_str("  br i1 %overflow, label %trap, label %ok\n\n");
        output.push_str("trap:\n  call void @llvm.trap()\n  unreachable\n\n");
        output.push_str("ok:\n  ret i64 %result\n}\n\n");
    }

    if operations.divide {
        output.push_str("define internal i64 @primer_i64_div(i64 %left, i64 %right) {\n");
        output.push_str("entry:\n");
        output.push_str("  %is_zero = icmp eq i64 %right, 0\n");
        output.push_str("  %is_min = icmp eq i64 %left, -9223372036854775808\n");
        output.push_str("  %is_negative_one = icmp eq i64 %right, -1\n");
        output.push_str("  %overflows = and i1 %is_min, %is_negative_one\n");
        output.push_str("  %invalid = or i1 %is_zero, %overflows\n");
        output.push_str("  br i1 %invalid, label %trap, label %ok\n\n");
        output.push_str("trap:\n  call void @llvm.trap()\n  unreachable\n\n");
        output.push_str("ok:\n  %result = sdiv i64 %left, %right\n  ret i64 %result\n}\n\n");
    }
}

fn compare_name(op: CompareOp, ty: &Type) -> &'static str {
    match (op, ty) {
        (_, Type::String) => unreachable!("string comparisons use a content helper"),
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
        Operand::String { id, length } => format!("{{ ptr @primer.string.{id}, i64 {length} }}"),
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
        .any(|instruction| matches!(instruction.instruction, Instruction::CallPuts { .. }))
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
        match &instruction.instruction {
            Instruction::ArrayGet {
                element, length, ..
            }
            | Instruction::ArraySet {
                element, length, ..
            } => {
                add(
                    &Type::Array {
                        element: Box::new(element.clone()),
                        length: *length,
                    },
                    &mut result,
                );
            }
            _ => {}
        }
    }
    result
}

fn array_set_types(module: &Module) -> Vec<Type> {
    let mut result = Vec::new();
    for instruction in module.instructions.iter().chain(
        module
            .functions
            .iter()
            .flat_map(|function| function.instructions.iter()),
    ) {
        if let Instruction::ArraySet {
            element, length, ..
        } = &instruction.instruction
        {
            let ty = Type::Array {
                element: Box::new(element.clone()),
                length: *length,
            };
            if !result.contains(&ty) {
                result.push(ty);
            }
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

fn emit_array_set(ty: &Type, module: &Module, output: &mut String) {
    let Type::Array { element, length } = ty else {
        unreachable!("array setter requires an array type")
    };
    let element_ty = type_name(element, module);
    let array_ty = format!("[{length} x {element_ty}]");
    writeln!(
        output,
        "define internal {array_ty} @{}({array_ty} %value, i64 %index, {element_ty} %replacement) {{",
        array_set_name(element, *length, module)
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
    writeln!(output, "  store {element_ty} %replacement, ptr %element").unwrap();
    writeln!(output, "  %result = load {array_ty}, ptr %array").unwrap();
    writeln!(output, "  ret {array_ty} %result").unwrap();
    output.push_str("}\n");
}

fn array_get_name(element: &Type, length: usize, module: &Module) -> String {
    format!(
        "primer.array.get.{}.{length}",
        array_element_name(element, module)
    )
}

fn array_set_name(element: &Type, length: usize, module: &Module) -> String {
    format!(
        "primer.array.set.{}.{length}",
        array_element_name(element, module)
    )
}

fn array_element_name(element: &Type, module: &Module) -> String {
    match element {
        Type::String => "string".into(),
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

fn emit_origin(origin: Origin, enabled: bool, output: &mut String) {
    if !enabled {
        return;
    }
    match origin {
        Origin::Source { node_id, span } => writeln!(
            output,
            "; primer-origin: #{} bytes {}..{}",
            node_id.0,
            span.start(),
            span.end()
        )
        .unwrap(),
        Origin::Synthetic => output.push_str("; primer-origin: synthetic\n"),
    }
}
