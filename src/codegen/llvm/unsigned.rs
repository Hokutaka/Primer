use super::failure::emit_trap;
use crate::{
    codegen::{IntegerBinaryOp as Op, NumericConversion},
    runtime::FailureCode as Code,
    types::{IntegerType, NumericType as N},
};
use std::fmt::Write;

pub(super) fn emit_binary(op: Op, output: &mut String) {
    writeln!(
        output,
        "define internal i64 @{}(i64 %left, i64 %right, ptr %failure) {{\nentry:",
        op.helper(IntegerType::U64)
    )
    .unwrap();
    let mut failure = Code::IntegerOverflow;
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
            failure = if op == Op::Divide {
                Code::DivisionByZero
            } else {
                Code::RemainderByZero
            };
            if op == Op::Divide { "udiv" } else { "urem" }
        }
        Op::ShiftLeft | Op::ShiftRight => {
            output.push_str("  %wide = icmp uge i64 %right, 64\n  br i1 %wide, label %count, label %bounds\ncount:\n");
            emit_trap(Code::InvalidShiftCount, output);
            output.push_str("bounds:\n");
            if op == Op::ShiftLeft {
                output.push_str(
                    "  %limit = lshr i64 -1, %right\n  %bad = icmp ugt i64 %left, %limit\n",
                );
                "shl"
            } else {
                output.push_str("  %result = lshr i64 %left, %right\n  ret i64 %result\n}\n\n");
                return;
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
    output.push_str("  br i1 %bad, label %trap, label %ok\ntrap:\n");
    emit_trap(failure, output);
    writeln!(
        output,
        "ok:\n  %result = {instruction} i64 %left, %right\n  ret i64 %result\n}}\n"
    )
    .unwrap();
}

pub(super) fn emit_conversion(c: NumericConversion, output: &mut String) {
    let from = super::conversion::type_name(c.from);
    let to = super::conversion::type_name(c.to);
    writeln!(
        output,
        "define internal {to} @{}({from} %value, ptr %failure) {{\nentry:",
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
            output.push_str("  br i1 %bad, label %trap, label %ok\ntrap:\n");
            emit_trap(Code::IntegerConversionOutOfRange, output);
            output.push_str("ok:\n  ret i64 %value\n");
        }
        (N::Integer(_), N::F32 | N::F64) => {
            writeln!(output, "  %result = uitofp i64 %value to {to}").unwrap();
            let number = if c.to == N::F32 {
                output.push_str("  %number = fpext float %result to double\n");
                "%number"
            } else {
                "%result"
            };
            writeln!(output,"  %bad = fcmp oge double {number}, 0x43F0000000000000\n  br i1 %bad, label %inexact, label %convert\nconvert:\n  %back = fptoui double {number} to i64\n  %changed = icmp ne i64 %back, %value\n  br i1 %changed, label %inexact, label %ok\ninexact:").unwrap();
            emit_trap(Code::ConversionInexact, output);
            writeln!(output, "ok:\n  ret {to} %result").unwrap();
        }
        (N::F32 | N::F64, N::Integer(_)) => {
            let number = super::conversion::widen(c.from, output);
            super::conversion::emit_integer_input_checks(
                number,
                0.0,
                18446744073709551616.0,
                output,
            );
            writeln!(output,"  %result = fptoui double {number} to i64\n  %back = uitofp i64 %result to double\n  %changed = fcmp one double %back, {number}\n  br i1 %changed, label %inexact, label %ok\ninexact:").unwrap();
            emit_trap(Code::ConversionInexact, output);
            output.push_str("ok:\n  ret i64 %result\n");
        }
        _ => unreachable!("u64 conversion"),
    }
    output.push_str("}\n\n");
}
