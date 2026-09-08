use crate::{
    codegen::{IntegerBinaryOp as Op, NumericConversion},
    types::{IntegerType, NumericType as N},
};
use std::fmt::Write;

// 符号付き変換や未定義の桁あふれへ依存せず、演算前に範囲を検査します。
pub(super) fn emit_binary(op: Op, output: &mut String) {
    writeln!(
        output,
        "static uint64_t {}(uint64_t left, uint64_t right) {{",
        op.helper(IntegerType::U64)
    )
    .unwrap();
    let symbol = match op {
        Op::Add => {
            output.push_str("    if (left > UINT64_MAX - right) abort();\n");
            "+"
        }
        Op::Subtract => {
            output.push_str("    if (left < right) abort();\n");
            "-"
        }
        Op::Multiply => {
            output.push_str("    if (right != 0 && left > UINT64_MAX / right) abort();\n");
            "*"
        }
        Op::Divide | Op::Remainder => {
            output.push_str("    if (right == 0) abort();\n");
            if op == Op::Divide { "/" } else { "%" }
        }
        Op::BitAnd => "&",
        Op::BitOr => "|",
        Op::BitXor => "^",
        Op::ShiftLeft | Op::ShiftRight => {
            output.push_str("    if (right >= 64) abort();\n");
            if op == Op::ShiftLeft {
                output.push_str("    if (left > (UINT64_MAX >> right)) abort();\n");
                "<<"
            } else {
                ">>"
            }
        }
    };
    writeln!(output, "    return left {symbol} right;\n}}\n").unwrap();
}
pub(super) fn emit_conversion(c: NumericConversion, output: &mut String) {
    let from = super::conversion::type_name(c.from);
    let to = super::conversion::type_name(c.to);
    writeln!(output, "static {to} {}({from} value) {{", c.helper()).unwrap();
    match (c.from, c.to) {
        (N::Integer(from), N::Integer(to)) => {
            if from == IntegerType::U64 {
                writeln!(
                    output,
                    "    if (value > UINT64_C({})) abort();",
                    to.maximum()
                )
                .unwrap();
            } else if from.is_signed() {
                output.push_str("    if (value < 0) abort();\n");
            }
            writeln!(
                output,
                "    return ({})value;",
                super::conversion::type_name(c.to)
            )
            .unwrap();
        }
        (N::Integer(_), N::F32 | N::F64) => {
            writeln!(output,"    {to} result = ({to})value;\n    double number = (double)result;\n    if (number >= 18446744073709551616.0) abort();\n    if ((uint64_t)number != value) abort();\n    return result;").unwrap();
        }
        (N::F32 | N::F64, N::Integer(_)) => {
            output.push_str("    double number = (double)value;\n    if (!isfinite(number) || (number == 0.0 && signbit(number))) abort();\n    if (number < 0.0 || number >= 18446744073709551616.0) abort();\n    uint64_t result = (uint64_t)number;\n    if ((double)result != number) abort();\n    return result;\n");
        }
        _ => unreachable!("u64 conversion"),
    }
    output.push_str("}\n\n");
}
