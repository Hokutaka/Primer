use std::fmt::Write;

use super::ir::{Function, Instruction, LoopKind, Module, Type};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();
    let i64_operations = i64_operations(module);

    writeln!(output, "(module").unwrap();

    // print() is provided by the host.
    if module_uses_bool_print(module) {
        writeln!(
            output,
            "  (import \"primer\" \"print_bool\" (func $print_bool (param i32)))"
        )
        .unwrap();
    }

    writeln!(
        output,
        "  (import \"primer\" \"print_i64\" (func $print_i64 (param i64)))"
    )
    .unwrap();

    writeln!(
        output,
        "  (import \"primer\" \"print_f32\" (func $print_f32 (param f32)))"
    )
    .unwrap();

    writeln!(
        output,
        "  (import \"primer\" \"print_f64\" (func $print_f64 (param f64)))"
    )
    .unwrap();

    writeln!(output).unwrap();

    emit_i64_operation_support(i64_operations, &mut output);

    if module.memory_pages > 0 {
        writeln!(output, "  (memory {})", module.memory_pages).unwrap();
        writeln!(output).unwrap();
    }

    for function in &module.functions {
        emit_function(function, module, &mut output);
        writeln!(output).unwrap();
    }

    writeln!(output, "  (func $main").unwrap();

    for local in &module.locals {
        writeln!(
            output,
            "    (local $primer_{} {})",
            local.name,
            wat_type(local.ty),
        )
        .unwrap();
    }

    if !module.locals.is_empty() {
        writeln!(output).unwrap();
    }

    for instruction in &module.instructions {
        emit_instruction(instruction, 2, module, &mut output);
    }

    if let Some(function_id) = module.explicit_main {
        writeln!(
            output,
            "    call ${}",
            function_name(&module.functions[function_id])
        )
        .unwrap();
    }

    writeln!(output, "  )").unwrap();

    writeln!(output, "  (export \"main\" (func $main))").unwrap();

    writeln!(output, ")").unwrap();

    output
}

#[derive(Clone, Default)]
struct I64Operations {
    integer_binary:
        std::collections::BTreeSet<(crate::codegen::IntegerBinaryOp, crate::types::IntegerType)>,
    range_checks: std::collections::BTreeSet<crate::types::IntegerType>,
    add: bool,
    subtract: bool,
    multiply: bool,
}

impl I64Operations {
    fn include(&mut self, instruction: &Instruction) {
        if let Instruction::IntegerBinary { op, ty, .. } = instruction {
            self.integer_binary.insert((*op, *ty));
        }
        match instruction {
            Instruction::CheckIntegerRange(ty) => {
                if *ty != crate::types::IntegerType::I64 {
                    self.range_checks.insert(*ty);
                }
            }
            Instruction::CheckedI64Add => self.add = true,
            Instruction::CheckedI64Sub => self.subtract = true,
            Instruction::CheckedI64Mul => self.multiply = true,
            Instruction::If {
                then_instructions,
                else_instructions,
            }
            | Instruction::IfBool {
                then_instructions,
                else_instructions,
            } => {
                for instruction in then_instructions.iter().chain(else_instructions) {
                    self.include(instruction);
                }
            }
            Instruction::Loop {
                condition_instructions,
                body_instructions,
                update_instructions,
                ..
            } => {
                for instruction in condition_instructions
                    .iter()
                    .chain(body_instructions)
                    .chain(update_instructions)
                {
                    self.include(instruction);
                }
            }
            _ => {}
        }
    }
}

fn i64_operations(module: &Module) -> I64Operations {
    let mut operations = I64Operations::default();
    for instruction in &module.instructions {
        operations.include(instruction);
    }
    for function in &module.functions {
        for instruction in &function.instructions {
            operations.include(instruction);
        }
    }
    operations
}

