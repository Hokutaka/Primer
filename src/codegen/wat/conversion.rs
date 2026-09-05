use crate::{codegen::NumericConversion, types::NumericType};

fn type_name(ty: NumericType) -> &'static str {
    match ty {
        NumericType::Integer(_) => "i64",
        NumericType::F32 => "f32",
        NumericType::F64 => "f64",
    }
}

pub(super) fn emit_support(conversion: NumericConversion, output: &mut String) {
    let NumericConversion { from, to } = conversion;
    output.push_str(&format!(
        "  (func ${} (param $value {}) (result {})\n    (local $result {})\n    (local $number f64)\n",
        conversion.helper(), type_name(from), type_name(to), type_name(to)
    ));
    match (from, to) {
        (NumericType::Integer(_), NumericType::F32 | NumericType::F64) => {
            output.push_str(&format!("    local.get $value\n    {}.convert_i64_s\n    local.set $result\n    local.get $result\n", type_name(to)));
            if to == NumericType::F32 {
                output.push_str("    f64.promote_f32\n");
            }
            output.push_str("    local.set $number\n    local.get $number\n    f64.const -9223372036854775808\n    f64.lt\n    local.get $number\n    f64.const 9223372036854775808\n    f64.ge\n    i32.or\n    if\n      unreachable\n    end\n    local.get $number\n    i64.trunc_f64_s\n    local.get $value\n    i64.ne\n    if\n      unreachable\n    end\n");
        }
        (NumericType::F32 | NumericType::F64, NumericType::Integer(ty)) => {
            output.push_str("    local.get $value\n");
            if from == NumericType::F32 {
                output.push_str("    f64.promote_f32\n");
            }
            // 順序比較の両方を満たす必要があるため、NaNも変換前に拒否します。
            output.push_str(&format!("    local.set $number\n    local.get $number\n    f64.const {}\n    f64.ge\n    local.get $number\n    f64.const {}\n    f64.lt\n    i32.and\n    i32.eqz\n    if\n      unreachable\n    end\n", ty.minimum(), i128::from(ty.maximum()) + 1));
            output.push_str("    local.get $number\n    i64.reinterpret_f64\n    i64.const -9223372036854775808\n    i64.eq\n    if\n      unreachable\n    end\n    local.get $number\n    i64.trunc_f64_s\n    local.set $result\n    local.get $result\n    f64.convert_i64_s\n    local.get $number\n    f64.ne\n    if\n      unreachable\n    end\n");
        }
        (NumericType::F32, NumericType::F64) => {
            output.push_str("    local.get $value\n    local.get $value\n    f32.ne\n    if\n      unreachable\n    end\n    local.get $value\n    f64.promote_f32\n    local.set $result\n");
        }
        (NumericType::F64, NumericType::F32) => {
            output.push_str("    local.get $value\n    local.get $value\n    f64.ne\n    if\n      unreachable\n    end\n    local.get $value\n    f32.demote_f64\n    local.set $result\n    local.get $result\n    f64.promote_f32\n    local.get $value\n    f64.ne\n    if\n      unreachable\n    end\n");
        }
        _ => unreachable!("identity and integer conversions are lowered separately"),
    }
    output.push_str("    local.get $result\n  )\n");
}
