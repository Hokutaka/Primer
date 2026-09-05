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
primer_fn_mark_0:
  pushq %rbp
  movq %rsp, %rbp
  subq $64, %rsp
  movq %rcx, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  addq $64, %rsp
  popq %rbp
  retq

.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $352, %rsp
  movabsq $1, %rax
  movq %rax, -16(%rbp)
  movabsq $7, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  testq %rcx, %rcx
  js .Lprimer_main_integer_bad_0
  cmpq $8, %rcx
  jge .Lprimer_main_integer_bad_0
  movabsq $0, %r11
  sarq %cl, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_integer_bad_0
  movabsq $255, %r11
  shrq %cl, %r11
  cmpq %r11, %rax
  jg .Lprimer_main_integer_bad_0
  shlq %cl, %rax
  jmp .Lprimer_main_integer_done_0
.Lprimer_main_integer_bad_0:
  ud2
.Lprimer_main_integer_done_0:
  # semantic u8, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_1
  movabsq $255, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_1
.Lprimer_main_range_bad_1:
  ud2
.Lprimer_main_range_ok_1:
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $7, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  testq %rcx, %rcx
  js .Lprimer_main_integer_bad_2
  cmpq $8, %rcx
  jge .Lprimer_main_integer_bad_2
  sarq %cl, %rax
  jmp .Lprimer_main_integer_done_2
.Lprimer_main_integer_bad_2:
  ud2
.Lprimer_main_integer_done_2:
  # semantic u8, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_3
  movabsq $255, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_3
.Lprimer_main_range_bad_3:
  ud2
.Lprimer_main_range_ok_3:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $0, %rax
  movabsq $255, %r11
  xorq %r11, %rax
  # semantic u8, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_4
  movabsq $255, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_4
.Lprimer_main_range_bad_4:
  ud2
.Lprimer_main_range_ok_4:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $1, %rax
  movq %rax, -24(%rbp)
  movq -24(%rbp), %rcx
  callq primer_fn_mark_0
  movq %rax, -16(%rbp)
  movabsq $2, %rax
  movq %rax, -32(%rbp)
  movq -32(%rbp), %rcx
  callq primer_fn_mark_0
  movq %rax, -24(%rbp)
  movabsq $3, %rax
  movq %rax, -32(%rbp)
  movq -32(%rbp), %rcx
  callq primer_fn_mark_0
  movq %rax, %rcx
  movq -24(%rbp), %rax
  xorq %rcx, %rax
  # semantic u8, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_6
  movabsq $255, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_6
.Lprimer_main_range_bad_6:
  ud2
.Lprimer_main_range_ok_6:
  movq %rax, %rcx
  movq -16(%rbp), %rax
  orq %rcx, %rax
  # semantic u8, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_8
  movabsq $255, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_8
.Lprimer_main_range_bad_8:
  ud2
.Lprimer_main_range_ok_8:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $127, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  andq %rcx, %rax
  # semantic u8, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_10
  movabsq $255, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_10
.Lprimer_main_range_bad_10:
  ud2
.Lprimer_main_range_ok_10:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $7, %rax
  negq %rax
  jno .Lprimer_main_integer_ok_11
  ud2
.Lprimer_main_integer_ok_11:
  movq %rax, -16(%rbp)
  movabsq $3, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  testq %rcx, %rcx
  je .Lprimer_main_integer_bad_12
  cmpq $-1, %rcx
  jne .Lprimer_main_integer_rem_12
  xorq %rax, %rax
  jmp .Lprimer_main_integer_done_12
.Lprimer_main_integer_rem_12:
  cqto
  idivq %rcx
  movq %rdx, %rax
  jmp .Lprimer_main_integer_done_12
.Lprimer_main_integer_bad_12:
  ud2
.Lprimer_main_integer_done_12:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $-9223372036854775808, %rax
  movq %rax, -16(%rbp)
  movabsq $1, %rax
  negq %rax
  jno .Lprimer_main_integer_ok_13
  ud2
.Lprimer_main_integer_ok_13:
  movq %rax, %rcx
  movq -16(%rbp), %rax
  testq %rcx, %rcx
  je .Lprimer_main_integer_bad_14
  cmpq $-1, %rcx
  jne .Lprimer_main_integer_rem_14
  xorq %rax, %rax
  jmp .Lprimer_main_integer_done_14
.Lprimer_main_integer_rem_14:
  cqto
  idivq %rcx
  movq %rdx, %rax
  jmp .Lprimer_main_integer_done_14
.Lprimer_main_integer_bad_14:
  ud2
.Lprimer_main_integer_done_14:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $3, %rax
  negq %rax
  jno .Lprimer_main_integer_ok_15
  ud2
.Lprimer_main_integer_ok_15:
  # semantic i8, storage i64
  movabsq $-128, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_16
  movabsq $127, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_16
.Lprimer_main_range_bad_16:
  ud2
.Lprimer_main_range_ok_16:
  movq %rax, -16(%rbp)
  movabsq $1, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  testq %rcx, %rcx
  js .Lprimer_main_integer_bad_17
  cmpq $8, %rcx
  jge .Lprimer_main_integer_bad_17
  sarq %cl, %rax
  jmp .Lprimer_main_integer_done_17
.Lprimer_main_integer_bad_17:
  ud2
.Lprimer_main_integer_done_17:
  # semantic i8, storage i64
  movabsq $-128, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_18
  movabsq $127, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_18
.Lprimer_main_range_bad_18:
  ud2
.Lprimer_main_range_ok_18:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $0, %rax
  testq %rax, %rax
  je .Lprimer_block_19
  movq -8(%rbp), %rax
  movq %rax, -24(%rbp)
  movabsq $1, %rax
  movq %rax, %rcx
  movq -24(%rbp), %rax
  testq %rcx, %rcx
  js .Lprimer_main_integer_bad_21
  cmpq $8, %rcx
  jge .Lprimer_main_integer_bad_21
  movabsq $0, %r11
  sarq %cl, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_integer_bad_21
  movabsq $255, %r11
  shrq %cl, %r11
  cmpq %r11, %rax
  jg .Lprimer_main_integer_bad_21
  shlq %cl, %rax
  jmp .Lprimer_main_integer_done_21
.Lprimer_main_integer_bad_21:
  ud2
.Lprimer_main_integer_done_21:
  # semantic u8, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_22
  movabsq $255, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_22
.Lprimer_main_range_bad_22:
  ud2
.Lprimer_main_range_ok_22:
  movq %rax, -16(%rbp)
  movabsq $0, %rax
  movq %rax, %rcx
  movq -16(%rbp), %rax
  cmpq %rcx, %rax
  sete %al
  movzbq %al, %rax
  jmp .Lprimer_block_20
.Lprimer_block_19: # logical_false
.Lprimer_block_20: # logical_end
  testq %rax, %rax
  leaq .Lprimer_bool_false(%rip), %rcx
  leaq .Lprimer_bool_true(%rip), %rdx
  cmovne %rdx, %rcx
  callq puts
  xorl %eax, %eax
  addq $352, %rsp
  popq %rbp
  retq
