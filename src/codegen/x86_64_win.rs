use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Program, Stmt, Type, UnaryOp};
use crate::semantic::{Bindings, type_of_expr};

pub fn emit_x86_64_win_asm(program: &Program, bindings: &Bindings) -> String {
    let binding_slots = assign_binding_slots(program);

    // 少し多めにscratch領域を確保。
    // Primer v0.1は小さいので、まず単純さ優先。
    let scratch_count = count_program_expr_nodes(program).max(1);

    let scratch_base = binding_slots.len();

    // Windows x64 ABIではcall時に32-byte shadow spaceが必要。
    let local_bytes = 8 * (binding_slots.len() + scratch_count);

    let frame_size = align16(32 + local_bytes);

    let mut generator = Generator {
        data: initial_data(),
        text: String::new(),

        bindings,
        binding_slots,

        scratch_base,

        float_id: 0,
        frame_size,
    };

    generator.emit_program(program);

    format!("{}{}", generator.data, generator.text,)
}

struct Generator<'a> {
    data: String,
    text: String,

    bindings: &'a Bindings,

    binding_slots: HashMap<String, usize>,

    scratch_base: usize,

    float_id: usize,
    frame_size: usize,
}

impl Generator<'_> {
    fn emit_program(&mut self, program: &Program) {
        self.text.push_str("\n.text\n");

        self.text.push_str(".globl main\n");

        self.text.push_str(".p2align 4\n");

        self.text.push_str("main:\n");

        self.text.push_str("  pushq %rbp\n");

        self.text.push_str("  movq %rsp, %rbp\n");

        self.text
            .push_str(&format!("  subq ${}, %rsp\n", self.frame_size,));

        for statement in &program.statements {
            self.emit_statement(statement);
        }

        self.text.push_str("  xorl %eax, %eax\n");

        self.text
            .push_str(&format!("  addq ${}, %rsp\n", self.frame_size,));

        self.text.push_str("  popq %rbp\n");

        self.text.push_str("  retq\n");
    }

    fn emit_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Binding { name, value, .. } => {
                let ty = self
                    .bindings
                    .get(name)
                    .copied()
                    .expect("binding must have been resolved by type checker");

                self.emit_expr(value, Some(ty), 0);

                let offset = self.binding_offset(name);

                match ty {
                    Type::I64 => {
                        self.text
                            .push_str(&format!("  movq %rax, {offset}(%rbp)\n"));
                    }

                    Type::F32 => {
                        self.text
                            .push_str(&format!("  movss %xmm0, {offset}(%rbp)\n"));
                    }

                    Type::F64 => {
                        self.text
                            .push_str(&format!("  movsd %xmm0, {offset}(%rbp)\n"));
                    }
                }
            }

            Stmt::Print { value } => {
                let ty =
                    type_of_expr(value, self.bindings).expect("expression must have been checked");

                self.emit_expr(value, Some(ty), 0);

                self.emit_print(ty);
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr, expected: Option<Type>, depth: usize) -> Type {
        match expr {
            Expr::Integer(value) => {
                self.text.push_str(&format!("  movabsq ${value}, %rax\n"));

                Type::I64
            }

            Expr::Float {
                text,
                explicit_type,
            } => {
                let ty = match explicit_type {
                    Some(ty) => *ty,

                    None => match expected {
                        Some(Type::F32) => Type::F32,

                        _ => Type::F64,
                    },
                };

                let label = self.add_float_constant(text, ty);

                match ty {
                    Type::F32 => {
                        self.text
                            .push_str(&format!("  movss {label}(%rip), %xmm0\n"));
                    }

                    Type::F64 => {
                        self.text
                            .push_str(&format!("  movsd {label}(%rip), %xmm0\n"));
                    }

                    Type::I64 => {
                        unreachable!("integer cannot be emitted as float");
                    }
                }

                ty
            }

            Expr::Variable(name) => {
                let ty = self
                    .bindings
                    .get(name)
                    .copied()
                    .expect("variable must have been resolved by type checker");

                let offset = self.binding_offset(name);

                match ty {
                    Type::I64 => {
                        self.text
                            .push_str(&format!("  movq {offset}(%rbp), %rax\n"));
                    }

                    Type::F32 => {
                        self.text
                            .push_str(&format!("  movss {offset}(%rbp), %xmm0\n"));
                    }

                    Type::F64 => {
                        self.text
                            .push_str(&format!("  movsd {offset}(%rbp), %xmm0\n"));
                    }
                }

                ty
            }

            Expr::Unary { op, value } => {
                let ty = expected.unwrap_or_else(|| {
                    type_of_expr(expr, self.bindings).expect("expression must have been checked")
                });

                self.emit_expr(value, Some(ty), depth);

                match (op, ty) {
                    (UnaryOp::Negate, Type::I64) => {
                        self.text.push_str("  negq %rax\n");
                    }

                    (UnaryOp::Negate, Type::F32) => {
                        self.text
                            .push_str("  xorps .Lprimer_sign_f32(%rip), %xmm0\n");
                    }

                    (UnaryOp::Negate, Type::F64) => {
                        self.text
                            .push_str("  xorpd .Lprimer_sign_f64(%rip), %xmm0\n");
                    }
                }

                ty
            }

            Expr::Binary { op, left, right } => {
                let ty = expected.unwrap_or_else(|| {
                    type_of_expr(expr, self.bindings).expect("expression must have been checked")
                });

                // 左辺を計算。
                self.emit_expr(left, Some(ty), depth + 1);

                // 左辺をscratchへ退避。
                let scratch = self.scratch_offset(depth);

                match ty {
                    Type::I64 => {
                        self.text
                            .push_str(&format!("  movq %rax, {scratch}(%rbp)\n"));
                    }

                    Type::F32 => {
                        self.text
                            .push_str(&format!("  movss %xmm0, {scratch}(%rbp)\n"));
                    }

                    Type::F64 => {
                        self.text
                            .push_str(&format!("  movsd %xmm0, {scratch}(%rbp)\n"));
                    }
                }

                // 右辺を計算。
                self.emit_expr(right, Some(ty), depth + 1);

                match ty {
                    Type::I64 => {
                        // right
                        self.text.push_str("  movq %rax, %rcx\n");

                        // left
                        self.text
                            .push_str(&format!("  movq {scratch}(%rbp), %rax\n"));

                        match op {
                            BinaryOp::Add => {
                                self.text.push_str("  addq %rcx, %rax\n");
                            }

                            BinaryOp::Subtract => {
                                self.text.push_str("  subq %rcx, %rax\n");
                            }

                            BinaryOp::Multiply => {
                                self.text.push_str("  imulq %rcx, %rax\n");
                            }

                            BinaryOp::Divide => {
                                self.text.push_str("  cqto\n");

                                self.text.push_str("  idivq %rcx\n");
                            }
                        }
                    }

                    Type::F32 => {
                        // right
                        self.text.push_str("  movaps %xmm0, %xmm1\n");

                        // left
                        self.text
                            .push_str(&format!("  movss {scratch}(%rbp), %xmm0\n"));

                        self.text
                            .push_str(&format!("  {} %xmm1, %xmm0\n", f32_instruction(*op,),));
                    }

                    Type::F64 => {
                        // right
                        self.text.push_str("  movapd %xmm0, %xmm1\n");

                        // left
                        self.text
                            .push_str(&format!("  movsd {scratch}(%rbp), %xmm0\n"));

                        self.text
                            .push_str(&format!("  {} %xmm1, %xmm0\n", f64_instruction(*op,),));
                    }
                }

                ty
            }
        }
    }

    fn emit_print(&mut self, ty: Type) {
        match ty {
            Type::I64 => {
                // printf(format, value)
                self.text.push_str("  movq %rax, %rdx\n");

                self.text.push_str("  leaq .Lprimer_fmt_i64(%rip), %rcx\n");

                self.text.push_str("  callq printf\n");
            }

            Type::F32 => {
                // C varargs:
                // float -> double
                self.text.push_str("  cvtss2sd %xmm0, %xmm1\n");

                // Windows x64 varargs requires
                // floating-point arg duplicated
                // into the corresponding GP register.
                self.text.push_str("  movq %xmm1, %rdx\n");

                self.text.push_str("  leaq .Lprimer_fmt_f32(%rip), %rcx\n");

                self.text.push_str("  callq printf\n");
            }

            Type::F64 => {
                self.text.push_str("  movsd %xmm0, %xmm1\n");

                self.text.push_str("  movq %xmm1, %rdx\n");

                self.text.push_str("  leaq .Lprimer_fmt_f64(%rip), %rcx\n");

                self.text.push_str("  callq printf\n");
            }
        }
    }

    fn add_float_constant(&mut self, text: &str, ty: Type) -> String {
        let id = self.float_id;

        self.float_id += 1;

        match ty {
            Type::F32 => {
                let value = text
                    .parse::<f32>()
                    .expect("validated floating-point literal");

                let label = format!(".Lprimer_f32_{id}");

                self.data.push_str(".p2align 2\n");

                self.data.push_str(&format!("{label}:\n"));

                self.data
                    .push_str(&format!("  .long 0x{:08X}\n", value.to_bits(),));

                label
            }

            Type::F64 => {
                let value = text
                    .parse::<f64>()
                    .expect("validated floating-point literal");

                let label = format!(".Lprimer_f64_{id}");

                self.data.push_str(".p2align 3\n");

                self.data.push_str(&format!("{label}:\n"));

                self.data
                    .push_str(&format!("  .quad 0x{:016X}\n", value.to_bits(),));

                label
            }

            Type::I64 => {
                unreachable!("integer cannot be emitted as float");
            }
        }
    }

    fn binding_offset(&self, name: &str) -> isize {
        let slot = self
            .binding_slots
            .get(name)
            .copied()
            .expect("binding must have a stack slot");

        slot_offset(slot)
    }

    fn scratch_offset(&self, depth: usize) -> isize {
        slot_offset(self.scratch_base + depth)
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

fn assign_binding_slots(program: &Program) -> HashMap<String, usize> {
    let mut slots = HashMap::new();

    let mut next = 0;

    for statement in &program.statements {
        if let Stmt::Binding { name, .. } = statement {
            slots.insert(name.clone(), next);

            next += 1;
        }
    }

    slots
}

fn count_program_expr_nodes(program: &Program) -> usize {
    program
        .statements
        .iter()
        .map(|statement| match statement {
            Stmt::Binding { value, .. } | Stmt::Print { value } => count_expr_nodes(value),
        })
        .sum()
}

fn count_expr_nodes(expr: &Expr) -> usize {
    match expr {
        Expr::Integer(_) | Expr::Float { .. } | Expr::Variable(_) => 1,

        Expr::Unary { value, .. } => 1 + count_expr_nodes(value),

        Expr::Binary { left, right, .. } => 1 + count_expr_nodes(left) + count_expr_nodes(right),
    }
}

fn slot_offset(slot: usize) -> isize {
    -8 * (slot as isize + 1)
}

fn align16(value: usize) -> usize {
    (value + 15) & !15
}

fn f32_instruction(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "addss",

        BinaryOp::Subtract => "subss",

        BinaryOp::Multiply => "mulss",

        BinaryOp::Divide => "divss",
    }
}