fn emit_i64_operation_support(operations: I64Operations, output: &mut String) {
    for &(op, ty) in &operations.integer_binary {
        super::integer::emit_support(op, ty, output);
    }
    for ty in &operations.range_checks {
        output.push_str(&format!("  (func $primer_check_{} (param $value i64) (result i64)\n    local.get $value\n    i64.const {}\n    i64.lt_s\n    local.get $value\n    i64.const {}\n    i64.gt_s\n    i32.or\n    if\n      unreachable\n    end\n    local.get $value\n  )\n\n", ty.name(), ty.minimum(), ty.maximum()));
    }

    if operations.add {
        output.push_str(
            "  (func $primer_i64_add (param $left i64) (param $right i64) (result i64)\n\
             \x20   (local $result i64)\n\
             \x20   local.get $left\n\
             \x20   local.get $right\n\
             \x20   i64.add\n\
             \x20   local.set $result\n\
             \x20   local.get $result\n\
             \x20   local.get $left\n\
             \x20   i64.xor\n\
             \x20   local.get $result\n\
             \x20   local.get $right\n\
             \x20   i64.xor\n\
             \x20   i64.and\n\
             \x20   i64.const 0\n\
             \x20   i64.lt_s\n\
             \x20   if\n\
             \x20     unreachable\n\
             \x20   end\n\
             \x20   local.get $result\n\
             \x20 )\n\n",
        );
    }

    if operations.subtract {
        output.push_str(
            "  (func $primer_i64_sub (param $left i64) (param $right i64) (result i64)\n\
             \x20   (local $result i64)\n\
             \x20   local.get $left\n\
             \x20   local.get $right\n\
             \x20   i64.sub\n\
             \x20   local.set $result\n\
             \x20   local.get $left\n\
             \x20   local.get $right\n\
             \x20   i64.xor\n\
             \x20   local.get $left\n\
             \x20   local.get $result\n\
             \x20   i64.xor\n\
             \x20   i64.and\n\
             \x20   i64.const 0\n\
             \x20   i64.lt_s\n\
             \x20   if\n\
             \x20     unreachable\n\
             \x20   end\n\
             \x20   local.get $result\n\
             \x20 )\n\n",
        );
    }

    if operations.multiply {
        output.push_str(
            "  (func $primer_i64_mul (param $left i64) (param $right i64) (result i64)\n\
             \x20   (local $result i64)\n\
             \x20   local.get $left\n\
             \x20   i64.eqz\n\
             \x20   if\n\
             \x20     i64.const 0\n\
             \x20     return\n\
             \x20   end\n\
             \x20   local.get $left\n\
             \x20   i64.const -1\n\
             \x20   i64.eq\n\
             \x20   local.get $right\n\
             \x20   i64.const -9223372036854775808\n\
             \x20   i64.eq\n\
             \x20   i32.and\n\
             \x20   local.get $right\n\
             \x20   i64.const -1\n\
             \x20   i64.eq\n\
             \x20   local.get $left\n\
             \x20   i64.const -9223372036854775808\n\
             \x20   i64.eq\n\
             \x20   i32.and\n\
             \x20   i32.or\n\
             \x20   if\n\
             \x20     unreachable\n\
             \x20   end\n\
             \x20   local.get $left\n\
             \x20   local.get $right\n\
             \x20   i64.mul\n\
             \x20   local.set $result\n\
             \x20   local.get $result\n\
             \x20   local.get $left\n\
             \x20   i64.div_s\n\
             \x20   local.get $right\n\
             \x20   i64.ne\n\
             \x20   if\n\
             \x20     unreachable\n\
             \x20   end\n\
             \x20   local.get $result\n\
             \x20 )\n\n",
        );
    }
}

fn emit_function(function: &Function, module: &Module, output: &mut String) {
    write!(output, "  (func ${}", function_name(function)).unwrap();
    for parameter in &function.parameters {
        write!(
            output,
            " (param $primer_{} {})",
            parameter.name,
            wat_type(parameter.ty)
        )
        .unwrap();
    }
    if let Some(return_type) = function.return_type {
        write!(output, " (result {})", wat_type(return_type)).unwrap();
    }
    writeln!(output).unwrap();

    for local in &function.locals {
        writeln!(
            output,
            "    (local $primer_{} {})",
            local.name,
            wat_type(local.ty)
        )
        .unwrap();
    }
    if !function.locals.is_empty() {
        writeln!(output).unwrap();
    }
    for instruction in &function.instructions {
        emit_instruction(instruction, 2, module, output);
    }
    write!(output, "  )").unwrap();
}

fn function_name(function: &Function) -> String {
    format!("primer_fn_{}_{}", function.name, function.id)
}

