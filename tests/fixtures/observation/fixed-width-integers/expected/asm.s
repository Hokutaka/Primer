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
  jno .Lprimer_fn_0_integer_ok_0
  ud2
.Lprimer_fn_0_integer_ok_0:
  # semantic i32, storage i64
  movabsq $-2147483648, %r11
  cmpq %r11, %rax
  jl .Lprimer_fn_0_range_bad_1
  movabsq $2147483647, %r11
  cmpq %r11, %rax
  jle .Lprimer_fn_0_range_ok_1
.Lprimer_fn_0_range_bad_1:
  ud2
.Lprimer_fn_0_range_ok_1:
  addq $80, %rsp
  popq %rbp
  retq

.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $160, %rsp
  movabsq $3, %rax
  negq %rax
  jno .Lprimer_main_integer_ok_0
  ud2
.Lprimer_main_integer_ok_0:
  # semantic i32, storage i64
  movabsq $-2147483648, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_1
  movabsq $2147483647, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_1
.Lprimer_main_range_bad_1:
  ud2
.Lprimer_main_range_ok_1:
  movq %rax, -24(%rbp)
  movabsq $5, %rax
  movq %rax, -32(%rbp)
  movq -24(%rbp), %rcx
  movq -32(%rbp), %rdx
  callq primer_fn_add_0
  movq %rax, -8(%rbp)
  movabsq $4294967295, %rax
  movq %rax, -16(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -16(%rbp), %rax
  movq %rax, -24(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -24(%rbp), %rax
  testq %rcx, %rcx
  je .Lprimer_main_division_trap_2
  cmpq $-1, %rcx
  jne .Lprimer_main_division_ok_2
  movabsq $-9223372036854775808, %rdx
  cmpq %rdx, %rax
  jne .Lprimer_main_division_ok_2
.Lprimer_main_division_trap_2:
  ud2
.Lprimer_main_division_ok_2:
  cqto
  idivq %rcx
  # semantic u32, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_3
  movabsq $4294967295, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_3
.Lprimer_main_range_bad_3:
  ud2
.Lprimer_main_range_ok_3:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -16(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -16(%rbp), %rax
  movq %rax, -24(%rbp)
  movabsq $2147483648, %rax
  movq %rax, %rcx
  movq -24(%rbp), %rax
  cmpq %rcx, %rax
  setg %al
  movzbq %al, %rax
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  movq -8(%rbp), %rax
  # semantic u32, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_4
  movabsq $4294967295, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_4
.Lprimer_main_range_bad_4:
  ud2
.Lprimer_main_range_ok_4:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $160, %rsp
  popq %rbp
  retq
