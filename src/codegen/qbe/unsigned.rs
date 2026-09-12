use crate::{codegen::IntegerBinaryOp as Op, types::IntegerType};
use std::fmt::Write;
pub(super) fn emit_binary(op: Op, output: &mut String) {
    writeln!(
        output,
        "function l ${}(l %left, l %right, l %origin, l %origin_len) {{\n@start",
        op.helper(IntegerType::U64)
    )
    .unwrap();
    let instruction = match op {
        Op::Add => {
            output.push_str("  %limit =l sub -1, %right\n  %bad =w cugtl %left, %limit\n");
            "add"
        }
        Op::Subtract => {
            output.push_str("  %bad =w cultl %left, %right\n");
            "sub"
        }
        Op::Multiply => {
            output.push_str("  %zero =w ceql %right, 0\n  jnz %zero, @ok, @bounds\n@bounds\n  %limit =l udiv -1, %right\n  %bad =w cugtl %left, %limit\n");
            "mul"
        }
        Op::Divide | Op::Remainder => {
            output.push_str("  %bad =w ceql %right, 0\n");
            if op == Op::Divide { "udiv" } else { "urem" }
        }
        Op::ShiftLeft | Op::ShiftRight => {
            output.push_str("  %wide =w cugel %right, 64\n  jnz %wide, @count, @bounds\n@count\n  call $primer_fail_invalid_shift_count(l %origin, l %origin_len)\n  hlt\n@bounds\n");
            if op == Op::ShiftLeft {
                output.push_str("  %limit =l shr -1, %right\n  %bad =w cugtl %left, %limit\n");
                "shl"
            } else {
                output.push_str("  %bad =w copy 0\n");
                "shr"
            }
        }
        Op::BitAnd | Op::BitOr | Op::BitXor => {
            let instruction = match op {
                Op::BitAnd => "and",
                Op::BitOr => "or",
                _ => "xor",
            };
            writeln!(
                output,
                "  %result =l {instruction} %left, %right\n  ret %result\n}}\n"
            )
            .unwrap();
            return;
        }
    };
    output.push_str("  jnz %bad, @trap, @ok\n");
    super::failure::block(
        "trap",
        match op {
            Op::Divide => crate::runtime::FailureCode::DivisionByZero,
            Op::Remainder => crate::runtime::FailureCode::RemainderByZero,
            _ => crate::runtime::FailureCode::IntegerOverflow,
        },
        output,
    );
    writeln!(
        output,
        "@ok\n  %result =l {instruction} %left, %right\n  ret %result\n}}\n"
    )
    .unwrap();
}
