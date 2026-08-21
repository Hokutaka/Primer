use super::ir::{BinaryOp, FloatConstant, Instruction, Module};

pub fn emit(module: &Module) -> String {
    let mut output = initial_data();

    for constant in &module.float_constants {
        emit_float_constant(constant, &mut output);
    }

    output.push_str("\n.text\n");

    output.push_str(".globl main\n");

    output.push_str(".p2align 4\n");

    output.push_str("main:\n");

    output.push_str("  pushq %rbp\n");

    output.push_str("  movq %rsp, %rbp\n");

    output.push_str(&format!("  subq ${}, %rsp\n", module.frame_size,));

    for instruction in &module.instructions {
        emit_instruction(instruction, &mut output);
    }

    output.push_str("  xorl %eax, %eax\n");

    output.push_str(&format!("  addq ${}, %rsp\n", module.frame_size,));

    output.push_str("  popq %rbp\n");

    output.push_str("  retq\n");

    output
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

fn emit_instruction(instruction: &Instruction, output: &mut String) {
    match instruction {
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

        Instruction::LoadF32Constant(id) => {
            output.push_str(&format!("  movss .Lprimer_f32_{id}(%rip), %xmm0\n"));
        }

        Instruction::LoadF64Constant(id) => {
            output.push_str(&format!("  movsd .Lprimer_f64_{id}(%rip), %xmm0\n"));
        }

        Instruction::NegI64 => {
            output.push_str("  negq %rax\n");
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

        Instruction::SignExtendRax => {
            output.push_str("  cqto\n");
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
    }
}

fn initial_data() -> String {
    let mut output = String::new();

    output.push_str(".section .rdata,\"dr\"\n");

    output.push_str(".Lprimer_fmt_i64:\n");

    output.push_str("  .asciz \"%lld\\n\"\n");

    output.push_str(".Lprimer_fmt_f32:\n");

    output.push_str("  .asciz \"%.9g\\n\"\n");

    output.push_str(".Lprimer_fmt_f64:\n");

    output.push_str("  .asciz \"%.17g\\n\"\n");

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
