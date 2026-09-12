use super::failure::emit_trap;
use crate::{codegen::NumericConversion, runtime::FailureCode as Code, types::NumericType};
use std::fmt::Write;

pub(super) fn type_name(ty: NumericType) -> &'static str {
    match ty {
        NumericType::Integer(_) => "i64",
        NumericType::F32 => "float",
        NumericType::F64 => "double",
    }
}

pub(super) fn emit_support(conversion: NumericConversion, output: &mut String) {
    if conversion.uses_u64() {
        return super::unsigned::emit_conversion(conversion, output);
    }
    let NumericConversion { from, to } = conversion;
    let result_ty = type_name(to);
    writeln!(
        output,
        "define internal {result_ty} @{}({} %value, ptr %failure) {{\nentry:",
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
            // 整数を保存できない丸めは、範囲外へ丸められてもinexactです。
            writeln!(output, "  %below = fcmp olt double {number}, 0xC3E0000000000000\n  %above = fcmp oge double {number}, 0x43E0000000000000\n  %outside = or i1 %below, %above\n  br i1 %outside, label %inexact, label %convert\nconvert:\n  %back = fptosi double {number} to i64\n  %changed = icmp ne i64 %back, %value\n  br i1 %changed, label %inexact, label %ok\ninexact:").unwrap();
            emit_trap(Code::ConversionInexact, output);
        }
        (NumericType::F32 | NumericType::F64, NumericType::Integer(ty)) => {
            let number = widen(from, output);
            emit_integer_input_checks(
                number,
                ty.minimum() as f64,
                (ty.maximum() + 1) as f64,
                output,
            );
            writeln!(output, "  %result = fptosi double {number} to i64\n  %back = sitofp i64 %result to double\n  %changed = fcmp one double %back, {number}\n  br i1 %changed, label %inexact, label %ok\ninexact:").unwrap();
            emit_trap(Code::ConversionInexact, output);
        }
        (NumericType::F32, NumericType::F64) => {
            output.push_str("  %nan = fcmp uno float %value, %value\n  br i1 %nan, label %nan_failure, label %convert\nnan_failure:\n");
            emit_trap(Code::ConversionNaN, output);
            output.push_str("convert:\n  %result = fpext float %value to double\n  br label %ok\n");
        }
        (NumericType::F64, NumericType::F32) => {
            output.push_str("  %nan = fcmp uno double %value, %value\n  br i1 %nan, label %nan_failure, label %bounds\nnan_failure:\n");
            emit_trap(Code::ConversionNaN, output);
            // 無限大は保存できます。有限なf32上限超過は丸めの前に分類します。
            let maximum = f64::from(f32::MAX).to_bits();
            let minimum = (-f64::from(f32::MAX)).to_bits();
            writeln!(output, "bounds:\n  %positive_inf = fcmp oeq double %value, 0x7FF0000000000000\n  %negative_inf = fcmp oeq double %value, 0xFFF0000000000000\n  %infinite = or i1 %positive_inf, %negative_inf\n  br i1 %infinite, label %convert, label %finite_bounds\nfinite_bounds:\n  %below = fcmp olt double %value, 0x{minimum:016X}\n  %above = fcmp ogt double %value, 0x{maximum:016X}\n  %outside = or i1 %below, %above\n  br i1 %outside, label %range, label %convert\nrange:").unwrap();
            emit_trap(Code::ConversionOutOfRange, output);
            output.push_str("convert:\n  %result = fptrunc double %value to float\n  %back = fpext float %result to double\n  %changed = fcmp one double %back, %value\n  br i1 %changed, label %inexact, label %ok\ninexact:\n");
            emit_trap(Code::ConversionInexact, output);
        }
        _ => unreachable!("integer and identity conversions use separate lowering"),
    }
    writeln!(output, "ok:\n  ret {result_ty} %result\n}}\n").unwrap();
}

pub(super) fn widen(from: NumericType, output: &mut String) -> &'static str {
    if from == NumericType::F32 {
        output.push_str("  %number = fpext float %value to double\n");
        "%number"
    } else {
        "%value"
    }
}

pub(super) fn emit_integer_input_checks(
    number: &str,
    minimum: f64,
    upper: f64,
    output: &mut String,
) {
    // VMと同じ順に非有限、負のゼロ、範囲を検査し、fptosi/fptouiのpoisonを避けます。
    writeln!(output, "  %nan = fcmp uno double {number}, {number}\n  %positive_inf = fcmp oeq double {number}, 0x7FF0000000000000\n  %negative_inf = fcmp oeq double {number}, 0xFFF0000000000000\n  %infinite = or i1 %positive_inf, %negative_inf\n  %nonfinite = or i1 %nan, %infinite\n  br i1 %nonfinite, label %nonfinite_failure, label %zero\nnonfinite_failure:").unwrap();
    emit_trap(Code::ConversionNotFinite, output);
    writeln!(output, "zero:\n  %bits = bitcast double {number} to i64\n  %negative_zero = icmp eq i64 %bits, -9223372036854775808\n  br i1 %negative_zero, label %zero_failure, label %bounds\nzero_failure:").unwrap();
    emit_trap(Code::ConversionNegativeZero, output);
    writeln!(output, "bounds:\n  %below = fcmp olt double {number}, 0x{:016X}\n  %above = fcmp oge double {number}, 0x{:016X}\n  %outside = or i1 %below, %above\n  br i1 %outside, label %range, label %convert\nrange:", minimum.to_bits(), upper.to_bits()).unwrap();
    emit_trap(Code::ConversionOutOfRange, output);
    output.push_str("convert:\n");
}
