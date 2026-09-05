use crate::{codegen::NumericConversion, types::NumericType};
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
        "function {result_ty} ${}({} %value) {{\n@start",
        conversion.helper(),
        type_name(from)
    )
    .unwrap();
    match (from, to) {
        (NumericType::Integer(_), NumericType::F32 | NumericType::F64) => {
            writeln!(output, "  %result ={result_ty} sltof %value").unwrap();
            let number = if to == NumericType::F32 {
                output.push_str("  %number =d exts %result\n");
                "%number"
            } else {
                "%result"
            };
            writeln!(output, "  %below =w cltd {number}, d_-9223372036854775808\n  %above =w cged {number}, d_9223372036854775808\n  %outside =w or %below, %above\n  jnz %outside, @trap, @convert\n@convert\n  %back =l dtosi {number}\n  %changed =w cnel %back, %value\n  jnz %changed, @trap, @ok").unwrap();
        }
        (NumericType::F32 | NumericType::F64, NumericType::Integer(ty)) => {
            let number = if from == NumericType::F32 {
                output.push_str("  %number =d exts %value\n");
                "%number"
            } else {
                "%value"
            };
            writeln!(output, "  %nan =w cuod {number}, {number}\n  %below =w cltd {number}, d_{}\n  %above =w cged {number}, d_{}\n  %outside =w or %below, %above\n  %bits =l cast {number}\n  %negative_zero =w ceql %bits, -9223372036854775808\n  %special =w or %nan, %negative_zero\n  %bad =w or %outside, %special\n  jnz %bad, @trap, @convert\n@convert\n  %result =l dtosi {number}\n  %back =d sltof %result\n  %changed =w cned %back, {number}\n  jnz %changed, @trap, @ok", ty.minimum(), i128::from(ty.maximum()) + 1).unwrap();
        }
        (NumericType::F32, NumericType::F64) => {
            output.push_str("  %nan =w cuos %value, %value\n  jnz %nan, @trap, @convert\n@convert\n  %result =d exts %value\n  jmp @ok\n");
        }
        (NumericType::F64, NumericType::F32) => {
            output.push_str("  %nan =w cuod %value, %value\n  jnz %nan, @trap, @convert\n@convert\n  %result =s truncd %value\n  %back =d exts %result\n  %changed =w cned %back, %value\n  jnz %changed, @trap, @ok\n");
        }
        _ => unreachable!("integer and identity conversions use separate lowering"),
    }
    output.push_str("@trap\n  call $abort()\n  hlt\n@ok\n  ret %result\n}\n\n");
}
