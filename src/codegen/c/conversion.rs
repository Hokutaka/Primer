use crate::{codegen::NumericConversion, types::NumericType};
use std::fmt::Write;

fn type_name(ty: NumericType) -> &'static str {
    match ty {
        NumericType::Integer(_) => "int64_t",
        NumericType::F32 => "float",
        NumericType::F64 => "double",
    }
}

pub(super) fn emit_support(conversion: NumericConversion, output: &mut String) {
    let NumericConversion { from, to } = conversion;
    writeln!(
        output,
        "static {} {}({} value) {{",
        type_name(to),
        conversion.helper(),
        type_name(from)
    )
    .unwrap();
    match (from, to) {
        (NumericType::Integer(_), NumericType::F32 | NumericType::F64) => {
            writeln!(
                output,
                "    {} result = ({})value;",
                type_name(to),
                type_name(to)
            )
            .unwrap();
            // 範囲検査を先に行い、Cの範囲外float-to-int変換を実行しません。
            output.push_str("    double number = (double)result;\n    if (number < -9223372036854775808.0 || number >= 9223372036854775808.0) abort();\n    if ((int64_t)number != value) abort();\n    return result;\n");
        }
        (NumericType::F32 | NumericType::F64, NumericType::Integer(ty)) => {
            output.push_str("    double number = (double)value;\n    if (!isfinite(number)) abort();\n    if (number == 0.0 && signbit(number)) abort();\n");
            writeln!(
                output,
                "    if (number < {}.0 || number >= {}.0) abort();",
                ty.minimum(),
                i128::from(ty.maximum()) + 1
            )
            .unwrap();
            output.push_str("    int64_t result = (int64_t)number;\n    if ((double)result != number) abort();\n    return result;\n");
        }
        (NumericType::F32, NumericType::F64) => {
            output.push_str("    if (isnan(value)) abort();\n    return (double)value;\n");
        }
        (NumericType::F64, NumericType::F32) => {
            output.push_str("    if (isnan(value)) abort();\n    if (isinf(value)) return signbit(value) ? -INFINITY : INFINITY;\n    if (value > (double)FLT_MAX || value < -(double)FLT_MAX) abort();\n    float result = (float)value;\n    if ((double)result != value) abort();\n    return result;\n");
        }
        _ => unreachable!("integer and identity conversions use separate lowering"),
    }
    output.push_str("}\n\n");
}
