use crate::{
    codegen::{IntegerBinaryOp as Op, NumericConversion},
    types::{IntegerType, NumericType as N},
};
use std::fmt::Write;
pub(super) fn emit_binary(op: Op, output: &mut String) {
    writeln!(
        output,
        "function l ${}(l %left, l %right) {{\n@start",
        op.helper(IntegerType::U64)
    )
    .unwrap();
    let instruction = match op {
        Op::Add => {
            output.push_str("  %limit =l sub -1, %right\n  %bad =w cugtl %left, %limit\n");
            "add"
        }
        Op::Subtract => {
            output.push_str("  %bad =w cultl %left, %right\n");
            "sub"
        }
        Op::Multiply => {
            output.push_str("  %zero =w ceql %right, 0\n  jnz %zero, @ok, @bounds\n@bounds\n  %limit =l udiv -1, %right\n  %bad =w cugtl %left, %limit\n");
            "mul"
        }
        Op::Divide | Op::Remainder => {
            output.push_str("  %bad =w ceql %right, 0\n");
            if op == Op::Divide { "udiv" } else { "urem" }
        }
        Op::ShiftLeft | Op::ShiftRight => {
            output.push_str("  %wide =w cugel %right, 64\n  jnz %wide, @trap, @bounds\n@bounds\n");
            if op == Op::ShiftLeft {
                output.push_str("  %limit =l shr -1, %right\n  %bad =w cugtl %left, %limit\n");
                "shl"
            } else {
                output.push_str("  %bad =w copy 0\n");
                "shr"
            }
        }
        Op::BitAnd | Op::BitOr | Op::BitXor => {
            let instruction = match op {
                Op::BitAnd => "and",
                Op::BitOr => "or",
                _ => "xor",
            };
            writeln!(
                output,
                "  %result =l {instruction} %left, %right\n  ret %result\n}}\n"
            )
            .unwrap();
            return;
        }
    };
    writeln!(output,"  jnz %bad, @trap, @ok\n@trap\n  call $abort()\n  hlt\n@ok\n  %result =l {instruction} %left, %right\n  ret %result\n}}\n").unwrap();
}
pub(super) fn emit_conversion(c: NumericConversion, output: &mut String) {
    let from = super::conversion::type_name(c.from);
    let to = super::conversion::type_name(c.to);
    writeln!(
        output,
        "function {to} ${}({from} %value) {{\n@start",
        c.helper()
    )
    .unwrap();
    match (c.from, c.to) {
        (N::Integer(from), N::Integer(to)) => {
            if from == IntegerType::U64 {
                writeln!(output, "  %bad =w cugtl %value, {}", to.maximum()).unwrap();
            } else {
                output.push_str("  %bad =w csltl %value, 0\n");
            }
            output.push_str("  jnz %bad, @trap, @ok\n@ok\n  ret %value\n");
        }
        (N::Integer(_), N::F32 | N::F64) => {
            writeln!(output, "  %result ={to} ultof %value").unwrap();
            let number = if c.to == N::F32 {
                output.push_str("  %number =d exts %result\n");
                "%number"
            } else {
                "%result"
            };
            writeln!(output,"  %bad =w cged {number}, d_18446744073709551616\n  jnz %bad, @trap, @convert\n@convert\n  %back =l dtoui {number}\n  %changed =w cnel %back, %value\n  jnz %changed, @trap, @ok\n@ok\n  ret %result").unwrap();
        }
        (N::F32 | N::F64, N::Integer(_)) => {
            let number = if c.from == N::F32 {
                output.push_str("  %number =d exts %value\n");
                "%number"
            } else {
                "%value"
            };
            writeln!(output,"  %nan =w cuod {number}, {number}\n  %below =w cltd {number}, d_0\n  %above =w cged {number}, d_18446744073709551616\n  %outside =w or %below, %above\n  %bits =l cast {number}\n  %negative_zero =w ceql %bits, -9223372036854775808\n  %special =w or %nan, %negative_zero\n  %bad =w or %outside, %special\n  jnz %bad, @trap, @convert\n@convert\n  %result =l dtoui {number}\n  %back =d ultof %result\n  %changed =w cned %back, {number}\n  jnz %changed, @trap, @ok\n@ok\n  ret %result").unwrap();
        }
        _ => unreachable!("u64 conversion"),
    }
    output.push_str("@trap\n  call $abort()\n  hlt\n}\n\n");
}
