use super::ir::{
    Argument, BinaryOp, CompareOp, FloatConstant, Function, Instruction, Module, Type,
};

pub fn emit(module: &Module) -> String {
    let mut output = initial_data(uses_bool_print(module));
    if module.uses_strings {
        super::string::emit_data(module, &mut output);
    }

    for constant in &module.float_constants {
        emit_float_constant(constant, &mut output);
    }

    output.push_str("\n.text\n");
    if module.uses_strings {
        output.push_str(super::string::SUPPORT);
    }

    for function in &module.functions {
        emit_function(function, module, &mut output);
        output.push('\n');
    }

    output.push_str(".globl main\n");

    output.push_str(".p2align 4\n");

    output.push_str("main:\n");

    output.push_str("  pushq %rbp\n");

    output.push_str("  movq %rsp, %rbp\n");

    emit_stack_allocation(module.frame_size, &mut output);
    if module.uses_strings {
        // 固定ターゲットのWindows CRTで、最初のPrimer処理より前にLF変換を止めます。
        output.push_str("  movl $1, %ecx\n  movl $32768, %edx\n  callq _setmode\n  cmpl $-1, %eax\n  jne .Lstdout_ready\n  movl $1, %eax\n");
        emit_epilogue(module.frame_size, false, &mut output);
        output.push_str(".Lstdout_ready:\n");
    }

    for instruction in &module.instructions {
        emit_instruction(instruction, module.frame_size, "main", module, &mut output);
    }

    if let Some(function_id) = module.explicit_main {
        output.push_str(&format!(
            "  callq {}\n",
            function_name(&module.functions[function_id])
        ));
    }

    emit_epilogue(module.frame_size, true, &mut output);

    output
}

fn emit_function(function: &Function, module: &Module, output: &mut String) {
    output.push_str(".p2align 4\n");
    output.push_str(&format!("{}:\n", function_name(function)));
    output.push_str("  pushq %rbp\n");
    output.push_str("  movq %rsp, %rbp\n");
    emit_stack_allocation(function.frame_size, output);
    for instruction in &function.instructions {
        emit_instruction(
            instruction,
            function.frame_size,
            &format!("fn_{}", function.id),
            module,
            output,
        );
    }
}

// Windowsのガードページを飛び越さないよう、確保前に各ページを検査します。
// __chkstkは引数レジスタとraxを保持するため、引数を保存する前でも呼び出せます。
fn emit_stack_allocation(frame_size: usize, output: &mut String) {
    if frame_size >= 4096 {
        output.push_str(&format!(
            "  movq ${frame_size}, %rax\n  callq __chkstk\n  subq %rax, %rsp\n"
        ));
    } else {
        output.push_str(&format!("  subq ${frame_size}, %rsp\n"));
    }
}

fn function_name(function: &Function) -> String {
    format!("primer_fn_{}_{}", function.name, function.id)
}

fn emit_epilogue(frame_size: usize, zero_result: bool, output: &mut String) {
    if zero_result {
        output.push_str("  xorl %eax, %eax\n");
    }
    output.push_str(&format!("  addq ${frame_size}, %rsp\n"));
    output.push_str("  popq %rbp\n");
    output.push_str("  retq\n");
}

fn emit_float_constant(constant: &FloatConstant, output: &mut String) {
    match constant {
        FloatConstant::F32 { id, bits } => {
            output.push_str(".p2align 2\n");

            output.push_str(&format!(".Lprimer_f32_{id}:\n"));

            output.push_str(&format!("  .long 0x{bits:08X}\n"));
        }

        FloatConstant::F64 { id, bits } => {
            output.push_str(".p2align 3\n");

            output.push_str(&format!(".Lprimer_f64_{id}:\n"));

            output.push_str(&format!("  .quad 0x{bits:016X}\n"));
        }
    }
}

