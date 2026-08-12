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
.p2align 2
.Lprimer_f32_0:
  .long 0x3DCCCCCD
.p2align 2
.Lprimer_f32_1:
  .long 0x3E4CCCCD
.p2align 3
.Lprimer_f64_2:
  .quad 0x3FB999999999999A
.p2align 3
.Lprimer_f64_3:
  .quad 0x3FC999999999999A
.p2align 3
.Lprimer_f64_4:
  .quad 0x3FB999999999999A
.p2align 3
.Lprimer_f64_5:
  .quad 0x3FC999999999999A

.text
.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $160, %rsp
  movss .Lprimer_f32_0(%rip), %xmm0
  movss %xmm0, -32(%rbp)
  movss .Lprimer_f32_1(%rip), %xmm0
  movaps %xmm0, %xmm1
  movss -32(%rbp), %xmm0
  addss %xmm1, %xmm0
  movss %xmm0, -8(%rbp)
  movsd .Lprimer_f64_2(%rip), %xmm0
  movsd %xmm0, -32(%rbp)
  movsd .Lprimer_f64_3(%rip), %xmm0
  movapd %xmm0, %xmm1
  movsd -32(%rbp), %xmm0
  addsd %xmm1, %xmm0
  movsd %xmm0, -16(%rbp)
  movsd .Lprimer_f64_4(%rip), %xmm0
  movsd %xmm0, -32(%rbp)
  movsd .Lprimer_f64_5(%rip), %xmm0
  movapd %xmm0, %xmm1
  movsd -32(%rbp), %xmm0
  addsd %xmm1, %xmm0
  movsd %xmm0, -24(%rbp)
  movss -8(%rbp), %xmm0
  cvtss2sd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f32(%rip), %rcx
  callq printf
  movsd -16(%rbp), %xmm0
  movsd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f64(%rip), %rcx
  callq printf
  movsd -24(%rbp), %xmm0
  movsd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $160, %rsp
  popq %rbp
  retq
