.section .rdata,"dr"
.Lprimer_fmt_i64:
  .asciz "%lld\n"
.Lprimer_fmt_f32:
  .asciz "%.9g\n"
.Lprimer_fmt_f64:
  .asciz "%.17g\n"
.p2align 4
.Lprimer_sign_f32:
  .long 0x80000000
  .long 0
  .long 0
  .long 0
.p2align 4
.Lprimer_sign_f64:
  .quad 0x8000000000000000
  .quad 0

.text
.p2align 4
primer_fn_add_0:
  pushq %rbp
  movq %rsp, %rbp
  subq $80, %rsp
  movq %rcx, -8(%rbp)
  movq %rdx, -16(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -24(%rbp)
  movq -16(%rbp), %rax
  movq %rax, %rcx
  movq -24(%rbp), %rax
  addq %rcx, %rax
  addq $80, %rsp
  popq %rbp
  retq

.p2align 4
primer_fn_show_1:
  pushq %rbp
  movq %rsp, %rbp
  subq $48, %rsp
  movq %rcx, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  addq $48, %rsp
  popq %rbp
  retq

.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $80, %rsp
  movabsq $20, %rax
  movq %rax, -16(%rbp)
  movabsq $22, %rax
  movq %rax, -24(%rbp)
  movq -16(%rbp), %rcx
  movq -24(%rbp), %rdx
  callq primer_fn_add_0
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movq -16(%rbp), %rcx
  callq primer_fn_show_1
  xorl %eax, %eax
  addq $80, %rsp
  popq %rbp
  retq