fn emit_instruction(
    instruction: &Instruction,
    frame_size: usize,
    label_prefix: &str,
    module: &Module,
    output: &mut String,
) {
    match instruction {
        Instruction::LoadStringLength => output.push_str("  movq (%rax), %rax\n"),
        Instruction::LoadStringConstant(id) => {
            output.push_str(&format!("  leaq .Lprimer_string_{id}(%rip), %rax\n"))
        }
        Instruction::CompareString { left_offset, equal } => {
            output.push_str(&format!(
                "  movq %rax, %rdx\n  movq {left_offset}(%rbp), %rcx\n  callq primer_string_equal\n"
            ));
            if !equal {
                output.push_str("  xorl $1, %eax\n");
            }
        }
        Instruction::PrintString => {
            output.push_str("  movq %rax, %rcx\n  callq primer_print_string\n")
        }
        Instruction::ConvertNumeric { conversion, label } => {
            super::conversion::emit(*conversion, *label, label_prefix, output)
        }
        Instruction::BitNot { mask } => {
            output.push_str(&format!("  movabsq ${mask}, %r11\n  xorq %r11, %rax\n"));
        }
        Instruction::IntegerBinary { op, ty, label } => {
            use crate::codegen::IntegerBinaryOp;
            let bad = format!(".Lprimer_{label_prefix}_integer_bad_{label}");
            let done = format!(".Lprimer_{label_prefix}_integer_done_{label}");
            match op {
                IntegerBinaryOp::BitAnd => output.push_str("  andq %rcx, %rax\n"),
                IntegerBinaryOp::BitOr => output.push_str("  orq %rcx, %rax\n"),
                IntegerBinaryOp::BitXor => output.push_str("  xorq %rcx, %rax\n"),
                IntegerBinaryOp::Remainder => {
                    let divide = format!(".Lprimer_{label_prefix}_integer_rem_{label}");
                    output.push_str(&format!("  testq %rcx, %rcx\n  je {bad}\n  cmpq $-1, %rcx\n  jne {divide}\n  xorq %rax, %rax\n  jmp {done}\n{divide}:\n  cqto\n  idivq %rcx\n  movq %rdx, %rax\n  jmp {done}\n{bad}:\n  ud2\n{done}:\n"));
                }
                IntegerBinaryOp::ShiftLeft | IntegerBinaryOp::ShiftRight => {
                    output.push_str(&format!(
                        "  testq %rcx, %rcx\n  js {bad}\n  cmpq ${}, %rcx\n  jge {bad}\n",
                        ty.bit_width()
                    ));
                    if *op == IntegerBinaryOp::ShiftLeft {
                        output.push_str(&format!("  movabsq ${}, %r11\n  sarq %cl, %r11\n  cmpq %r11, %rax\n  jl {bad}\n  movabsq ${}, %r11\n  shrq %cl, %r11\n  cmpq %r11, %rax\n  jg {bad}\n  shlq %cl, %rax\n", ty.minimum(), ty.maximum()));
                    } else {
                        output.push_str("  sarq %cl, %rax\n");
                    }
                    output.push_str(&format!("  jmp {done}\n{bad}:\n  ud2\n{done}:\n"));
                }
            }
        }
        Instruction::CheckIntegerRange { ty, label } => {
            let bad = format!(".Lprimer_{label_prefix}_range_bad_{label}");
            let done = format!(".Lprimer_{label_prefix}_range_ok_{label}");
            output.push_str(&format!("  # semantic {}, storage i64\n  movabsq ${}, %r11\n  cmpq %r11, %rax\n  jl {bad}\n  movabsq ${}, %r11\n  cmpq %r11, %rax\n  jle {done}\n{bad}:\n  ud2\n{done}:\n", ty.name(), ty.minimum(), ty.maximum()));
        }
        Instruction::Label { id, name } => {
            output.push_str(&format!("{}: # {name}\n", block_label(label_prefix, *id)));
        }

        Instruction::JumpIfZero(label) => {
            output.push_str("  testq %rax, %rax\n");
            output.push_str(&format!("  je {}\n", block_label(label_prefix, *label)));
        }

        Instruction::Jump(label) => {
            output.push_str(&format!("  jmp {}\n", block_label(label_prefix, *label)));
        }

        Instruction::MovI64ImmediateToRax(value) => {
            output.push_str(&format!("  movabsq ${value}, %rax\n"));
        }

        Instruction::LoadI64FromStack(offset) | Instruction::LoadI64ScratchToRax(offset) => {
            output.push_str(&format!("  movq {offset}(%rbp), %rax\n"));
        }

        Instruction::StoreI64ToStack(offset) => {
            output.push_str(&format!("  movq %rax, {offset}(%rbp)\n"));
        }

        Instruction::LoadF32FromStack(offset) | Instruction::LoadF32ScratchToXmm0(offset) => {
            output.push_str(&format!("  movss {offset}(%rbp), %xmm0\n"));
        }

        Instruction::StoreF32ToStack(offset) => {
            output.push_str(&format!("  movss %xmm0, {offset}(%rbp)\n"));
        }

        Instruction::LoadF64FromStack(offset) | Instruction::LoadF64ScratchToXmm0(offset) => {
            output.push_str(&format!("  movsd {offset}(%rbp), %xmm0\n"));
        }

        Instruction::StoreF64ToStack(offset) => {
            output.push_str(&format!("  movsd %xmm0, {offset}(%rbp)\n"));
        }

        Instruction::CheckedArrayLoad {
            ty,
            base_offset,
            length,
            label,
        } => {
            let trap = format!(".Lprimer_{label_prefix}_array_oob_{label}");
            let done = format!(".Lprimer_{label_prefix}_array_done_{label}");
            output.push_str("  testq %rax, %rax\n");
            output.push_str(&format!("  js {trap}\n"));
            output.push_str(&format!("  cmpq ${length}, %rax\n"));
            output.push_str(&format!("  jge {trap}\n"));
            output.push_str("  negq %rax\n");
            match ty {
                Type::String | Type::Bool | Type::I64 => {
                    output.push_str(&format!("  movq {base_offset}(%rbp,%rax,8), %rax\n"));
                }
                Type::F32 => {
                    output.push_str(&format!("  movss {base_offset}(%rbp,%rax,8), %xmm0\n"));
                }
                Type::F64 => {
                    output.push_str(&format!("  movsd {base_offset}(%rbp,%rax,8), %xmm0\n"));
                }
            }
            output.push_str(&format!("  jmp {done}\n"));
            output.push_str(&format!("{trap}:\n"));
            output.push_str("  ud2\n");
            output.push_str(&format!("{done}:\n"));
        }

        Instruction::CheckedArrayCopy {
            base_offset,
            length,
            element_slots,
            destination_offset,
            label,
        } => {
            let trap = format!(".Lprimer_{label_prefix}_array_oob_{label}");
            let done = format!(".Lprimer_{label_prefix}_array_done_{label}");
            output.push_str("  testq %rax, %rax\n");
            output.push_str(&format!("  js {trap}\n"));
            output.push_str(&format!("  cmpq ${length}, %rax\n"));
            output.push_str(&format!("  jge {trap}\n"));
            output.push_str(&format!("  imulq $-{}, %rax\n", element_slots));
            for slot in 0..*element_slots {
                let source = base_offset - 8 * slot as isize;
                let destination = destination_offset - 8 * slot as isize;
                output.push_str(&format!("  movq {source}(%rbp,%rax,8), %rcx\n"));
                output.push_str(&format!("  movq %rcx, {destination}(%rbp)\n"));
            }
            output.push_str(&format!("  jmp {done}\n"));
            output.push_str(&format!("{trap}:\n"));
            output.push_str("  ud2\n");
            output.push_str(&format!("{done}:\n"));
        }

        Instruction::CheckedArrayAddress {
            base_offset,
            base_is_pointer,
            length,
            element_slots,
            destination_offset,
            label,
        } => {
            let trap = format!(".Lprimer_{label_prefix}_array_oob_{label}");
            let done = format!(".Lprimer_{label_prefix}_array_done_{label}");
            output.push_str("  testq %rax, %rax\n");
            output.push_str(&format!("  js {trap}\n"));
            output.push_str(&format!("  cmpq ${length}, %rax\n"));
            output.push_str(&format!("  jge {trap}\n"));
            output.push_str(&format!("  imulq $-{}, %rax\n", element_slots));
            if *base_is_pointer {
                output.push_str(&format!("  movq {base_offset}(%rbp), %rcx\n"));
                output.push_str("  leaq (%rcx,%rax,8), %rcx\n");
            } else {
                output.push_str(&format!("  leaq {base_offset}(%rbp,%rax,8), %rcx\n"));
            }
            output.push_str(&format!("  movq %rcx, {destination_offset}(%rbp)\n"));
            output.push_str(&format!("  jmp {done}\n"));
            output.push_str(&format!("{trap}:\n"));
            output.push_str("  ud2\n");
            output.push_str(&format!("{done}:\n"));
        }

        Instruction::StoreI64ToPointer(pointer_offset) => {
            output.push_str(&format!("  movq {pointer_offset}(%rbp), %rcx\n"));
            output.push_str("  movq %rax, (%rcx)\n");
        }
        Instruction::StoreF32ToPointer(pointer_offset) => {
            output.push_str(&format!("  movq {pointer_offset}(%rbp), %rcx\n"));
            output.push_str("  movss %xmm0, (%rcx)\n");
        }
        Instruction::StoreF64ToPointer(pointer_offset) => {
            output.push_str(&format!("  movq {pointer_offset}(%rbp), %rcx\n"));
            output.push_str("  movsd %xmm0, (%rcx)\n");
        }
        Instruction::CopyToPointer {
            source_offset,
            slots,
            pointer_offset,
        } => {
            output.push_str(&format!("  movq {pointer_offset}(%rbp), %r11\n"));
            for slot in 0..*slots {
                let source = source_offset - 8 * slot as isize;
                let destination = -8 * slot as isize;
                output.push_str(&format!("  movq {source}(%rbp), %r10\n"));
                output.push_str(&format!("  movq %r10, {destination}(%r11)\n"));
            }
        }

        Instruction::StoreParameter { index, ty, offset } => {
            emit_store_parameter(*index, *ty, *offset, output);
        }

        Instruction::StoreAggregateParameter {
            index,
            slots,
            destination_offset,
        } => {
            let register = integer_argument_register(*index);
            for slot in 0..*slots {
                let source = -8 * slot as isize;
                let destination = destination_offset - 8 * slot as isize;
                output.push_str(&format!("  movq {source}({register}), %r10\n"));
                output.push_str(&format!("  movq %r10, {destination}(%rbp)\n"));
            }
        }

        Instruction::StoreAggregateReturnPointer { offset } => {
            output.push_str(&format!("  movq %rax, {offset}(%rbp)\n"));
        }

        Instruction::CopyToAggregateReturn {
            source_offset,
            slots,
            pointer_offset,
        } => {
            output.push_str(&format!("  movq {pointer_offset}(%rbp), %r11\n"));
            for slot in 0..*slots {
                let source = source_offset - 8 * slot as isize;
                let destination = -8 * slot as isize;
                output.push_str(&format!("  movq {source}(%rbp), %r10\n"));
                output.push_str(&format!("  movq %r10, {destination}(%r11)\n"));
            }
        }

        Instruction::Call {
            function_id,
            arguments,
            aggregate_result_offset,
        } => {
            for (index, argument) in arguments.iter().enumerate() {
                match argument {
                    Argument::Scalar { ty, offset } => {
                        emit_load_argument(index, *ty, *offset, output)
                    }
                    Argument::Aggregate { offset } => {
                        let register = integer_argument_register(index);
                        output.push_str(&format!("  leaq {offset}(%rbp), {register}\n"));
                    }
                }
            }
            if let Some(offset) = aggregate_result_offset {
                output.push_str(&format!("  leaq {offset}(%rbp), %rax\n"));
            }
            output.push_str(&format!(
                "  callq {}\n",
                function_name(&module.functions[*function_id])
            ));
        }

        Instruction::Return => emit_epilogue(frame_size, false, output),

        Instruction::LoadF32Constant(id) => {
            output.push_str(&format!("  movss .Lprimer_f32_{id}(%rip), %xmm0\n"));
        }

        Instruction::LoadF64Constant(id) => {
            output.push_str(&format!("  movsd .Lprimer_f64_{id}(%rip), %xmm0\n"));
        }

        Instruction::NegI64 => {
            output.push_str("  negq %rax\n");
        }

        Instruction::TrapIfOverflow(label) => {
            let done = format!(".Lprimer_{label_prefix}_integer_ok_{label}");
            output.push_str(&format!("  jno {done}\n"));
            output.push_str("  ud2\n");
            output.push_str(&format!("{done}:\n"));
        }

        Instruction::NotBool => {
            output.push_str("  xorq $1, %rax\n");
        }

        Instruction::NegF32 => {
            output.push_str("  xorps .Lprimer_sign_f32(%rip), %xmm0\n");
        }

        Instruction::NegF64 => {
            output.push_str("  xorpd .Lprimer_sign_f64(%rip), %xmm0\n");
        }

        Instruction::MoveRaxToRcx => {
            output.push_str("  movq %rax, %rcx\n");
        }

        Instruction::I64Binary(op) => {
            output.push_str(match op {
                BinaryOp::Add => "  addq %rcx, %rax\n",
                BinaryOp::Subtract => "  subq %rcx, %rax\n",
                BinaryOp::Multiply => "  imulq %rcx, %rax\n",
                BinaryOp::Divide => {
                    unreachable!("i64 division is lowered to sign-extend + divide")
                }
            });
        }

        Instruction::CompareI64(op) => {
            output.push_str("  cmpq %rcx, %rax\n");
            output.push_str(compare_setcc(*op));
            output.push_str("  movzbq %al, %rax\n");
        }

        Instruction::SignExtendRax => {
            output.push_str("  cqto\n");
        }

        Instruction::TrapIfInvalidI64Division(label) => {
            let trap = format!(".Lprimer_{label_prefix}_division_trap_{label}");
            let done = format!(".Lprimer_{label_prefix}_division_ok_{label}");
            output.push_str("  testq %rcx, %rcx\n");
            output.push_str(&format!("  je {trap}\n"));
            output.push_str("  cmpq $-1, %rcx\n");
            output.push_str(&format!("  jne {done}\n"));
            output.push_str("  movabsq $-9223372036854775808, %rdx\n");
            output.push_str("  cmpq %rdx, %rax\n");
            output.push_str(&format!("  jne {done}\n"));
            output.push_str(&format!("{trap}:\n"));
            output.push_str("  ud2\n");
            output.push_str(&format!("{done}:\n"));
        }

        Instruction::DivideRaxByRcx => {
            output.push_str("  idivq %rcx\n");
        }

        Instruction::CopyXmm0ToXmm1F32 => {
            output.push_str("  movaps %xmm0, %xmm1\n");
        }

        Instruction::CopyXmm0ToXmm1F64 => {
            output.push_str("  movapd %xmm0, %xmm1\n");
        }

        Instruction::F32Binary(op) => {
            output.push_str(match op {
                BinaryOp::Add => "  addss %xmm1, %xmm0\n",
                BinaryOp::Subtract => "  subss %xmm1, %xmm0\n",
                BinaryOp::Multiply => "  mulss %xmm1, %xmm0\n",
                BinaryOp::Divide => "  divss %xmm1, %xmm0\n",
            });
        }

        Instruction::F64Binary(op) => {
            output.push_str(match op {
                BinaryOp::Add => "  addsd %xmm1, %xmm0\n",
                BinaryOp::Subtract => "  subsd %xmm1, %xmm0\n",
                BinaryOp::Multiply => "  mulsd %xmm1, %xmm0\n",
                BinaryOp::Divide => "  divsd %xmm1, %xmm0\n",
            });
        }

        Instruction::CompareF32(op) => {
            output.push_str("  ucomiss %xmm1, %xmm0\n");
            emit_float_comparison(*op, output);
        }

        Instruction::CompareF64(op) => {
            output.push_str("  ucomisd %xmm1, %xmm0\n");
            emit_float_comparison(*op, output);
        }

        Instruction::MoveRaxToRdx => {
            output.push_str("  movq %rax, %rdx\n");
        }

        Instruction::LoadFormatI64ToRcx => {
            output.push_str("  leaq .Lprimer_fmt_i64(%rip), %rcx\n");
        }

        Instruction::ConvertF32ToF64Argument => {
            output.push_str("  cvtss2sd %xmm0, %xmm1\n");
        }

        Instruction::MoveXmm1ToRdx => {
            output.push_str("  movq %xmm1, %rdx\n");
        }

        Instruction::LoadFormatF32ToRcx => {
            output.push_str("  leaq .Lprimer_fmt_f32(%rip), %rcx\n");
        }

        Instruction::CopyXmm0ToXmm1F64Scalar => {
            output.push_str("  movsd %xmm0, %xmm1\n");
        }

        Instruction::LoadFormatF64ToRcx => {
            output.push_str("  leaq .Lprimer_fmt_f64(%rip), %rcx\n");
        }

        Instruction::CallPrintf => {
            output.push_str("  callq printf\n");
        }

        Instruction::CallPrintBool => {
            output.push_str("  testq %rax, %rax\n");
            output.push_str("  leaq .Lprimer_bool_false(%rip), %rcx\n");
            output.push_str("  leaq .Lprimer_bool_true(%rip), %rdx\n");
            output.push_str("  cmovne %rdx, %rcx\n");
            output.push_str("  callq puts\n");
        }
    }
}