fn emit_instruction(
    instruction: &Instruction,
    indent: usize,
    module: &Module,
    output: &mut String,
) {
    let prefix = "  ".repeat(indent);

    match instruction {
        Instruction::IntegerBinary { op, ty } => {
            writeln!(output, "{prefix}call ${}", op.helper(*ty)).unwrap();
        }
        Instruction::CheckIntegerRange(ty) => {
            writeln!(output, "{prefix}call $primer_check_{}", ty.name()).unwrap();
        }
        Instruction::I32Const(value) => {
            writeln!(output, "{prefix}i32.const {value}").unwrap();
        }

        Instruction::I64Const(value) => {
            writeln!(output, "{prefix}i64.const {value}").unwrap();
        }

        Instruction::F32Const(text) => {
            writeln!(output, "{prefix}f32.const {text}").unwrap();
        }

        Instruction::F64Const(text) => {
            writeln!(output, "{prefix}f64.const {text}").unwrap();
        }

        Instruction::LocalGet(name) => {
            writeln!(output, "{prefix}local.get $primer_{name}").unwrap();
        }

        Instruction::LocalSet(name) => {
            writeln!(output, "{prefix}local.set $primer_{name}").unwrap();
        }

        Instruction::I32Load { offset } => emit_memory("i32.load", *offset, &prefix, output),
        Instruction::I64Load { offset } => emit_memory("i64.load", *offset, &prefix, output),
        Instruction::F32Load { offset } => emit_memory("f32.load", *offset, &prefix, output),
        Instruction::F64Load { offset } => emit_memory("f64.load", *offset, &prefix, output),
        Instruction::I32Store { offset } => emit_memory("i32.store", *offset, &prefix, output),
        Instruction::I64Store { offset } => emit_memory("i64.store", *offset, &prefix, output),
        Instruction::F32Store { offset } => emit_memory("f32.store", *offset, &prefix, output),
        Instruction::F64Store { offset } => emit_memory("f64.store", *offset, &prefix, output),

        Instruction::I32WrapI64 => emit_simple("i32.wrap_i64", &prefix, output),
        Instruction::I32Add => emit_simple("i32.add", &prefix, output),
        Instruction::I32Mul => emit_simple("i32.mul", &prefix, output),
        Instruction::Unreachable => emit_simple("unreachable", &prefix, output),

        Instruction::If {
            then_instructions,
            else_instructions,
        }
        | Instruction::IfBool {
            then_instructions,
            else_instructions,
        } => {
            let result = if matches!(instruction, Instruction::IfBool { .. }) {
                " (result i32)"
            } else {
                ""
            };
            writeln!(output, "{prefix}if{result}").unwrap();
            for instruction in then_instructions {
                emit_instruction(instruction, indent + 1, module, output);
            }
            if !else_instructions.is_empty() {
                writeln!(output, "{prefix}else").unwrap();
                for instruction in else_instructions {
                    emit_instruction(instruction, indent + 1, module, output);
                }
            }
            writeln!(output, "{prefix}end").unwrap();
        }

        Instruction::Loop {
            kind,
            id,
            condition_instructions,
            body_instructions,
            update_instructions,
        } => {
            let name = loop_name(*kind);
            writeln!(output, "{prefix}block ${name}_end_{id}").unwrap();
            writeln!(output, "{prefix}  loop ${name}_condition_{id}").unwrap();
            for instruction in condition_instructions {
                emit_instruction(instruction, indent + 2, module, output);
            }
            writeln!(output, "{prefix}    i32.eqz").unwrap();
            writeln!(output, "{prefix}    br_if ${name}_end_{id}").unwrap();
            writeln!(output, "{prefix}    block ${name}_continue_{id}").unwrap();
            for instruction in body_instructions {
                emit_instruction(instruction, indent + 3, module, output);
            }
            writeln!(output, "{prefix}    end").unwrap();
            for instruction in update_instructions {
                emit_instruction(instruction, indent + 2, module, output);
            }
            writeln!(output, "{prefix}    br ${name}_condition_{id}").unwrap();
            writeln!(output, "{prefix}  end").unwrap();
            writeln!(output, "{prefix}end").unwrap();
        }

        Instruction::Break { kind, id } => {
            writeln!(output, "{prefix}br ${}_end_{id}", loop_name(*kind)).unwrap();
        }

        Instruction::Continue { kind, id } => {
            writeln!(output, "{prefix}br ${}_continue_{id}", loop_name(*kind)).unwrap();
        }

        Instruction::Call { function_id } => {
            writeln!(
                output,
                "{prefix}call ${}",
                function_name(&module.functions[*function_id])
            )
            .unwrap();
        }

        Instruction::Return => emit_simple("return", &prefix, output),

        Instruction::CheckedI64Add => emit_simple("call $primer_i64_add", &prefix, output),
        Instruction::CheckedI64Sub => emit_simple("call $primer_i64_sub", &prefix, output),
        Instruction::CheckedI64Mul => emit_simple("call $primer_i64_mul", &prefix, output),
        Instruction::CheckedI64DivS => emit_simple("i64.div_s", &prefix, output),
        Instruction::I64Eq => emit_simple("i64.eq", &prefix, output),
        Instruction::I64Ne => emit_simple("i64.ne", &prefix, output),
        Instruction::I64LtS => emit_simple("i64.lt_s", &prefix, output),
        Instruction::I64LeS => emit_simple("i64.le_s", &prefix, output),
        Instruction::I64GtS => emit_simple("i64.gt_s", &prefix, output),
        Instruction::I64GeS => emit_simple("i64.ge_s", &prefix, output),

        Instruction::I32Eq => emit_simple("i32.eq", &prefix, output),
        Instruction::I32Ne => emit_simple("i32.ne", &prefix, output),
        Instruction::I32Eqz => emit_simple("i32.eqz", &prefix, output),

        Instruction::F32Add => emit_simple("f32.add", &prefix, output),
        Instruction::F32Sub => emit_simple("f32.sub", &prefix, output),
        Instruction::F32Mul => emit_simple("f32.mul", &prefix, output),
        Instruction::F32Div => emit_simple("f32.div", &prefix, output),
        Instruction::F32Neg => emit_simple("f32.neg", &prefix, output),
        Instruction::F32Eq => emit_simple("f32.eq", &prefix, output),
        Instruction::F32Ne => emit_simple("f32.ne", &prefix, output),
        Instruction::F32Lt => emit_simple("f32.lt", &prefix, output),
        Instruction::F32Le => emit_simple("f32.le", &prefix, output),
        Instruction::F32Gt => emit_simple("f32.gt", &prefix, output),
        Instruction::F32Ge => emit_simple("f32.ge", &prefix, output),

        Instruction::F64Add => emit_simple("f64.add", &prefix, output),
        Instruction::F64Sub => emit_simple("f64.sub", &prefix, output),
        Instruction::F64Mul => emit_simple("f64.mul", &prefix, output),
        Instruction::F64Div => emit_simple("f64.div", &prefix, output),
        Instruction::F64Neg => emit_simple("f64.neg", &prefix, output),
        Instruction::F64Eq => emit_simple("f64.eq", &prefix, output),
        Instruction::F64Ne => emit_simple("f64.ne", &prefix, output),
        Instruction::F64Lt => emit_simple("f64.lt", &prefix, output),
        Instruction::F64Le => emit_simple("f64.le", &prefix, output),
        Instruction::F64Gt => emit_simple("f64.gt", &prefix, output),
        Instruction::F64Ge => emit_simple("f64.ge", &prefix, output),

        Instruction::CallPrint(ty) => {
            let function = match ty {
                Type::Bool => "$print_bool",
                Type::I64 => "$print_i64",
                Type::F32 => "$print_f32",
                Type::F64 => "$print_f64",
                Type::Pointer => unreachable!("pointers are not printable Primer values"),
            };

            writeln!(output, "{prefix}call {function}").unwrap();
        }
    }
}

