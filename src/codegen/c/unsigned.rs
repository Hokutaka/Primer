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
            guard("left > UINT64_MAX - right", "u64 add overflow", output);
            "+"
        }
        Op::Subtract => {
            guard("left < right", "u64 subtract overflow", output);
            "-"
        }
        Op::Multiply => {
            guard(
                "right != 0 && left > UINT64_MAX / right",
                "u64 multiply overflow",
                output,
            );
            "*"
        }
        Op::Divide | Op::Remainder => {
            guard(
                "right == 0",
                if op == Op::Divide {
                    "integer division by zero"
                } else {
                    "integer remainder by zero"
                },
                output,
            );
            if op == Op::Divide { "/" } else { "%" }
        }
        Op::BitAnd => "&",
        Op::BitOr => "|",
        Op::BitXor => "^",
        Op::ShiftLeft | Op::ShiftRight => {
            guard("right >= 64", "u64 invalid shift count", output);
            if op == Op::ShiftLeft {
                guard(
                    "left > (UINT64_MAX >> right)",
                    "u64 left shift overflow",
                    output,
                );
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
                guard(
                    &format!("value > UINT64_C({})", to.maximum()),
                    "integer conversion out of range",
                    output,
                );
            } else if from.is_signed() {
                guard("value < 0", "integer conversion out of range", output);
            }
            writeln!(
                output,
                "    return ({})value;",
                super::conversion::type_name(c.to)
            )
            .unwrap();
        }
        (N::Integer(_), N::F32 | N::F64) => {
            writeln!(
                output,
                "    {to} result = ({to})value;\n    double number = (double)result;"
            )
            .unwrap();
            guard(
                "number >= 18446744073709551616.0",
                "numeric conversion inexact",
                output,
            );
            guard(
                "(uint64_t)number != value",
                "numeric conversion inexact",
                output,
            );
            output.push_str("    return result;\n");
        }
        (N::F32 | N::F64, N::Integer(_)) => {
            output.push_str("    double number = (double)value;\n");
            guard("!isfinite(number)", "numeric conversion not finite", output);
            guard(
                "number == 0.0 && signbit(number)",
                "numeric conversion negative zero",
                output,
            );
            guard(
                "number < 0.0 || number >= 18446744073709551616.0",
                "numeric conversion out of range",
                output,
            );
            output.push_str("    uint64_t result = (uint64_t)number;\n");
            guard(
                "(double)result != number",
                "numeric conversion inexact",
                output,
            );
            output.push_str("    return result;\n");
        }
        _ => unreachable!("u64 conversion"),
    }
    output.push_str("}\n\n");
}

fn guard(condition: &str, reason: &str, output: &mut String) {
    writeln!(
        output,
        "    if ({condition}) {{ fputs(\"primer: {reason}\\n\", stderr); abort(); }}"
    )
    .unwrap();
}
