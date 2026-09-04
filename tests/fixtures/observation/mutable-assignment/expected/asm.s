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
.p2align 2
.Lprimer_f32_0:
  .long 0x3E800000
.p2align 2
.Lprimer_f32_1:
  .long 0x40000000

.text
.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $128, %rsp
  movabsq $40, %rax
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -24(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -24(%rbp), %rax
  addq %rcx, %rax
  jno .Lprimer_main_integer_ok_0
  ud2
.Lprimer_main_integer_ok_0:
  movq %rax, -8(%rbp)
  movss .Lprimer_f32_0(%rip), %xmm0
  movss %xmm0, -16(%rbp)
  movss -16(%rbp), %xmm0
  movss %xmm0, -24(%rbp)
  movss .Lprimer_f32_1(%rip), %xmm0
  movaps %xmm0, %xmm1
  movss -24(%rbp), %xmm0
  mulss %xmm1, %xmm0
  movss %xmm0, -16(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movss -16(%rbp), %xmm0
  cvtss2sd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f32(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $128, %rsp
  popq %rbp
  retq