fn block_label(prefix: &str, id: usize) -> String {
    if prefix == "main" {
        format!(".Lprimer_block_{id}")
    } else {
        format!(".Lprimer_{prefix}_block_{id}")
    }
}

fn emit_store_parameter(index: usize, ty: Type, offset: isize, output: &mut String) {
    match ty {
        Type::String | Type::Bool | Type::I64 => {
            let register = integer_argument_register(index);
            output.push_str(&format!("  movq {register}, {offset}(%rbp)\n"));
        }
        Type::F32 => {
            output.push_str(&format!("  movss %xmm{index}, {offset}(%rbp)\n"));
        }
        Type::F64 => {
            output.push_str(&format!("  movsd %xmm{index}, {offset}(%rbp)\n"));
        }
    }
}

fn emit_load_argument(index: usize, ty: Type, offset: isize, output: &mut String) {
    match ty {
        Type::String | Type::Bool | Type::I64 => {
            let register = integer_argument_register(index);
            output.push_str(&format!("  movq {offset}(%rbp), {register}\n"));
        }
        Type::F32 => {
            output.push_str(&format!("  movss {offset}(%rbp), %xmm{index}\n"));
        }
        Type::F64 => {
            output.push_str(&format!("  movsd {offset}(%rbp), %xmm{index}\n"));
        }
    }
}

