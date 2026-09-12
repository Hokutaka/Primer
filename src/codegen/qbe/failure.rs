use std::fmt::Write;

use crate::runtime::FailureCode;

use super::ir::{BinaryOp, FailureOrigin, Instruction, Module};

fn origin(instruction: &Instruction) -> Option<FailureOrigin> {
    match instruction {
        Instruction::ConvertNumeric { origin, .. }
        | Instruction::IntegerBinary { origin, .. }
        | Instruction::CheckIntegerRange { origin, .. }
        | Instruction::CheckedI64Negate { origin, .. }
        | Instruction::Abort { origin }
        | Instruction::Binary {
            origin,
            op:
                BinaryOp::CheckedI64Add
                | BinaryOp::CheckedI64Subtract
                | BinaryOp::CheckedI64Multiply
                | BinaryOp::CheckedI64Divide,
            ..
        } => Some(*origin),
        _ => None,
    }
}

fn suffix(origin: FailureOrigin) -> String {
    format!(
        " node={}{} bytes={}..{}\n",
        origin.node.0,
        origin.span.source_id().record_field(),
        origin.span.start(),
        origin.span.end()
    )
}

fn symbol(origin: FailureOrigin) -> String {
    format!(
        "primer_origin_{}_{}_{}",
        origin.node.0,
        origin.span.start(),
        origin.span.end()
    )
}

pub(super) fn arguments(origin: FailureOrigin) -> String {
    format!("l ${}, l {}", symbol(origin), suffix(origin).len())
}

pub(super) fn code_arguments(code: FailureCode) -> String {
    format!(
        "l $primer_code_{}, l {}",
        code.name().replace('-', "_"),
        prefix(code).len()
    )
}

fn prefix(code: FailureCode) -> String {
    format!("primer: runtime-v1 code={}", code.name())
}

pub(super) fn call(code: FailureCode, output: &mut String) {
    writeln!(
        output,
        "  call $primer_runtime_failure({}, l %origin, l %origin_len)\n  hlt",
        code_arguments(code)
    )
    .unwrap();
}

pub(super) fn block(label: &str, code: FailureCode, output: &mut String) {
    writeln!(output, "@{label}").unwrap();
    call(code, output);
}

pub(super) fn emit(module: &Module, output: &mut String) {
    let mut origins = Vec::new();
    for item in module
        .instructions
        .iter()
        .chain(module.functions.iter().flat_map(|f| &f.instructions))
        .filter_map(origin)
    {
        if !origins.contains(&item) {
            origins.push(item);
        }
    }
    if origins.is_empty() {
        return;
    }
    for origin in origins {
        data(&symbol(origin), &suffix(origin), output);
    }
    use FailureCode::*;
    for code in [
        IntegerOverflow,
        DivisionByZero,
        DivisionOverflow,
        RemainderByZero,
        InvalidShiftCount,
        IntegerConversionOutOfRange,
        ConversionOutOfRange,
        ConversionInexact,
        ConversionNotFinite,
        ConversionNaN,
        ConversionNegativeZero,
        ArrayIndexOutOfBounds,
    ] {
        data(
            &format!("primer_code_{}", code.name().replace('-', "_")),
            &prefix(code),
            output,
        );
        writeln!(
            output,
            "function $primer_fail_{}(l %origin, l %origin_len) {{\n@start",
            code.name().replace('-', "_")
        )
        .unwrap();
        call(code, output);
        output.push_str("}\n\n");
    }
    // QBEの既存のLinux/SysV ABI。失敗時だけstdoutをflushし、不変の二片を出力します。
    output.push_str("function $primer_runtime_failure(l %code, l %code_len, l %origin, l %origin_len) {\n@start\n  call $fflush(l 0)\n  call $write(w 2, l %code, l %code_len)\n  call $write(w 2, l %origin, l %origin_len)\n  call $abort()\n  hlt\n}\n\n");
}

fn data(name: &str, text: &str, output: &mut String) {
    write!(output, "section \".rodata\" data ${name} = {{ ").unwrap();
    for (index, byte) in text.bytes().enumerate() {
        if index != 0 {
            output.push_str(", ");
        }
        write!(output, "b {byte}").unwrap();
    }
    output.push_str(" }\n");
}
