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
.p2align 3
.Lprimer_string_0:
  .quad 11
  .byte 230
  .byte 151
  .byte 165
  .byte 230
  .byte 156
  .byte 172
  .byte 232
  .byte 170
  .byte 158
  .byte 10
  .byte 0
.p2align 3
.Lprimer_string_1:
  .quad 7
  .byte 99
  .byte 104
  .byte 97
  .byte 110
  .byte 103
  .byte 101
  .byte 100
.p2align 3
.Lprimer_string_2:
  .quad 11
  .byte 230
  .byte 151
  .byte 165
  .byte 230
  .byte 156
  .byte 172
  .byte 232
  .byte 170
  .byte 158
  .byte 10
  .byte 0

.text

.p2align 4
primer_string_equal:
  movq (%rcx), %r8
  cmpq (%rdx), %r8
  jne .Lstring_different
  xorq %r9, %r9
.Lstring_compare:
  cmpq %r8, %r9
  je .Lstring_equal
  movzbl 8(%rcx,%r9), %eax
  cmpb 8(%rdx,%r9), %al
  jne .Lstring_different
  incq %r9
  jmp .Lstring_compare
.Lstring_equal:
  movl $1, %eax
  retq
.Lstring_different:
  xorl %eax, %eax
  retq

.p2align 4
primer_print_string:
  subq $56, %rsp
  movq %rcx, 32(%rsp)
  movq $0, 40(%rsp)
.Lstring_write:
  movq 32(%rsp), %rax
  movq 40(%rsp), %rdx
  cmpq (%rax), %rdx
  je .Lstring_newline
  movzbl 8(%rax,%rdx), %ecx
  callq putchar
  incq 40(%rsp)
  jmp .Lstring_write
.Lstring_newline:
  movl $10, %ecx
  callq putchar
  addq $56, %rsp
  retq

.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $112, %rsp
  movl $1, %ecx
  movl $32768, %edx
  callq _setmode
  cmpl $-1, %eax
  jne .Lstdout_ready
  movl $1, %eax
  addq $112, %rsp
  popq %rbp
  retq
.Lstdout_ready:
  leaq .Lprimer_string_0(%rip), %rax
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  leaq .Lprimer_string_1(%rip), %rax
  movq %rax, -8(%rbp)
  movq -16(%rbp), %rax
  movq %rax, -24(%rbp)
  leaq .Lprimer_string_2(%rip), %rax
  movq %rax, %rdx
  movq -24(%rbp), %rcx
  callq primer_string_equal
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -8(%rbp), %rax
  movq %rax, %rcx
  callq primer_print_string
  xorl %eax, %eax
  addq $112, %rsp
  popq %rbp
  retq
