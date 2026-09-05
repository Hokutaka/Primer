use std::fmt::Write;

use crate::{codegen::IntegerBinaryOp, types::IntegerType};

pub(super) fn emit_support(op: IntegerBinaryOp, ty: IntegerType, output: &mut String) {
    writeln!(
        output,
        "function l ${}(l %left, l %right) {{\n@start",
        op.helper(ty)
    )
    .unwrap();
    let instruction = match op {
        IntegerBinaryOp::BitAnd => "and",
        IntegerBinaryOp::BitOr => "or",
        IntegerBinaryOp::BitXor => "xor",
        IntegerBinaryOp::Remainder => {
            output.push_str("  %zero =w ceql %right, 0\n  jnz %zero, @trap, @special\n@special\n  %negative_one =w ceql %right, -1\n  jnz %negative_one, @zero_result, @ok\n@zero_result\n  ret 0\n@trap\n  call $abort()\n  hlt\n@ok\n");
            "rem"
        }
        IntegerBinaryOp::ShiftLeft | IntegerBinaryOp::ShiftRight => {
            writeln!(output, "  %negative =w csltl %right, 0\n  %wide =w csgel %right, {}\n  %bad_count =w or %negative, %wide\n  jnz %bad_count, @trap, @bounds\n@trap\n  call $abort()\n  hlt\n@bounds", ty.bit_width()).unwrap();
            if op == IntegerBinaryOp::ShiftLeft {
                writeln!(output, "  %minimum =l sar {}, %right\n  %maximum =l shr {}, %right\n  %below =w csltl %left, %minimum\n  %above =w csgtl %left, %maximum\n  %overflow =w or %below, %above\n  jnz %overflow, @trap, @ok\n@ok", ty.minimum(), ty.maximum()).unwrap();
                "shl"
            } else {
                "sar"
            }
        }
    };
    writeln!(
        output,
        "  %result =l {instruction} %left, %right\n  ret %result\n}}\n"
    )
    .unwrap();
}
