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
  subq $160, %rsp
  movabsq $8, %rax
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $1, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  addq %rcx, %rax
  jno .Lprimer_main_integer_ok_0
  ud2
.Lprimer_main_integer_ok_0:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $1, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  subq %rcx, %rax
  jno .Lprimer_main_integer_ok_1
  ud2
.Lprimer_main_integer_ok_1:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  imulq %rcx, %rax
  jno .Lprimer_main_integer_ok_2
  ud2
.Lprimer_main_integer_ok_2:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  testq %rcx, %rcx
  je .Lprimer_main_division_trap_3
  cmpq $-1, %rcx
  jne .Lprimer_main_division_ok_3
  movabsq $-9223372036854775808, %rdx
  cmpq %rdx, %rax
  jne .Lprimer_main_division_ok_3
.Lprimer_main_division_trap_3:
  ud2
.Lprimer_main_division_ok_3:
  cqto
  idivq %rcx
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  negq %rax
  jno .Lprimer_main_integer_ok_4
  ud2
.Lprimer_main_integer_ok_4:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $160, %rsp
  popq %rbp
  retq
