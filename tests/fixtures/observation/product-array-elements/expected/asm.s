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
  subq $448, %rsp
  movabsq $1, %rax
  movq %rax, -288(%rbp)
  movabsq $2, %rax
  movq %rax, -296(%rbp)
  movq -288(%rbp), %rax
  movq %rax, -256(%rbp)
  movq -296(%rbp), %rax
  movq %rax, -264(%rbp)
  movabsq $3, %rax
  movq %rax, -304(%rbp)
  movabsq $4, %rax
  movq %rax, -312(%rbp)
  movq -304(%rbp), %rax
  movq %rax, -272(%rbp)
  movq -312(%rbp), %rax
  movq %rax, -280(%rbp)
  movq -256(%rbp), %rax
  movq %rax, -8(%rbp)
  movq -264(%rbp), %rax
  movq %rax, -16(%rbp)
  movq -272(%rbp), %rax
  movq %rax, -24(%rbp)
  movq -280(%rbp), %rax
  movq %rax, -32(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -40(%rbp)
  movq -16(%rbp), %rax
  movq %rax, -48(%rbp)
  movq -24(%rbp), %rax
  movq %rax, -56(%rbp)
  movq -32(%rbp), %rax
  movq %rax, -64(%rbp)
  movabsq $5, %rax
  movq %rax, -352(%rbp)
  movabsq $6, %rax
  movq %rax, -360(%rbp)
  movq -352(%rbp), %rax
  movq %rax, -320(%rbp)
  movq -360(%rbp), %rax
  movq %rax, -328(%rbp)
  movabsq $7, %rax
  movq %rax, -368(%rbp)
  movabsq $8, %rax
  movq %rax, -376(%rbp)
  movq -368(%rbp), %rax
  movq %rax, -336(%rbp)
  movq -376(%rbp), %rax
  movq %rax, -344(%rbp)
  movq -320(%rbp), %rax
  movq %rax, -8(%rbp)
  movq -328(%rbp), %rax
  movq %rax, -16(%rbp)
  movq -336(%rbp), %rax
  movq %rax, -24(%rbp)
  movq -344(%rbp), %rax
  movq %rax, -32(%rbp)
  movabsq $1, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_0
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_0
  imulq $-2, %rax
  movq -40(%rbp,%rax,8), %rcx
  movq %rcx, -384(%rbp)
  movq -48(%rbp,%rax,8), %rcx
  movq %rcx, -392(%rbp)
  jmp .Lprimer_main_array_done_0
.Lprimer_main_array_oob_0:
  ud2
.Lprimer_main_array_done_0:
  movq -384(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $0, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_1
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_1
  imulq $-2, %rax
  movq -8(%rbp,%rax,8), %rcx
  movq %rcx, -400(%rbp)
  movq -16(%rbp,%rax,8), %rcx
  movq %rcx, -408(%rbp)
  jmp .Lprimer_main_array_done_1
.Lprimer_main_array_oob_1:
  ud2
.Lprimer_main_array_done_1:
  movq -408(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $448, %rsp
  popq %rbp
  retq
