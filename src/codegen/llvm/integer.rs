use std::fmt::Write;

use crate::{codegen::IntegerBinaryOp, types::IntegerType};

pub(super) fn emit_support(op: IntegerBinaryOp, ty: IntegerType, output: &mut String) {
    writeln!(
        output,
        "define internal i64 @{}(i64 %left, i64 %right) {{\nentry:",
        op.helper(ty)
    )
    .unwrap();
    let instruction = match op {
        IntegerBinaryOp::BitAnd => "and",
        IntegerBinaryOp::BitOr => "or",
        IntegerBinaryOp::BitXor => "xor",
        IntegerBinaryOp::Remainder => {
            output.push_str("  %zero = icmp eq i64 %right, 0\n  br i1 %zero, label %trap, label %special\nspecial:\n  %negative_one = icmp eq i64 %right, -1\n  br i1 %negative_one, label %zero_result, label %ok\nzero_result:\n  ret i64 0\ntrap:\n  call void @llvm.trap()\n  unreachable\nok:\n");
            "srem"
        }
        IntegerBinaryOp::ShiftLeft | IntegerBinaryOp::ShiftRight => {
            writeln!(output, "  %negative = icmp slt i64 %right, 0\n  %wide = icmp sge i64 %right, {}\n  %bad_count = or i1 %negative, %wide\n  br i1 %bad_count, label %trap, label %bounds\ntrap:\n  call void @llvm.trap()\n  unreachable\nbounds:", ty.bit_width()).unwrap();
            if op == IntegerBinaryOp::ShiftLeft {
                // count検査の後に境界を求め、範囲外のシフトによるpoisonも避けます。
                writeln!(output, "  %minimum = ashr i64 {}, %right\n  %maximum = lshr i64 {}, %right\n  %below = icmp slt i64 %left, %minimum\n  %above = icmp sgt i64 %left, %maximum\n  %overflow = or i1 %below, %above\n  br i1 %overflow, label %trap, label %ok\nok:", ty.minimum(), ty.maximum()).unwrap();
                "shl"
            } else {
                "ashr"
            }
        }
    };
    writeln!(
        output,
        "  %result = {instruction} i64 %left, %right\n  ret i64 %result\n}}\n"
    )
    .unwrap();
}
