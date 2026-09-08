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
primer_fn_average_0:
  pushq %rbp
  movq %rsp, %rbp
  subq $96, %rsp
  movq %rcx, -8(%rbp)
  movq %rdx, -16(%rbp)
  movq -8(%rbp), %rax
  # semantic u16, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_fn_0_range_bad_0
  movabsq $65535, %r11
  cmpq %r11, %rax
  jle .Lprimer_fn_0_range_ok_0
.Lprimer_fn_0_range_bad_0:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_0(%rip), %rdx
  movl $76, %r8d
  callq _write
  ud2
.Lprimer_fn_0_range_ok_0:
  movq %rax, -32(%rbp)
  movq -16(%rbp), %rax
  # semantic u16, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_fn_0_range_bad_1
  movabsq $65535, %r11
  cmpq %r11, %rax
  jle .Lprimer_fn_0_range_ok_1
.Lprimer_fn_0_range_bad_1:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_1(%rip), %rdx
  movl $76, %r8d
  callq _write
  ud2
.Lprimer_fn_0_range_ok_1:
  movq %rax, %rcx
  movq -32(%rbp), %rax
  addq %rcx, %rax
  jno .Lprimer_fn_0_integer_ok_2
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_2(%rip), %rdx
  movl $61, %r8d
  callq _write
  ud2
.Lprimer_fn_0_integer_ok_2:
  # semantic u16, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_fn_0_range_bad_3
  movabsq $65535, %r11
  cmpq %r11, %rax
  jle .Lprimer_fn_0_range_ok_3
.Lprimer_fn_0_range_bad_3:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_3(%rip), %rdx
  movl $61, %r8d
  callq _write
  ud2
.Lprimer_fn_0_range_ok_3:
  movq %rax, -24(%rbp)
  movabsq $2, %rax
  movq %rax, %rcx
  movq -24(%rbp), %rax
  testq %rcx, %rcx
  je .Lprimer_fn_0_division_trap_4
  cmpq $-1, %rcx
  jne .Lprimer_fn_0_division_ok_4
  movabsq $-9223372036854775808, %rdx
  cmpq %rdx, %rax
  jne .Lprimer_fn_0_division_ok_4
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_4(%rip), %rdx
  movl $62, %r8d
  callq _write
  ud2
.Lprimer_fn_0_division_trap_4:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_5(%rip), %rdx
  movl $61, %r8d
  callq _write
  ud2
.Lprimer_fn_0_division_ok_4:
  cqto
  idivq %rcx
  # semantic u16, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_fn_0_range_bad_5
  movabsq $65535, %r11
  cmpq %r11, %rax
  jle .Lprimer_fn_0_range_ok_5
.Lprimer_fn_0_range_bad_5:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_6(%rip), %rdx
  movl $62, %r8d
  callq _write
  ud2
.Lprimer_fn_0_range_ok_5:
  # semantic u8, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_fn_0_range_bad_6
  movabsq $255, %r11
  cmpq %r11, %rax
  jle .Lprimer_fn_0_range_ok_6
.Lprimer_fn_0_range_bad_6:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_7(%rip), %rdx
  movl $76, %r8d
  callq _write
  ud2
.Lprimer_fn_0_range_ok_6:
  addq $96, %rsp
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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_8(%rip), %rdx
  movl $64, %r8d
  callq _write
  ud2
.Lprimer_main_integer_ok_0:
  # semantic i8, storage i64
  movabsq $-128, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_1
  movabsq $127, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_1
.Lprimer_main_range_bad_1:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_9(%rip), %rdx
  movl $64, %r8d
  callq _write
  ud2
.Lprimer_main_range_ok_1:
  movq %rax, -8(%rbp)
  movabsq $32000, %rax
  negq %rax
  jno .Lprimer_main_integer_ok_2
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_10(%rip), %rdx
  movl $64, %r8d
  callq _write
  ud2
.Lprimer_main_integer_ok_2:
  # semantic i16, storage i64
  movabsq $-32768, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_3
  movabsq $32767, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_3
.Lprimer_main_range_bad_3:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_11(%rip), %rdx
  movl $64, %r8d
  callq _write
  ud2
