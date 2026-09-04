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
  subq $608, %rsp
  movabsq $1, %rax
  movq %rax, -384(%rbp)
  movabsq $2, %rax
  movq %rax, -392(%rbp)
  movabsq $3, %rax
  movq %rax, -400(%rbp)
  movq -384(%rbp), %rax
  movq %rax, -336(%rbp)
  movq -392(%rbp), %rax
  movq %rax, -344(%rbp)
  movq -400(%rbp), %rax
  movq %rax, -352(%rbp)
  movabsq $4, %rax
  movq %rax, -408(%rbp)
  movabsq $5, %rax
  movq %rax, -416(%rbp)
  movabsq $6, %rax
  movq %rax, -424(%rbp)
  movq -408(%rbp), %rax
  movq %rax, -360(%rbp)
  movq -416(%rbp), %rax
  movq %rax, -368(%rbp)
  movq -424(%rbp), %rax
  movq %rax, -376(%rbp)
  movq -336(%rbp), %rax
  movq %rax, -8(%rbp)
  movq -344(%rbp), %rax
  movq %rax, -16(%rbp)
  movq -352(%rbp), %rax
  movq %rax, -24(%rbp)
  movq -360(%rbp), %rax
  movq %rax, -32(%rbp)
  movq -368(%rbp), %rax
  movq %rax, -40(%rbp)
  movq -376(%rbp), %rax
  movq %rax, -48(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -56(%rbp)
  movq -16(%rbp), %rax
  movq %rax, -64(%rbp)
  movq -24(%rbp), %rax
  movq %rax, -72(%rbp)
  movq -32(%rbp), %rax
  movq %rax, -80(%rbp)
  movq -40(%rbp), %rax
  movq %rax, -88(%rbp)
  movq -48(%rbp), %rax
  movq %rax, -96(%rbp)
  movabsq $7, %rax
  movq %rax, -480(%rbp)
  movabsq $8, %rax
  movq %rax, -488(%rbp)
  movabsq $9, %rax
  movq %rax, -496(%rbp)
  movq -480(%rbp), %rax
  movq %rax, -432(%rbp)
  movq -488(%rbp), %rax
  movq %rax, -440(%rbp)
  movq -496(%rbp), %rax
  movq %rax, -448(%rbp)
  movabsq $10, %rax
  movq %rax, -504(%rbp)
  movabsq $11, %rax
  movq %rax, -512(%rbp)
  movabsq $12, %rax
  movq %rax, -520(%rbp)
  movq -504(%rbp), %rax
  movq %rax, -456(%rbp)
  movq -512(%rbp), %rax
  movq %rax, -464(%rbp)
  movq -520(%rbp), %rax
  movq %rax, -472(%rbp)
  movq -432(%rbp), %rax
  movq %rax, -8(%rbp)
  movq -440(%rbp), %rax
  movq %rax, -16(%rbp)
  movq -448(%rbp), %rax
  movq %rax, -24(%rbp)
  movq -456(%rbp), %rax
  movq %rax, -32(%rbp)
  movq -464(%rbp), %rax
  movq %rax, -40(%rbp)
  movq -472(%rbp), %rax
  movq %rax, -48(%rbp)
  movabsq $1, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_0
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_0
  imulq $-3, %rax
  movq -56(%rbp,%rax,8), %rcx
  movq %rcx, -528(%rbp)
  movq -64(%rbp,%rax,8), %rcx
  movq %rcx, -536(%rbp)
  movq -72(%rbp,%rax,8), %rcx
  movq %rcx, -544(%rbp)
  jmp .Lprimer_main_array_done_0
.Lprimer_main_array_oob_0:
  ud2
.Lprimer_main_array_done_0:
  movabsq $2, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_1
  cmpq $3, %rax
  jge .Lprimer_main_array_oob_1
  negq %rax
  movq -528(%rbp,%rax,8), %rax
  jmp .Lprimer_main_array_done_1
.Lprimer_main_array_oob_1:
  ud2
.Lprimer_main_array_done_1:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $0, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_2
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_2
  imulq $-3, %rax
  movq -8(%rbp,%rax,8), %rcx
  movq %rcx, -552(%rbp)
  movq -16(%rbp,%rax,8), %rcx
  movq %rcx, -560(%rbp)
  movq -24(%rbp,%rax,8), %rcx
  movq %rcx, -568(%rbp)
  jmp .Lprimer_main_array_done_2
.Lprimer_main_array_oob_2:
  ud2
.Lprimer_main_array_done_2:
  movabsq $1, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_3
  cmpq $3, %rax
  jge .Lprimer_main_array_oob_3
  negq %rax
  movq -552(%rbp,%rax,8), %rax
  jmp .Lprimer_main_array_done_3
.Lprimer_main_array_oob_3:
  ud2
.Lprimer_main_array_done_3:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $608, %rsp
  popq %rbp
  retq
