use super::ir::{BinaryOp, CompareOp, FloatConstant, Function, Instruction, Module, Type};

pub fn emit(module: &Module) -> String {
    let mut output = initial_data(uses_bool_print(module));

    for constant in &module.float_constants {
        emit_float_constant(constant, &mut output);
    }

    output.push_str("\n.text\n");

    for function in &module.functions {
        emit_function(function, module, &mut output);
        output.push('\n');
    }

    output.push_str(".globl main\n");

    output.push_str(".p2align 4\n");

    output.push_str("main:\n");

    output.push_str("  pushq %rbp\n");

    output.push_str("  movq %rsp, %rbp\n");

    output.push_str(&format!("  subq ${}, %rsp\n", module.frame_size,));

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
    output.push_str(&format!("  subq ${}, %rsp\n", function.frame_size));
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

        Instruction::StoreParameter { index, ty, offset } => {
            emit_store_parameter(*index, *ty, *offset, output);
        }

        Instruction::Call {
            function_id,
            arguments,
        } => {
            for (index, (ty, offset)) in arguments.iter().enumerate() {
                emit_load_argument(index, *ty, *offset, output);
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
        Type::Bool | Type::I64 => {
            let register = ["%rcx", "%rdx", "%r8", "%r9"][index];
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
        Type::Bool | Type::I64 => {
            let register = ["%rcx", "%rdx", "%r8", "%r9"][index];
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
