use std::fmt::Write;

use super::ir::{Instruction, LoopKind, Module, Type};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    writeln!(output, "(module").unwrap();

    // print() is provided by the host.
    if module.instructions.iter().any(instruction_uses_bool_print) {
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

    if module.memory_pages > 0 {
        writeln!(output, "  (memory {})", module.memory_pages).unwrap();
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
        emit_instruction(instruction, 2, &mut output);
    }

    writeln!(output, "  )").unwrap();

    writeln!(output, "  (export \"main\" (func $main))").unwrap();

    writeln!(output, ")").unwrap();

    output
}

fn emit_instruction(instruction: &Instruction, indent: usize, output: &mut String) {
    let prefix = "  ".repeat(indent);

    match instruction {
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

        Instruction::If {
            then_instructions,
            else_instructions,
        } => {
            writeln!(output, "{prefix}if").unwrap();
            for instruction in then_instructions {
                emit_instruction(instruction, indent + 1, output);
            }
            if !else_instructions.is_empty() {
                writeln!(output, "{prefix}else").unwrap();
                for instruction in else_instructions {
                    emit_instruction(instruction, indent + 1, output);
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
                emit_instruction(instruction, indent + 2, output);
            }
            writeln!(output, "{prefix}    i32.eqz").unwrap();
            writeln!(output, "{prefix}    br_if ${name}_end_{id}").unwrap();
            writeln!(output, "{prefix}    block ${name}_continue_{id}").unwrap();
            for instruction in body_instructions {
                emit_instruction(instruction, indent + 3, output);
            }
            writeln!(output, "{prefix}    end").unwrap();
            for instruction in update_instructions {
                emit_instruction(instruction, indent + 2, output);
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

        Instruction::I64Add => emit_simple("i64.add", &prefix, output),
        Instruction::I64Sub => emit_simple("i64.sub", &prefix, output),
        Instruction::I64Mul => emit_simple("i64.mul", &prefix, output),
        Instruction::I64DivS => emit_simple("i64.div_s", &prefix, output),
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
    }
}

fn instruction_uses_bool_print(instruction: &Instruction) -> bool {
    match instruction {
        Instruction::CallPrint(Type::Bool) => true,
        Instruction::If {
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

fn loop_name(kind: LoopKind) -> &'static str {
    match kind {
        LoopKind::While => "while",
        LoopKind::For => "for",
    }
}
