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
  subq $192, %rsp
  movabsq $0, %rax
  movq %rax, -8(%rbp)
  movabsq $0, %rax
  movq %rax, -16(%rbp)
.Lprimer_block_0: # while_condition
  movq -8(%rbp), %rax
  movq %rax, -32(%rbp)
  movabsq $4, %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  cmpq %rcx, %rax
  setl %al
  movzbq %al, %rax
  testq %rax, %rax
  je .Lprimer_block_1
  movq -16(%rbp), %rax
  movq %rax, -32(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  addq %rcx, %rax
  jno .Lprimer_main_integer_ok_2
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_0(%rip), %rdx
  movl $61, %r8d
  callq _write
  ud2
.Lprimer_main_integer_ok_2:
  movq %rax, -16(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -32(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  cmpq %rcx, %rax
  sete %al
  movzbq %al, %rax
  testq %rax, %rax
  je .Lprimer_block_4
  movabsq $1, %rax
  movq %rax, -24(%rbp)
  movq -24(%rbp), %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  jmp .Lprimer_block_4
.Lprimer_block_4: # if_end
  movq -8(%rbp), %rax
  movq %rax, -32(%rbp)
  movabsq $1, %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  addq %rcx, %rax
  jno .Lprimer_main_integer_ok_5
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_1(%rip), %rdx
  movl $64, %r8d
  callq _write
  ud2
.Lprimer_main_integer_ok_5:
  movq %rax, -8(%rbp)
  jmp .Lprimer_block_0
.Lprimer_block_1: # while_end
  movq -16(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $192, %rsp
  popq %rbp
  retq

.section .rdata,"dr"
.Lprimer_failure_0:
  .asciz "primer: runtime-v1 code=integer-overflow node=9 bytes=67..78\n"
.Lprimer_failure_1:
  .asciz "primer: runtime-v1 code=integer-overflow node=21 bytes=172..181\n"