fn integer_argument_register(index: usize) -> &'static str {
    ["%rcx", "%rdx", "%r8", "%r9"][index]
}

fn initial_data(include_bool_text: bool) -> String {
    let mut output = String::new();

    output.push_str(".section .rdata,\"dr\"\n");

    output.push_str(".Lprimer_fmt_i64:\n");

    output.push_str("  .asciz \"%lld\\n\"\n");

    output.push_str(".Lprimer_fmt_f32:\n");

    output.push_str("  .asciz \"%.9g\\n\"\n");

    output.push_str(".Lprimer_fmt_f64:\n");

    output.push_str("  .asciz \"%.17g\\n\"\n");

    if include_bool_text {
        output.push_str(".Lprimer_bool_false:\n");
        output.push_str("  .asciz \"false\"\n");
        output.push_str(".Lprimer_bool_true:\n");
        output.push_str("  .asciz \"true\"\n");
    }

    // Unary minus masks.
    output.push_str(".p2align 4\n");

    output.push_str(".Lprimer_sign_f32:\n");

    output.push_str("  .long 0x80000000\n");

    output.push_str("  .long 0\n");

    output.push_str("  .long 0\n");

    output.push_str("  .long 0\n");

    output.push_str(".p2align 4\n");

    output.push_str(".Lprimer_sign_f64:\n");

    output.push_str("  .quad 0x8000000000000000\n");

    output.push_str("  .quad 0\n");

    output
}

