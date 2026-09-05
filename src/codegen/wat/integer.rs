use std::fmt::Write;

use crate::{codegen::IntegerBinaryOp, types::IntegerType};

pub(super) fn emit_support(op: IntegerBinaryOp, ty: IntegerType, output: &mut String) {
    writeln!(
        output,
        "  (func ${} (param $left i64) (param $right i64) (result i64)",
        op.helper(ty)
    )
    .unwrap();
    let instruction = match op {
        IntegerBinaryOp::BitAnd => "and",
        IntegerBinaryOp::BitOr => "or",
        IntegerBinaryOp::BitXor => "xor",
        IntegerBinaryOp::Remainder => {
            // rem_sは最小値 % -1でも0を返し、ゼロ除算だけがtrapになります。
            "rem_s"
        }
        IntegerBinaryOp::ShiftLeft | IntegerBinaryOp::ShiftRight => {
            writeln!(output, "    local.get $right\n    i64.const 0\n    i64.lt_s\n    local.get $right\n    i64.const {}\n    i64.ge_s\n    i32.or\n    if\n      unreachable\n    end", ty.bit_width()).unwrap();
            if op == IntegerBinaryOp::ShiftLeft {
                writeln!(output, "    local.get $left\n    i64.const {}\n    local.get $right\n    i64.shr_s\n    i64.lt_s\n    local.get $left\n    i64.const {}\n    local.get $right\n    i64.shr_u\n    i64.gt_s\n    i32.or\n    if\n      unreachable\n    end", ty.minimum(), ty.maximum()).unwrap();
                "shl"
            } else {
                "shr_s"
            }
        }
    };
    writeln!(
        output,
        "    local.get $left\n    local.get $right\n    i64.{instruction}\n  )\n"
    )
    .unwrap();
}
