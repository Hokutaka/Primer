use crate::{codegen::NumericConversion, types::NumericType};
use std::fmt::Write;

pub(super) fn type_name(ty: NumericType) -> &'static str {
    match ty {
        NumericType::Integer(_) => "i64",
        NumericType::F32 => "float",
        NumericType::F64 => "double",
    }
}

pub(super) fn emit_support(conversion: NumericConversion, output: &mut String) {
    let NumericConversion { from, to } = conversion;
    let result_ty = type_name(to);
    writeln!(
        output,
        "define internal {result_ty} @{}({} %value) {{\nentry:",
        conversion.helper(),
        type_name(from)
    )
    .unwrap();
    match (from, to) {
        (NumericType::Integer(_), NumericType::F32 | NumericType::F64) => {
            writeln!(output, "  %result = sitofp i64 %value to {result_ty}").unwrap();
            let number = if to == NumericType::F32 {
                output.push_str("  %number = fpext float %result to double\n");
                "%number"
            } else {
                "%result"
            };
            // fptosiの前に範囲を検査し、poison値を比較へ流しません。
            writeln!(output, "  %below = fcmp olt double {number}, 0xC3E0000000000000\n  %above = fcmp oge double {number}, 0x43E0000000000000\n  %outside = or i1 %below, %above\n  br i1 %outside, label %trap, label %convert\nconvert:\n  %back = fptosi double {number} to i64\n  %changed = icmp ne i64 %back, %value\n  br i1 %changed, label %trap, label %ok").unwrap();
        }
        (NumericType::F32 | NumericType::F64, NumericType::Integer(ty)) => {
            let number = if from == NumericType::F32 {
                output.push_str("  %number = fpext float %value to double\n");
                "%number"
            } else {
                "%value"
            };
            let lower = (ty.minimum() as f64).to_bits();
            let upper = ((i128::from(ty.maximum()) + 1) as f64).to_bits();
            writeln!(output, "  %below = fcmp ult double {number}, 0x{lower:016X}\n  %above = fcmp uge double {number}, 0x{upper:016X}\n  %outside = or i1 %below, %above\n  %bits = bitcast double {number} to i64\n  %negative_zero = icmp eq i64 %bits, -9223372036854775808\n  %bad = or i1 %outside, %negative_zero\n  br i1 %bad, label %trap, label %convert\nconvert:\n  %result = fptosi double {number} to i64\n  %back = sitofp i64 %result to double\n  %changed = fcmp one double %back, {number}\n  br i1 %changed, label %trap, label %ok").unwrap();
        }
        (NumericType::F32, NumericType::F64) => {
            output.push_str("  %nan = fcmp uno float %value, %value\n  br i1 %nan, label %trap, label %convert\nconvert:\n  %result = fpext float %value to double\n  br label %ok\n");
        }
        (NumericType::F64, NumericType::F32) => {
            output.push_str("  %nan = fcmp uno double %value, %value\n  br i1 %nan, label %trap, label %convert\nconvert:\n  %result = fptrunc double %value to float\n  %back = fpext float %result to double\n  %changed = fcmp one double %back, %value\n  br i1 %changed, label %trap, label %ok\n");
        }
        _ => unreachable!("integer and identity conversions use separate lowering"),
    }
    writeln!(
        output,
        "trap:\n  call void @llvm.trap()\n  unreachable\nok:\n  ret {result_ty} %result\n}}\n"
    )
    .unwrap();
}
