use std::fmt::Write;

use crate::{codegen::IntegerBinaryOp, types::IntegerType};

pub(super) fn emit_support(op: IntegerBinaryOp, ty: IntegerType, output: &mut String) {
    if ty == IntegerType::U64 {
        return super::unsigned::emit_binary(op, output);
    }
    writeln!(
        output,
        "function l ${}(l %left, l %right, l %origin, l %origin_len) {{\n@start",
        op.helper(ty)
    )
    .unwrap();
    let instruction = match op {
        IntegerBinaryOp::Add
        | IntegerBinaryOp::Subtract
        | IntegerBinaryOp::Multiply
        | IntegerBinaryOp::Divide => unreachable!("u64 operations are emitted above"),
        IntegerBinaryOp::BitAnd => "and",
        IntegerBinaryOp::BitOr => "or",
        IntegerBinaryOp::BitXor => "xor",
        IntegerBinaryOp::Remainder => {
            output.push_str("  %zero =w ceql %right, 0\n  jnz %zero, @trap, @special\n@special\n  %negative_one =w ceql %right, -1\n  jnz %negative_one, @zero_result, @ok\n@zero_result\n  ret 0\n@trap\n  call $primer_fail_remainder_by_zero(l %origin, l %origin_len)\n  hlt\n@ok\n");
            "rem"
        }
        IntegerBinaryOp::ShiftLeft | IntegerBinaryOp::ShiftRight => {
            writeln!(output, "  %negative =w csltl %right, 0\n  %wide =w csgel %right, {}\n  %bad_count =w or %negative, %wide\n  jnz %bad_count, @count, @bounds\n@count\n  call $primer_fail_invalid_shift_count(l %origin, l %origin_len)\n  hlt\n@bounds", ty.bit_width()).unwrap();
            if op == IntegerBinaryOp::ShiftLeft {
                writeln!(output, "  %minimum =l sar {}, %right\n  %maximum =l shr {}, %right\n  %below =w csltl %left, %minimum\n  %above =w csgtl %left, %maximum\n  %overflow =w or %below, %above\n  jnz %overflow, @overflow_failure, @ok\n@overflow_failure\n  call $primer_fail_integer_overflow(l %origin, l %origin_len)\n  hlt\n@ok", ty.minimum(), ty.maximum()).unwrap();
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
