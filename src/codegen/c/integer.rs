use std::fmt::Write;

use crate::{codegen::IntegerBinaryOp, types::IntegerType};

pub(super) fn emit_support(op: IntegerBinaryOp, ty: IntegerType, output: &mut String) {
    writeln!(
        output,
        "static int64_t {}(int64_t left, int64_t right) {{",
        op.helper(ty)
    )
    .unwrap();
    match op {
        IntegerBinaryOp::BitAnd | IntegerBinaryOp::BitOr | IntegerBinaryOp::BitXor => {
            let symbol = match op {
                IntegerBinaryOp::BitAnd => "&",
                IntegerBinaryOp::BitOr => "|",
                IntegerBinaryOp::BitXor => "^",
                _ => unreachable!(),
            };
            writeln!(output, "    return left {symbol} right;").unwrap();
        }
        IntegerBinaryOp::Remainder => {
            // Cの最小値 / -1は未定義ですが、Primerの余りは0になります。
            output.push_str("    if (right == 0) abort();\n    if (right == -1) return 0;\n    return left % right;\n");
        }
        IntegerBinaryOp::ShiftLeft | IntegerBinaryOp::ShiftRight => {
            writeln!(
                output,
                "    if (right < 0 || right >= {}) abort();",
                ty.bit_width()
            )
            .unwrap();
            if op == IntegerBinaryOp::ShiftLeft {
                // 負数の右シフトや符号ビットへの左シフトをCの実装に委ねません。
                let lower = if ty.is_signed() {
                    format!("-1 - (int64_t)((uint64_t){}LL >> right)", -1 - ty.minimum())
                } else {
                    "0".to_owned()
                };
                writeln!(
                    output,
                    "    if (left < ({lower}) || left > ({}LL >> right)) abort();",
                    ty.maximum()
                )
                .unwrap();
                if ty.bit_width() == 64 {
                    output.push_str("    if (right == 63) return left == 0 ? 0 : INT64_MIN;\n");
                }
                output.push_str("    return left * (INT64_C(1) << right);\n");
            } else {
                output.push_str("    return left >= 0 ? (int64_t)((uint64_t)left >> right)\n        : -1 - (int64_t)((uint64_t)(-1 - left) >> right);\n");
            }
        }
    }
    output.push_str("}\n\n");
}
