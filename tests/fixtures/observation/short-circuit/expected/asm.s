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
.p2align 4
primer_fn_report_0:
  pushq %rbp
  movq %rsp, %rbp
  subq $64, %rsp
  movq %rcx, -8(%rbp)
  movq -8(%rbp), %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -8(%rbp), %rax
  addq $64, %rsp
  popq %rbp
  retq

.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $320, %rsp
  movabsq $4, %rax
  movq %rax, -272(%rbp)
  movabsq $9, %rax
  movq %rax, -280(%rbp)
  movq -272(%rbp), %rax
  movq %rax, -8(%rbp)
  movq -280(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $2, %rax
  movq %rax, -24(%rbp)
  movq -24(%rbp), %rax
  movq %rax, -32(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  cmpq %rcx, %rax
  setl %al
  movzbq %al, %rax
  testq %rax, %rax
  je .Lprimer_block_0
  movq -24(%rbp), %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_2
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_2
  negq %rax
  movq -8(%rbp,%rax,8), %rax
  jmp .Lprimer_main_array_done_2
.Lprimer_main_array_oob_2:
  ud2
.Lprimer_main_array_done_2:
  movq %rax, -32(%rbp)
  movabsq $0, %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  cmpq %rcx, %rax
  setg %al
  movzbq %al, %rax
  jmp .Lprimer_block_1
.Lprimer_block_0: # logical_false
.Lprimer_block_1: # logical_end
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -24(%rbp), %rax
  movq %rax, -32(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  cmpq %rcx, %rax
  sete %al
  movzbq %al, %rax
  testq %rax, %rax
  je .Lprimer_block_3
  jmp .Lprimer_block_4
.Lprimer_block_3: # logical_false
  movabsq $0, %rax
  movq %rax, -32(%rbp)
  movq -32(%rbp), %rcx
  callq primer_fn_report_0
.Lprimer_block_4: # logical_end
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movabsq $0, %rax
  testq %rax, %rax
  je .Lprimer_block_5
  jmp .Lprimer_block_6
.Lprimer_block_5: # logical_false
  movabsq $1, %rax
  movq %rax, -32(%rbp)
  movq -32(%rbp), %rcx
  callq primer_fn_report_0
  testq %rax, %rax
  je .Lprimer_block_7
  movq -24(%rbp), %rax
  movq %rax, -32(%rbp)
  movabsq $0, %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  cmpq %rcx, %rax
  setg %al
  movzbq %al, %rax
  testq %rax, %rax
  je .Lprimer_block_9
  jmp .Lprimer_block_10
.Lprimer_block_9: # logical_false
  movabsq $0, %rax
  movq %rax, -32(%rbp)
  movq -32(%rbp), %rcx
  callq primer_fn_report_0
.Lprimer_block_10: # logical_end
  jmp .Lprimer_block_8
.Lprimer_block_7: # logical_false
.Lprimer_block_8: # logical_end
.Lprimer_block_6: # logical_end
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  xorl %eax, %eax
  addq $320, %rsp
  popq %rbp
  retq