fn f64_instruction(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "addsd",

        BinaryOp::Subtract => "subsd",

        BinaryOp::Multiply => "mulsd",

        BinaryOp::Divide => "divsd",
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::lex, parser::parse, semantic::check};

    use super::emit_x86_64_win_asm;

    #[test]
    fn emits_i64_arithmetic() {
        let program = parse(
            lex("x: i64 = 1 + 2;
                     print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let asm = emit_x86_64_win_asm(&program, &bindings);

        assert!(asm.contains("addq %rcx, %rax"));

        assert!(asm.contains("callq printf"));
    }

    #[test]
    fn emits_f32_arithmetic() {
        let program = parse(
            lex("x: f32 = 0.1 + 0.2;
                     print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let asm = emit_x86_64_win_asm(&program, &bindings);

        assert!(asm.contains("addss %xmm1, %xmm0"));

        assert!(asm.contains("cvtss2sd %xmm0, %xmm1"));
    }

    #[test]
    fn emits_f64_arithmetic() {
        let program = parse(
            lex("x: f64 = 0.1 + 0.2;
                     print(x);")
            .unwrap(),
        )
        .unwrap();

        let bindings = check(&program).unwrap();

        let asm = emit_x86_64_win_asm(&program, &bindings);

        assert!(asm.contains("addsd %xmm1, %xmm0"));

        assert!(asm.contains("movq %xmm1, %rdx"));
    }
}
