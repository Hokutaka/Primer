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

.text
.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $128, %rsp
  movabsq $1, %rax
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -24(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -24(%rbp), %rax
  cmpq %rcx, %rax
  setl %al
  movzbq %al, %rax
  testq %rax, %rax
  je .Lprimer_block_0
  movabsq $42, %rax
  movq %rax, -8(%rbp)
  movabsq $1, %rax
  movq %rax, -16(%rbp)
  movq -16(%rbp), %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  jmp .Lprimer_block_1
.Lprimer_block_0: # if_else
  movabsq $1, %rax
  negq %rax
  jno .Lprimer_main_integer_ok_2
  ud2
.Lprimer_main_integer_ok_2:
  movq %rax, -8(%rbp)
.Lprimer_block_1: # if_end
  movq -8(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $128, %rsp
  popq %rbp
  retq
