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
.Lprimer_fmt_u64:
  .asciz "%llu\n"

.text
.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $144, %rsp
  movabsq $-1, %rax
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_u64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $0, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  cmpq %rcx, %rax
  seta %al
  movzbq %al, %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  testq %rcx, %rcx
  je .Lprimer_main_u64_bad_0
  xorq %rdx, %rdx
  divq %rcx
  jmp .Lprimer_main_u64_done_0
.Lprimer_main_u64_bad_0:
  ud2
.Lprimer_main_u64_done_0:
  movq %rax, %rdx
  leaq .Lprimer_fmt_u64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $63, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  cmpq $64, %rcx
  jae .Lprimer_main_u64_bad_1
  shrq %cl, %rax
  jmp .Lprimer_main_u64_done_1
.Lprimer_main_u64_bad_1:
  ud2
.Lprimer_main_u64_done_1:
  movq %rax, %rdx
  leaq .Lprimer_fmt_u64(%rip), %rcx
  callq printf
  movabsq $42, %rax
  testq %rax, %rax
  js .Lprimer_main_u64_convert_2_bad
  jmp .Lprimer_main_u64_convert_2_done
.Lprimer_main_u64_convert_2_bad:
  ud2
.Lprimer_main_u64_convert_2_done:
  movq %rax, %rdx
  leaq .Lprimer_fmt_u64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $144, %rsp
  popq %rbp
  retq
