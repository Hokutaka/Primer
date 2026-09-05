use crate::types::NumericType;

use super::{Value, VmErrorKind, VmResult};

/// 変換先の型と、数値を保てなかった理由を分けて記録します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericConversionFailure {
    OutOfRange,
    Inexact,
    NotFinite,
    NaN,
    NegativeZero,
}

pub(super) fn convert(value: Value, from: NumericType, to: NumericType) -> VmResult<Value> {
    if value.ty() != from.into() {
        return Err(VmErrorKind::TypeMismatch {
            expected: from.into(),
            actual: value.ty(),
        });
    }
    if from == to {
        return Ok(value);
    }
    let fail = |reason| VmErrorKind::NumericConversionFailed { from, to, reason };
    match value {
        Value::Integer(value, _) => {
            let (result, number) = match to {
                NumericType::Integer(ty) => {
                    if !ty.contains(value) {
                        return Err(fail(NumericConversionFailure::OutOfRange));
                    }
                    return Ok(Value::Integer(value, ty));
                }
                NumericType::F32 => {
                    let result = value as f32;
                    (Value::F32(result), f64::from(result))
                }
                NumericType::F64 => {
                    let result = value as f64;
                    (Value::F64(result), result)
                }
            };
            // i64最大値は浮動小数点では2^63へ丸められます。i64へ戻す比較では見逃します。
            if number as i128 != i128::from(value) {
                return Err(fail(NumericConversionFailure::Inexact));
            }
            Ok(result)
        }
        Value::F32(_) | Value::F64(_) => {
            let number = match value {
                Value::F32(value) => f64::from(value),
                Value::F64(value) => value,
                _ => unreachable!(),
            };
            match to {
                NumericType::Integer(ty) => {
                    if !number.is_finite() {
                        return Err(fail(NumericConversionFailure::NotFinite));
                    }
                    if number == 0.0 && number.is_sign_negative() {
                        return Err(fail(NumericConversionFailure::NegativeZero));
                    }
                    // 上限は含まない境界にします。i64最大値をf64で比較すると上へ丸められるためです。
                    let upper = (i128::from(ty.maximum()) + 1) as f64;
                    if number < ty.minimum() as f64 || number >= upper {
                        return Err(fail(NumericConversionFailure::OutOfRange));
                    }
                    if number.trunc() != number {
                        return Err(fail(NumericConversionFailure::Inexact));
                    }
                    Ok(Value::Integer(number as i64, ty))
                }
                NumericType::F32 => {
                    if number.is_nan() {
                        return Err(fail(NumericConversionFailure::NaN));
                    }
                    if number.is_finite() && number.abs() > f64::from(f32::MAX) {
                        return Err(fail(NumericConversionFailure::OutOfRange));
                    }
                    let result = number as f32;
                    if f64::from(result) != number {
                        return Err(fail(NumericConversionFailure::Inexact));
                    }
                    Ok(Value::F32(result))
                }
                NumericType::F64 => {
                    if number.is_nan() {
                        return Err(fail(NumericConversionFailure::NaN));
                    }
                    Ok(Value::F64(number))
                }
            }
        }
        _ => unreachable!("numeric operand type checked above"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_preserves_nan_payloads_and_signed_zero_bits() {
        for bits in [0, 0x80000000, 0x7fc01234, 0xffc01234, 0x7f801234] {
            let Value::F32(result) = convert(
                Value::F32(f32::from_bits(bits)),
                NumericType::F32,
                NumericType::F32,
            )
            .unwrap() else {
                panic!()
            };
            assert_eq!(result.to_bits(), bits);
        }
        for bits in [
            0,
            0x8000000000000000,
            0x7ff8000000001234,
            0xfff8000000001234,
            0x7ff0000000001234,
        ] {
            let Value::F64(result) = convert(
                Value::F64(f64::from_bits(bits)),
                NumericType::F64,
                NumericType::F64,
            )
            .unwrap() else {
                panic!()
            };
            assert_eq!(result.to_bits(), bits);
        }
    }

    #[test]
    fn exact_float_width_changes_preserve_subnormals_and_special_value_signs() {
        for value in [
            f32::from_bits(1),
            -f32::from_bits(1),
            f32::MIN_POSITIVE,
            f32::MAX,
            -f32::MAX,
            0.0,
            -0.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
        ] {
            let Value::F64(wide) =
                convert(Value::F32(value), NumericType::F32, NumericType::F64).unwrap()
            else {
                panic!()
            };
            assert_eq!(wide.to_bits(), f64::from(value).to_bits());
            let Value::F32(back) =
                convert(Value::F64(wide), NumericType::F64, NumericType::F32).unwrap()
            else {
                panic!()
            };
            assert_eq!(back.to_bits(), value.to_bits());
        }
    }
}
