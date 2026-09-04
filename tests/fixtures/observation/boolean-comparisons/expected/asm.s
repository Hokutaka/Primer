.section .rdata,"dr"
.Lprimer_fmt_i64:
  .asciz "%lld\n"
.Lprimer_fmt_f32:
  .asciz "%.9g\n"
.Lprimer_fmt_f64:
  .asciz "%.17g\n"
.Lprimer_bool_false:
  .asciz "false"
.Lprimer_bool_true:
  .asciz "true"
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
  .long 0x3DCCCCCD
.p2align 2
.Lprimer_f32_1:
  .long 0x3E4CCCCD

.text
.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $224, %rsp
  movabsq $1, %rax
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  xorq $1, %rax
  movq %rax, -16(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -48(%rbp)
  movabsq $1, %rax
  movq %rax, %rcx
  movq -48(%rbp), %rax
  cmpq %rcx, %rax
  sete %al
  movzbq %al, %rax
  movq %rax, -24(%rbp)
  movabsq $1, %rax
  movq %rax, -56(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -56(%rbp), %rax
  addq %rcx, %rax
  jno .Lprimer_main_integer_ok_0
  ud2
.Lprimer_main_integer_ok_0:
  movq %rax, -48(%rbp)
  movabsq $4, %rax
  movq %rax, %rcx
  movq -48(%rbp), %rax
  cmpq %rcx, %rax
  setl %al
  movzbq %al, %rax
  movq %rax, -32(%rbp)
  movss .Lprimer_f32_0(%rip), %xmm0
  movss %xmm0, -48(%rbp)
  movss .Lprimer_f32_1(%rip), %xmm0
  movaps %xmm0, %xmm1
  movss -48(%rbp), %xmm0
  ucomiss %xmm1, %xmm0
  setne %al
  setp %cl
  orb %cl, %al
  movzbq %al, %rax
  movq %rax, -40(%rbp)
  movq -8(%rbp), %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -16(%rbp), %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -24(%rbp), %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -32(%rbp), %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -40(%rbp), %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  xorl %eax, %eax
  addq $224, %rsp
  popq %rbp
  retq
