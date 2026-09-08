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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_0(%rip), %rdx
  movl $61, %r8d
  callq _write
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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_1(%rip), %rdx
  movl $61, %r8d
  callq _write
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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_2(%rip), %rdx
  movl $62, %r8d
  callq _write
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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_3(%rip), %rdx
  movl $63, %r8d
  callq _write
  ud2
.Lprimer_main_division_trap_3:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_4(%rip), %rdx
  movl $62, %r8d
  callq _write
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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_5(%rip), %rdx
  movl $63, %r8d
  callq _write
  ud2
.Lprimer_main_integer_ok_4:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $160, %rsp
  popq %rbp
  retq

.section .rdata,"dr"
.Lprimer_failure_0:
  .asciz "primer: runtime-v1 code=integer-overflow node=3 bytes=22..31\n"
.Lprimer_failure_1:
  .asciz "primer: runtime-v1 code=integer-overflow node=7 bytes=40..49\n"
.Lprimer_failure_2:
  .asciz "primer: runtime-v1 code=integer-overflow node=11 bytes=58..67\n"
.Lprimer_failure_3:
  .asciz "primer: runtime-v1 code=division-overflow node=15 bytes=76..85\n"
.Lprimer_failure_4:
  .asciz "primer: runtime-v1 code=division-by-zero node=15 bytes=76..85\n"
.Lprimer_failure_5:
  .asciz "primer: runtime-v1 code=integer-overflow node=19 bytes=94..100\n"
