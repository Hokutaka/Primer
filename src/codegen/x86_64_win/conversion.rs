use crate::{codegen::NumericConversion, types::NumericType};

pub(super) fn emit(conversion: NumericConversion, label: usize, prefix: &str, output: &mut String) {
    let bad = format!(".Lprimer_{prefix}_convert_bad_{label}");
    let done = format!(".Lprimer_{prefix}_convert_done_{label}");
    match (conversion.from, conversion.to) {
        (NumericType::Integer(_), NumericType::F32 | NumericType::F64) => {
            output.push_str("  movq %rax, %r10\n");
            if conversion.to == NumericType::F32 {
                output.push_str("  cvtsi2ssq %rax, %xmm0\n  cvtss2sd %xmm0, %xmm2\n");
            } else {
                output.push_str("  cvtsi2sdq %rax, %xmm0\n  movapd %xmm0, %xmm2\n");
            }
            // i64の最大値が2^63へ丸められても、逆変換の前に検出します。
            load_bound(9223372036854775808.0, output);
            output.push_str(&format!("  ucomisd %xmm1, %xmm2\n  jae {bad}\n  cvttsd2siq %xmm2, %rax\n  cmpq %r10, %rax\n  jne {bad}\n"));
        }
        (NumericType::F32 | NumericType::F64, NumericType::Integer(ty)) => {
            if conversion.from == NumericType::F32 {
                output.push_str("  cvtss2sd %xmm0, %xmm2\n");
            } else {
                output.push_str("  movapd %xmm0, %xmm2\n");
            }
            output.push_str(&format!("  ucomisd %xmm2, %xmm2\n  jp {bad}\n  movq %xmm2, %r11\n  movabsq $-9223372036854775808, %r10\n  cmpq %r10, %r11\n  je {bad}\n"));
            load_bound(ty.minimum() as f64, output);
            output.push_str(&format!("  ucomisd %xmm1, %xmm2\n  jb {bad}\n"));
            load_bound((i128::from(ty.maximum()) + 1) as f64, output);
            output.push_str(&format!("  ucomisd %xmm1, %xmm2\n  jae {bad}\n  cvttsd2siq %xmm2, %rax\n  cvtsi2sdq %rax, %xmm1\n  ucomisd %xmm1, %xmm2\n  jne {bad}\n"));
        }
        (NumericType::F32, NumericType::F64) => {
            output.push_str(&format!(
                "  ucomiss %xmm0, %xmm0\n  jp {bad}\n  cvtss2sd %xmm0, %xmm0\n"
            ));
        }
        (NumericType::F64, NumericType::F32) => {
            output.push_str(&format!("  ucomisd %xmm0, %xmm0\n  jp {bad}\n  movapd %xmm0, %xmm2\n  cvtsd2ss %xmm0, %xmm0\n  cvtss2sd %xmm0, %xmm1\n  ucomisd %xmm1, %xmm2\n  jne {bad}\n"));
        }
        _ => unreachable!("identity and integer conversions are lowered separately"),
    }
    output.push_str(&format!("  jmp {done}\n{bad}:\n  ud2\n{done}:\n"));
}

fn load_bound(value: f64, output: &mut String) {
    output.push_str(&format!(
        "  movabsq ${}, %r11\n  movq %r11, %xmm1\n",
        value.to_bits() as i64
    ));
}
