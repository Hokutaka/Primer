use crate::{
    codegen::{IntegerBinaryOp as Op, NumericConversion},
    types::{IntegerType, NumericType as N},
};
use std::fmt::Write;

pub(super) fn emit_binary(op: Op, output: &mut String) {
    writeln!(
        output,
        "define internal i64 @{}(i64 %left, i64 %right) {{\nentry:",
        op.helper(IntegerType::U64)
    )
    .unwrap();
    let instruction = match op {
        Op::Add => {
            output.push_str("  %limit = sub i64 -1, %right\n  %bad = icmp ugt i64 %left, %limit\n");
            "add"
        }
        Op::Subtract => {
            output.push_str("  %bad = icmp ult i64 %left, %right\n");
            "sub"
        }
        Op::Multiply => {
            output.push_str("  %zero = icmp eq i64 %right, 0\n  br i1 %zero, label %ok, label %bounds\nbounds:\n  %limit = udiv i64 -1, %right\n  %bad = icmp ugt i64 %left, %limit\n");
            "mul"
        }
        Op::Divide | Op::Remainder => {
            output.push_str("  %bad = icmp eq i64 %right, 0\n");
            if op == Op::Divide { "udiv" } else { "urem" }
        }
        Op::ShiftLeft | Op::ShiftRight => {
            output.push_str("  %wide = icmp uge i64 %right, 64\n  br i1 %wide, label %trap, label %bounds\nbounds:\n");
            if op == Op::ShiftLeft {
                output.push_str(
                    "  %limit = lshr i64 -1, %right\n  %bad = icmp ugt i64 %left, %limit\n",
                );
                "shl"
            } else {
                output.push_str("  %bad = icmp ne i64 0, 0\n");
                "lshr"
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
                "  %result = {instruction} i64 %left, %right\n  ret i64 %result\n}}\n"
            )
            .unwrap();
            return;
        }
    };
    writeln!(output, "  br i1 %bad, label %trap, label %ok\ntrap:\n  call void @llvm.trap()\n  unreachable\nok:\n  %result = {instruction} i64 %left, %right\n  ret i64 %result\n}}\n").unwrap();
}
pub(super) fn emit_conversion(c: NumericConversion, output: &mut String) {
    let from = super::conversion::type_name(c.from);
    let to = super::conversion::type_name(c.to);
    writeln!(
        output,
        "define internal {to} @{}({from} %value) {{\nentry:",
        c.helper()
    )
    .unwrap();
    match (c.from, c.to) {
        (N::Integer(from), N::Integer(to)) => {
            if from == IntegerType::U64 {
                writeln!(output, "  %bad = icmp ugt i64 %value, {}", to.maximum()).unwrap();
            } else {
                output.push_str("  %bad = icmp slt i64 %value, 0\n");
            }
            output.push_str("  br i1 %bad, label %trap, label %ok\nok:\n  ret i64 %value\n");
        }
        (N::Integer(_), N::F32 | N::F64) => {
            writeln!(output, "  %result = uitofp i64 %value to {to}").unwrap();
            let number = if c.to == N::F32 {
                output.push_str("  %number = fpext float %result to double\n");
                "%number"
            } else {
                "%result"
            };
            writeln!(output,"  %bad = fcmp oge double {number}, 0x43F0000000000000\n  br i1 %bad, label %trap, label %convert\nconvert:\n  %back = fptoui double {number} to i64\n  %changed = icmp ne i64 %back, %value\n  br i1 %changed, label %trap, label %ok\nok:\n  ret {to} %result").unwrap();
        }
        (N::F32 | N::F64, N::Integer(_)) => {
            let number = if c.from == N::F32 {
                output.push_str("  %number = fpext float %value to double\n");
                "%number"
            } else {
                "%value"
            };
            writeln!(output,"  %below = fcmp ult double {number}, 0.0\n  %above = fcmp uge double {number}, 0x43F0000000000000\n  %outside = or i1 %below, %above\n  %bits = bitcast double {number} to i64\n  %negative_zero = icmp eq i64 %bits, -9223372036854775808\n  %bad = or i1 %outside, %negative_zero\n  br i1 %bad, label %trap, label %convert\nconvert:\n  %result = fptoui double {number} to i64\n  %back = uitofp i64 %result to double\n  %changed = fcmp one double %back, {number}\n  br i1 %changed, label %trap, label %ok\nok:\n  ret i64 %result").unwrap();
        }
        _ => unreachable!("u64 conversion"),
    }
    output.push_str("trap:\n  call void @llvm.trap()\n  unreachable\n}\n\n");
}
