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
.p2align 3
.Lprimer_f64_0:
  .quad 0x4000000000000000
.p2align 3
.Lprimer_f64_1:
  .quad 0x0000000000000000
.p2align 3
.Lprimer_f64_2:
  .quad 0x4010000000000000
.p2align 3
.Lprimer_f64_3:
  .quad 0x4014000000000000

.text
.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $320, %rsp
  movsd .Lprimer_f64_0(%rip), %xmm0
  movsd %xmm0, -240(%rbp)
  movsd .Lprimer_f64_1(%rip), %xmm0
  movsd %xmm0, -232(%rbp)
  movsd -232(%rbp), %xmm0
  movsd %xmm0, -8(%rbp)
  movsd -240(%rbp), %xmm0
  movsd %xmm0, -16(%rbp)
  movsd -8(%rbp), %xmm0
  movsd %xmm0, -24(%rbp)
  movsd -16(%rbp), %xmm0
  movsd %xmm0, -32(%rbp)
  movsd .Lprimer_f64_2(%rip), %xmm0
  movsd %xmm0, -248(%rbp)
  movsd .Lprimer_f64_3(%rip), %xmm0
  movsd %xmm0, -256(%rbp)
  movsd -248(%rbp), %xmm0
  movsd %xmm0, -8(%rbp)
  movsd -256(%rbp), %xmm0
  movsd %xmm0, -16(%rbp)
  movsd -24(%rbp), %xmm0
  movsd %xmm0, -264(%rbp)
  movsd -32(%rbp), %xmm0
  movsd %xmm0, -272(%rbp)
  movsd -8(%rbp), %xmm0
  movsd %xmm0, -280(%rbp)
  movsd -16(%rbp), %xmm0
  movsd %xmm0, -288(%rbp)
  movsd -264(%rbp), %xmm0
  movsd %xmm0, -40(%rbp)
  movsd -272(%rbp), %xmm0
  movsd %xmm0, -48(%rbp)
  movsd -280(%rbp), %xmm0
  movsd %xmm0, -56(%rbp)
  movsd -288(%rbp), %xmm0
  movsd %xmm0, -64(%rbp)
  movsd -24(%rbp), %xmm0
  movsd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f64(%rip), %rcx
  callq printf
  movsd -32(%rbp), %xmm0
  movsd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f64(%rip), %rcx
  callq printf
  movsd -48(%rbp), %xmm0
  movsd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f64(%rip), %rcx
  callq printf
  movsd -56(%rbp), %xmm0
  movsd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $320, %rsp
  popq %rbp
  retq
