use crate::{
    codegen::{IntegerBinaryOp as Op, NumericConversion},
    types::{IntegerType, NumericType as N},
};
use std::fmt::Write;
pub(super) fn emit_binary(op: Op, output: &mut String) {
    writeln!(
        output,
        "  (func ${} (param $left i64) (param $right i64) (result i64)",
        op.helper(IntegerType::U64)
    )
    .unwrap();
    let instruction = match op {
        Op::Add => {
            output.push_str("    local.get $left\n    i64.const -1\n    local.get $right\n    i64.sub\n    i64.gt_u\n    if\n      unreachable\n    end\n");
            "add"
        }
        Op::Subtract => {
            output.push_str("    local.get $left\n    local.get $right\n    i64.lt_u\n    if\n      unreachable\n    end\n");
            "sub"
        }
        Op::Multiply => {
            output.push_str("    local.get $right\n    i64.eqz\n    if\n    else\n      local.get $left\n      i64.const -1\n      local.get $right\n      i64.div_u\n      i64.gt_u\n      if\n        unreachable\n      end\n    end\n");
            "mul"
        }
        Op::Divide | Op::Remainder => {
            output.push_str(
                "    local.get $right\n    i64.eqz\n    if\n      unreachable\n    end\n",
            );
            if op == Op::Divide { "div_u" } else { "rem_u" }
        }
        Op::BitAnd => "and",
        Op::BitOr => "or",
        Op::BitXor => "xor",
        Op::ShiftLeft | Op::ShiftRight => {
            output.push_str("    local.get $right\n    i64.const 64\n    i64.ge_u\n    if\n      unreachable\n    end\n");
            if op == Op::ShiftLeft {
                output.push_str("    local.get $left\n    i64.const -1\n    local.get $right\n    i64.shr_u\n    i64.gt_u\n    if\n      unreachable\n    end\n");
                "shl"
            } else {
                "shr_u"
            }
        }
    };
    writeln!(
        output,
        "    local.get $left\n    local.get $right\n    i64.{instruction}\n  )\n"
    )
    .unwrap();
}
pub(super) fn emit_conversion(c: NumericConversion, output: &mut String) {
    let from = super::conversion::type_name(c.from);
    let to = super::conversion::type_name(c.to);
    writeln!(output,"  (func ${} (param $value {from}) (result {to})\n    (local $result {to}) (local $number f64)",c.helper()).unwrap();
    match (c.from, c.to) {
        (N::Integer(from), N::Integer(to)) => {
            output.push_str("    local.get $value\n");
            if from == IntegerType::U64 {
                writeln!(output, "    i64.const {}\n    i64.gt_u", to.maximum()).unwrap();
            } else {
                output.push_str("    i64.const 0\n    i64.lt_s\n");
            }
            output.push_str(
                "    if\n      unreachable\n    end\n    local.get $value\n    local.set $result\n",
            );
        }
        (N::Integer(_), N::F32 | N::F64) => {
            writeln!(
                output,
                "    local.get $value\n    {to}.convert_i64_u\n    local.tee $result"
            )
            .unwrap();
            if c.to == N::F32 {
                output.push_str("    f64.promote_f32\n");
            }
            output.push_str("    local.set $number\n    local.get $number\n    f64.const 18446744073709551616\n    f64.ge\n    if\n      unreachable\n    end\n    local.get $number\n    i64.trunc_f64_u\n    local.get $value\n    i64.ne\n    if\n      unreachable\n    end\n");
        }
        (N::F32 | N::F64, N::Integer(_)) => {
            output.push_str("    local.get $value\n");
            if c.from == N::F32 {
                output.push_str("    f64.promote_f32\n");
            }
            output.push_str("    local.set $number\n    local.get $number\n    f64.const 0\n    f64.ge\n    local.get $number\n    f64.const 18446744073709551616\n    f64.lt\n    i32.and\n    i32.eqz\n    if\n      unreachable\n    end\n    local.get $number\n    i64.reinterpret_f64\n    i64.const -9223372036854775808\n    i64.eq\n    if\n      unreachable\n    end\n    local.get $number\n    i64.trunc_f64_u\n    local.tee $result\n    f64.convert_i64_u\n    local.get $number\n    f64.ne\n    if\n      unreachable\n    end\n");
        }
        _ => unreachable!("u64 conversion"),
    }
    output.push_str("    local.get $result\n  )\n");
}