fn emit_simple(instruction: &str, prefix: &str, output: &mut String) {
    writeln!(output, "{prefix}{instruction}").unwrap();
}

fn emit_memory(instruction: &str, offset: u32, prefix: &str, output: &mut String) {
    if offset == 0 {
        writeln!(output, "{prefix}{instruction}").unwrap();
    } else {
        writeln!(output, "{prefix}{instruction} offset={offset}").unwrap();
    }
}

fn wat_type(ty: Type) -> &'static str {
    match ty {
        Type::Bool => "i32",
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
        Type::Pointer => "i32",
    }
}

fn instruction_uses_bool_print(instruction: &Instruction) -> bool {
    match instruction {
        Instruction::CallPrint(Type::Bool) => true,
        Instruction::If {
            then_instructions,
            else_instructions,
        }
        | Instruction::IfBool {
            then_instructions,
            else_instructions,
        } => {
            then_instructions.iter().any(instruction_uses_bool_print)
                || else_instructions.iter().any(instruction_uses_bool_print)
        }
        Instruction::Loop {
            condition_instructions,
            body_instructions,
            update_instructions,
            ..
        } => {
            condition_instructions
                .iter()
                .any(instruction_uses_bool_print)
                || body_instructions.iter().any(instruction_uses_bool_print)
                || update_instructions.iter().any(instruction_uses_bool_print)
        }
        _ => false,
    }
}

fn module_uses_bool_print(module: &Module) -> bool {
    module.instructions.iter().any(instruction_uses_bool_print)
        || module.functions.iter().any(|function| {
            function
                .instructions
                .iter()
                .any(instruction_uses_bool_print)
        })
}

fn loop_name(kind: LoopKind) -> &'static str {
    match kind {
        LoopKind::While => "while",
        LoopKind::For => "for",
    }
}
