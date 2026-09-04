use std::fmt::Write;

use super::ir::{
    BinaryOp, CompareOp, Function, Instruction, Module, Operand, ParameterPassing, PrintFormat,
    Slot, Temp, Type,
};

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

    for function in &module.functions {
        emit_function(function, module, &mut output);
        output.push('\n');
    }

    output.push_str("export function w $main() {\n");
    output.push_str("@start\n");

    for slot in &module.slots {
        writeln!(output, "  %slot_{} =l alloc8 {}", slot.name, slot.size).unwrap();
    }

    for instruction in &module.instructions {
        emit_instruction(instruction, &module.slots, module, &mut output);
    }

    if let Some(function_id) = module.explicit_main {
        writeln!(
            output,
            "  call ${}()",
            function_name(&module.functions[function_id])
        )
        .unwrap();
    }

    output.push_str("  ret 0\n");
    output.push_str("}\n");

    output
}

fn emit_function(function: &Function, module: &Module, output: &mut String) {
    output.push_str("function ");
    if let Some(return_type) = function.return_type {
        write!(output, "{} ", type_name(return_type)).unwrap();
    }
    write!(output, "${}(", function_name(function)).unwrap();
    let mut has_parameter = false;
    if function.aggregate_return_size.is_some() {
        output.push_str("l %result");
        has_parameter = true;
    }
    for (index, parameter) in function.parameters.iter().enumerate() {
        if has_parameter {
            output.push_str(", ");
        }
        let ty = match parameter.passing {
            ParameterPassing::Scalar(ty) => ty,
            ParameterPassing::Aggregate { .. } => Type::Pointer,
        };
        write!(output, "{} %arg{index}", type_name(ty)).unwrap();
        has_parameter = true;
    }
    output.push_str(") {\n@start\n");

    for slot in &function.slots {
        writeln!(output, "  %slot_{} =l alloc8 {}", slot.name, slot.size).unwrap();
    }
    for (index, parameter) in function.parameters.iter().enumerate() {
        match parameter.passing {
            ParameterPassing::Scalar(ty) => {
                writeln!(
                    output,
                    "  {} %arg{index}, %slot_{}",
                    store_name(ty),
                    function.slots[parameter.slot].name
                )
                .unwrap();
            }
            ParameterPassing::Aggregate { size } => {
                writeln!(
                    output,
                    "  blit %arg{index}, %slot_{}, {size}",
                    function.slots[parameter.slot].name
                )
                .unwrap();
            }
        }
    }
    for instruction in &function.instructions {
        emit_instruction(instruction, &function.slots, module, output);
    }
    output.push_str("}\n");
}

fn function_name(function: &Function) -> String {
    format!("primer_fn_{}_{}", function.name, function.id)
}

fn emit_instruction(
    instruction: &Instruction,
    slots: &[Slot],
    module: &Module,
    output: &mut String,
) {
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
                operand(condition, slots)
            )
            .unwrap();
        }

        Instruction::Jump(label) => {
            writeln!(output, "  jmp @block{label}").unwrap();
        }

        Instruction::Store { address, ty, value } => {
            writeln!(
                output,
                "  {} {}, {}",
                store_name(*ty),
                operand(value, slots),
                operand(address, slots),
            )
            .unwrap();
        }

        Instruction::Load { dest, address, ty } => {
            writeln!(
                output,
                "  {} ={} {} {}",
                temp(*dest),
                type_name(*ty),
                load_name(*ty),
                operand(address, slots),
            )
            .unwrap();
        }

        Instruction::Address { dest, base, offset } => {
            writeln!(
                output,
                "  {} =l add {}, {}",
                temp(*dest),
                operand(base, slots),
                offset
            )
            .unwrap();
        }

        Instruction::Blit {
            source,
            destination,
            size,
        } => {
            writeln!(
                output,
                "  blit {}, {}, {size}",
                operand(source, slots),
                operand(destination, slots)
            )
            .unwrap();
        }

        Instruction::Abort => {
            output.push_str("  call $abort()\n");
            output.push_str("  hlt\n");
        }

        Instruction::Call {
            dest,
            function_id,
            return_type,
            arguments,
        } => {
            output.push_str("  ");
            if let (Some(dest), Some(return_type)) = (dest, return_type) {
                write!(output, "{} ={} ", temp(*dest), type_name(*return_type)).unwrap();
            }
            write!(
                output,
                "call ${}(",
                function_name(&module.functions[*function_id])
            )
            .unwrap();
            for (index, (ty, argument)) in arguments.iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                write!(output, "{} {}", type_name(*ty), operand(argument, slots)).unwrap();
            }
            output.push_str(")\n");
        }

        Instruction::Return { value } => match value {
            Some((_, value)) => {
                writeln!(output, "  ret {}", operand(value, slots)).unwrap();
            }
            None => output.push_str("  ret\n"),
        },

        Instruction::Negate { dest, ty, value } => {
            writeln!(
                output,
                "  {} ={} neg {}",
                temp(*dest),
                type_name(*ty),
                operand(value, slots),
            )
            .unwrap();
        }

        Instruction::Not { dest, value } => {
            writeln!(
                output,
                "  {} =w ceqw {}, 0",
                temp(*dest),
                operand(value, slots)
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
                operand(left, slots),
                operand(right, slots),
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
                operand(left, slots),
                operand(right, slots),
            )
            .unwrap();
        }

        Instruction::ExtendSingleToDouble { dest, value } => {
            writeln!(
                output,
                "  {} =d exts {}",
                temp(*dest),
                operand(value, slots)
            )
            .unwrap();
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
                operand(value, slots),
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
            writeln!(
                output,
                "  {} =l extsw {}",
                temp(*offset),
                operand(value, slots)
            )
            .unwrap();
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
        Type::Pointer => "l",
    }
}

fn store_name(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "storew",
        Type::I64 => "storel",
        Type::Single => "stores",
        Type::Double => "stored",
        Type::Pointer => unreachable!("pointers are passed without scalar stores"),
    }
}

fn load_name(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "loadw",
        Type::I64 => "loadl",
        Type::Single => "loads",
        Type::Double => "loadd",
        Type::Pointer => unreachable!("pointers are passed without scalar loads"),
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
        (_, Type::Pointer) => unreachable!("pointers are not Primer comparison operands"),
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

fn operand(value: &Operand, slots: &[Slot]) -> String {
    match value {
        Operand::Boolean(value) => i32::from(*value).to_string(),
        Operand::Integer(value) => value.to_string(),
        Operand::Float32(text) => format!("s_{text}"),
        Operand::Float64(text) => format!("d_{text}"),
        Operand::Temp(temp) => self::temp(*temp),
        Operand::Slot(slot) => format!("%slot_{}", slots[*slot].name),
        Operand::ReturnPointer => "%result".into(),
    }
}

fn temp(temp: Temp) -> String {
    format!("%tmp{}", temp.0)
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
        .any(|instruction| matches!(instruction, Instruction::CallPrintBool { .. }))
}
