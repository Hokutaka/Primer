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
.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $256, %rsp
  movabsq $2, %rax
  movq %rax, -176(%rbp)
  movabsq $4, %rax
  movq %rax, -184(%rbp)
  movabsq $6, %rax
  movq %rax, -192(%rbp)
  movq -176(%rbp), %rax
  movq %rax, -8(%rbp)
  movq -184(%rbp), %rax
  movq %rax, -16(%rbp)
  movq -192(%rbp), %rax
  movq %rax, -24(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -32(%rbp)
  movq -16(%rbp), %rax
  movq %rax, -40(%rbp)
  movq -24(%rbp), %rax
  movq %rax, -48(%rbp)
  movabsq $1, %rax
  movq %rax, -200(%rbp)
  movabsq $3, %rax
  movq %rax, -208(%rbp)
  movabsq $5, %rax
  movq %rax, -216(%rbp)
  movq -200(%rbp), %rax
  movq %rax, -8(%rbp)
  movq -208(%rbp), %rax
  movq %rax, -16(%rbp)
  movq -216(%rbp), %rax
  movq %rax, -24(%rbp)
  movabsq $2, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_0
  cmpq $3, %rax
  jge .Lprimer_main_array_oob_0
  negq %rax
  movq -32(%rbp,%rax,8), %rax
  jmp .Lprimer_main_array_done_0
.Lprimer_main_array_oob_0:
  ud2
.Lprimer_main_array_done_0:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $1, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_1
  cmpq $3, %rax
  jge .Lprimer_main_array_oob_1
  negq %rax
  movq -8(%rbp,%rax,8), %rax
  jmp .Lprimer_main_array_done_1
.Lprimer_main_array_oob_1:
  ud2
.Lprimer_main_array_done_1:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $256, %rsp
  popq %rbp
  retq