fn compare_setcc(op: CompareOp) -> &'static str {
    match op {
        CompareOp::Equal => "  sete %al\n",
        CompareOp::NotEqual => "  setne %al\n",
        CompareOp::Less => "  setl %al\n",
        CompareOp::LessEqual => "  setle %al\n",
        CompareOp::Greater => "  setg %al\n",
        CompareOp::GreaterEqual => "  setge %al\n",
    }
}

fn emit_float_comparison(op: CompareOp, output: &mut String) {
    match op {
        CompareOp::Equal => {
            output.push_str("  sete %al\n");
            output.push_str("  setnp %cl\n");
            output.push_str("  andb %cl, %al\n");
        }
        CompareOp::NotEqual => {
            output.push_str("  setne %al\n");
            output.push_str("  setp %cl\n");
            output.push_str("  orb %cl, %al\n");
        }
        CompareOp::Less => {
            output.push_str("  setb %al\n");
            output.push_str("  setnp %cl\n");
            output.push_str("  andb %cl, %al\n");
        }
        CompareOp::LessEqual => {
            output.push_str("  setbe %al\n");
            output.push_str("  setnp %cl\n");
            output.push_str("  andb %cl, %al\n");
        }
        CompareOp::Greater => output.push_str("  seta %al\n"),
        CompareOp::GreaterEqual => output.push_str("  setae %al\n"),
    }

    output.push_str("  movzbq %al, %rax\n");
}

fn uses_bool_print(module: &Module) -> bool {
    module
        .instructions
        .iter()
        .chain(
            module
                .functions
                .iter()
                .flat_map(|function| function.instructions.iter()),
        )
        .any(|instruction| matches!(instruction, Instruction::CallPrintBool))
}
