use crate::{
    codegen::NumericConversion,
    runtime::FailureCode,
    types::{IntegerType, NumericType},
};
use std::fmt::Write;

pub(super) fn type_name(ty: NumericType) -> &'static str {
    match ty {
        NumericType::Integer(_) => "l",
        NumericType::F32 => "s",
        NumericType::F64 => "d",
    }
}

pub(super) fn emit_support(conversion: NumericConversion, output: &mut String) {
    let NumericConversion { from, to } = conversion;
    let result_ty = type_name(to);
    writeln!(
        output,
        "function {result_ty} ${}({} %value, l %origin, l %origin_len) {{\n@start",
        conversion.helper(),
        type_name(from)
    )
    .unwrap();
    match (from, to) {
        (NumericType::Integer(from), NumericType::Integer(to)) => {
            if from == IntegerType::U64 {
                writeln!(output, "  %bad =w cugtl %value, {}", to.maximum()).unwrap();
            } else {
                output.push_str("  %bad =w csltl %value, 0\n");
            }
            output.push_str("  jnz %bad, @range, @ok\n");
            super::failure::block("range", FailureCode::IntegerConversionOutOfRange, output);
            output.push_str("@ok\n  ret %value\n}\n\n");
            return;
        }
        (NumericType::Integer(integer), NumericType::F32 | NumericType::F64) => {
            let unsigned = integer == IntegerType::U64;
            let instruction = if unsigned { "ultof" } else { "sltof" };
            writeln!(output, "  %result ={result_ty} {instruction} %value").unwrap();
            let number = if to == NumericType::F32 {
                output.push_str("  %number =d exts %result\n");
                "%number"
            } else {
                "%result"
            };
            let upper = if unsigned {
                "18446744073709551616"
            } else {
                "9223372036854775808"
            };
            let instruction = if unsigned { "dtoui" } else { "dtosi" };
            writeln!(output, "  %outside =w cged {number}, d_{upper}\n  jnz %outside, @inexact, @convert\n@convert\n  %back =l {instruction} {number}\n  %changed =w cnel %back, %value\n  jnz %changed, @inexact, @ok").unwrap();
            super::failure::block("inexact", FailureCode::ConversionInexact, output);
        }
        (NumericType::F32 | NumericType::F64, NumericType::Integer(integer)) => {
            let number = if from == NumericType::F32 {
                output.push_str("  %number =d exts %value\n");
                "%number"
            } else {
                "%value"
            };
            let instruction = if integer == IntegerType::U64 {
                "dtoui"
            } else {
                "dtosi"
            };
            let back = if integer == IntegerType::U64 {
                "ultof"
            } else {
                "sltof"
            };
            writeln!(output, "  %bits =l cast {number}\n  %magnitude =l and %bits, 9223372036854775807\n  %not_finite =w cugel %magnitude, 9218868437227405312\n  jnz %not_finite, @not_finite, @zero_check\n@zero_check\n  %negative_zero =w ceql %bits, -9223372036854775808\n  jnz %negative_zero, @negative_zero, @bounds\n@bounds\n  %below =w cltd {number}, d_{}\n  %above =w cged {number}, d_{}\n  %outside =w or %below, %above\n  jnz %outside, @range, @convert\n@convert\n  %result =l {instruction} {number}\n  %back =d {back} %result\n  %changed =w cned %back, {number}\n  jnz %changed, @inexact, @ok", integer.minimum(), integer.maximum() + 1).unwrap();
            super::failure::block("not_finite", FailureCode::ConversionNotFinite, output);
            super::failure::block("negative_zero", FailureCode::ConversionNegativeZero, output);
            super::failure::block("range", FailureCode::ConversionOutOfRange, output);
            super::failure::block("inexact", FailureCode::ConversionInexact, output);
        }
        (NumericType::F32, NumericType::F64) => {
            output.push_str("  %nan =w cuos %value, %value\n  jnz %nan, @nan, @convert\n@convert\n  %result =d exts %value\n  jmp @ok\n");
            super::failure::block("nan", FailureCode::ConversionNaN, output);
        }
        (NumericType::F64, NumericType::F32) => {
            // 無限大は保持できるが、有限値がf32の範囲外なら丸めの前に区別する。
            output.push_str("  %nan =w cuod %value, %value\n  jnz %nan, @nan, @bounds\n@bounds\n  %bits =l cast %value\n  %magnitude =l and %bits, 9223372036854775807\n  %finite =w cultl %magnitude, 9218868437227405312\n  %below =w cltd %value, d_-340282346638528859811704183484516925440\n  %above =w cgtd %value, d_340282346638528859811704183484516925440\n  %outside =w or %below, %above\n  %range_error =w and %finite, %outside\n  jnz %range_error, @range, @convert\n@convert\n  %result =s truncd %value\n  %back =d exts %result\n  %changed =w cned %back, %value\n  jnz %changed, @inexact, @ok\n");
            super::failure::block("nan", FailureCode::ConversionNaN, output);
            super::failure::block("range", FailureCode::ConversionOutOfRange, output);
            super::failure::block("inexact", FailureCode::ConversionInexact, output);
        }
        _ => unreachable!("identity conversions use separate lowering"),
    }
    output.push_str("@ok\n  ret %result\n}\n\n");
}
