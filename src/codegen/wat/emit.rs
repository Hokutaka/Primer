use std::fmt::Write;

use super::ir::{Instruction, Module, Type};

pub fn emit(module: &Module) -> String {
    let mut output = String::new();

    writeln!(output, "(module").unwrap();

    // print() is provided by the host.
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
        emit_instruction(instruction, &mut output);
    }

    writeln!(output, "  )").unwrap();

    writeln!(output, "  (export \"main\" (func $main))").unwrap();

    writeln!(output, ")").unwrap();

    output
}

fn emit_instruction(instruction: &Instruction, output: &mut String) {
    match instruction {
        Instruction::I64Const(value) => {
            writeln!(output, "    i64.const {value}").unwrap();
        }

        Instruction::F32Const(text) => {
            writeln!(output, "    f32.const {text}").unwrap();
        }

        Instruction::F64Const(text) => {
            writeln!(output, "    f64.const {text}").unwrap();
        }

        Instruction::LocalGet(name) => {
            writeln!(output, "    local.get $primer_{name}").unwrap();
        }

        Instruction::LocalSet(name) => {
            writeln!(output, "    local.set $primer_{name}").unwrap();
        }

        Instruction::I64Add => emit_simple("i64.add", output),
        Instruction::I64Sub => emit_simple("i64.sub", output),
        Instruction::I64Mul => emit_simple("i64.mul", output),
        Instruction::I64DivS => emit_simple("i64.div_s", output),

        Instruction::F32Add => emit_simple("f32.add", output),
        Instruction::F32Sub => emit_simple("f32.sub", output),
        Instruction::F32Mul => emit_simple("f32.mul", output),
        Instruction::F32Div => emit_simple("f32.div", output),
        Instruction::F32Neg => emit_simple("f32.neg", output),

        Instruction::F64Add => emit_simple("f64.add", output),
        Instruction::F64Sub => emit_simple("f64.sub", output),
        Instruction::F64Mul => emit_simple("f64.mul", output),
        Instruction::F64Div => emit_simple("f64.div", output),
        Instruction::F64Neg => emit_simple("f64.neg", output),

        Instruction::CallPrint(ty) => {
            let function = match ty {
                Type::I64 => "$print_i64",
                Type::F32 => "$print_f32",
                Type::F64 => "$print_f64",
            };

            writeln!(output, "    call {function}").unwrap();
        }
    }
}

fn emit_simple(instruction: &str, output: &mut String) {
    writeln!(output, "    {instruction}").unwrap();
}

fn wat_type(ty: Type) -> &'static str {
    match ty {
        Type::I64 => "i64",
        Type::F32 => "f32",
        Type::F64 => "f64",
    }
}