.Lprimer_main_range_ok_3:
  movq %rax, -16(%rbp)
  movq -16(%rbp), %rax
  movq %rax, -24(%rbp)
  movq -8(%rbp), %rax
  # semantic i16, storage i64
  movabsq $-32768, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_4
  movabsq $32767, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_4
.Lprimer_main_range_bad_4:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_12(%rip), %rdx
  movl $79, %r8d
  callq _write
  ud2
.Lprimer_main_range_ok_4:
  movq %rax, %rcx
  movq -24(%rbp), %rax
  addq %rcx, %rax
  jno .Lprimer_main_integer_ok_5
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_13(%rip), %rdx
  movl $64, %r8d
  callq _write
  ud2
.Lprimer_main_integer_ok_5:
  # semantic i16, storage i64
  movabsq $-32768, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_6
  movabsq $32767, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_6
.Lprimer_main_range_bad_6:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_14(%rip), %rdx
  movl $64, %r8d
  callq _write
  ud2
.Lprimer_main_range_ok_6:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $240, %rax
  movq %rax, -24(%rbp)
  movabsq $80, %rax
  movq %rax, -32(%rbp)
  movq -24(%rbp), %rcx
  movq -32(%rbp), %rdx
  callq primer_fn_average_0
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $127, %rax
  movq %rax, -24(%rbp)
  movabsq $-128, %rax
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
  movabsq $255, %rax
  # semantic u16, storage i64
  movabsq $0, %r11
  cmpq %r11, %rax
  jl .Lprimer_main_range_bad_7
  movabsq $65535, %r11
  cmpq %r11, %rax
  jle .Lprimer_main_range_ok_7
.Lprimer_main_range_bad_7:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_15(%rip), %rdx
  movl $79, %r8d
  callq _write
  ud2
.Lprimer_main_range_ok_7:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $160, %rsp
  popq %rbp
  retq

.section .rdata,"dr"
.Lprimer_failure_0:
  .asciz "primer: runtime-v1 code=integer-conversion-out-of-range node=4 bytes=55..64\n"
.Lprimer_failure_1:
  .asciz "primer: runtime-v1 code=integer-conversion-out-of-range node=6 bytes=67..77\n"
.Lprimer_failure_2:
  .asciz "primer: runtime-v1 code=integer-overflow node=3 bytes=54..78\n"
.Lprimer_failure_3:
  .asciz "primer: runtime-v1 code=integer-overflow node=3 bytes=54..78\n"
.Lprimer_failure_4:
  .asciz "primer: runtime-v1 code=division-overflow node=2 bytes=54..82\n"
.Lprimer_failure_5:
  .asciz "primer: runtime-v1 code=division-by-zero node=2 bytes=54..82\n"
.Lprimer_failure_6:
  .asciz "primer: runtime-v1 code=division-overflow node=2 bytes=54..82\n"
.Lprimer_failure_7:
  .asciz "primer: runtime-v1 code=integer-conversion-out-of-range node=1 bytes=51..83\n"
.Lprimer_failure_8:
  .asciz "primer: runtime-v1 code=integer-overflow node=10 bytes=101..103\n"
.Lprimer_failure_9:
  .asciz "primer: runtime-v1 code=integer-overflow node=10 bytes=101..103\n"
.Lprimer_failure_10:
  .asciz "primer: runtime-v1 code=integer-overflow node=13 bytes=120..126\n"
.Lprimer_failure_11:
  .asciz "primer: runtime-v1 code=integer-overflow node=13 bytes=120..126\n"
.Lprimer_failure_12:
  .asciz "primer: runtime-v1 code=integer-conversion-out-of-range node=18 bytes=144..155\n"
.Lprimer_failure_13:
  .asciz "primer: runtime-v1 code=integer-overflow node=16 bytes=134..155\n"
.Lprimer_failure_14:
  .asciz "primer: runtime-v1 code=integer-overflow node=16 bytes=134..155\n"
.Lprimer_failure_15:
  .asciz "primer: runtime-v1 code=integer-conversion-out-of-range node=29 bytes=212..231\n"
