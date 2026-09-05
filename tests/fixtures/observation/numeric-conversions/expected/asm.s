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
  .quad 0x0000000000000000

.text
.p2align 4
primer_fn_measure_0:
  pushq %rbp
  movq %rsp, %rbp
  subq $64, %rsp
  movq %rcx, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %r10
  cvtsi2sdq %rax, %xmm0
  movapd %xmm0, %xmm2
  movabsq $4890909195324358656, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jae .Lprimer_fn_0_convert_bad_0
  cvttsd2siq %xmm2, %rax
  cmpq %r10, %rax
  jne .Lprimer_fn_0_convert_bad_0
  jmp .Lprimer_fn_0_convert_done_0
.Lprimer_fn_0_convert_bad_0:
  ud2
.Lprimer_fn_0_convert_done_0:
  movsd %xmm0, -16(%rbp)
  movabsq $2, %rax
  movq %rax, %r10
  cvtsi2sdq %rax, %xmm0
  movapd %xmm0, %xmm2
  movabsq $4890909195324358656, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jae .Lprimer_fn_0_convert_bad_1
  cvttsd2siq %xmm2, %rax
  cmpq %r10, %rax
  jne .Lprimer_fn_0_convert_bad_1
  jmp .Lprimer_fn_0_convert_done_1
.Lprimer_fn_0_convert_bad_1:
  ud2
.Lprimer_fn_0_convert_done_1:
  movapd %xmm0, %xmm1
  movsd -16(%rbp), %xmm0
  divsd %xmm1, %xmm0
  addq $64, %rsp
  popq %rbp
  retq

.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $144, %rsp
  movabsq $42, %rax
  movq %rax, -8(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %r10
  cvtsi2sdq %rax, %xmm0
  movapd %xmm0, %xmm2
  movabsq $4890909195324358656, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jae .Lprimer_main_convert_bad_0
  cvttsd2siq %xmm2, %rax
  cmpq %r10, %rax
  jne .Lprimer_main_convert_bad_0
  jmp .Lprimer_main_convert_done_0
.Lprimer_main_convert_bad_0:
  ud2
.Lprimer_main_convert_done_0:
  movsd %xmm0, -16(%rbp)
  movsd -16(%rbp), %xmm0
  ucomisd %xmm0, %xmm0
  jp .Lprimer_main_convert_bad_1
  movapd %xmm0, %xmm2
  cvtsd2ss %xmm0, %xmm0
  cvtss2sd %xmm0, %xmm1
  ucomisd %xmm1, %xmm2
  jne .Lprimer_main_convert_bad_1
  jmp .Lprimer_main_convert_done_1
.Lprimer_main_convert_bad_1:
  ud2
.Lprimer_main_convert_done_1:
  movss %xmm0, -24(%rbp)
  movss -24(%rbp), %xmm0
  cvtss2sd %xmm0, %xmm2
  ucomisd %xmm2, %xmm2
  jp .Lprimer_main_convert_bad_2
  movq %xmm2, %r11
  movabsq $-9223372036854775808, %r10
  cmpq %r10, %r11
  je .Lprimer_main_convert_bad_2
  movabsq $-4548635623644200960, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jb .Lprimer_main_convert_bad_2
  movabsq $4674736413210574848, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jae .Lprimer_main_convert_bad_2
  cvttsd2siq %xmm2, %rax
  cvtsi2sdq %rax, %xmm1
  ucomisd %xmm1, %xmm2
  jne .Lprimer_main_convert_bad_2
  jmp .Lprimer_main_convert_done_2
.Lprimer_main_convert_bad_2:
  ud2
.Lprimer_main_convert_done_2:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movsd -16(%rbp), %xmm0
  movapd %xmm0, %xmm2
  ucomisd %xmm2, %xmm2
  jp .Lprimer_main_convert_bad_3
  movq %xmm2, %r11
  movabsq $-9223372036854775808, %r10
  cmpq %r10, %r11
  je .Lprimer_main_convert_bad_3
  movabsq $-4332462841530417152, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jb .Lprimer_main_convert_bad_3
  movabsq $4890909195324358656, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jae .Lprimer_main_convert_bad_3
  cvttsd2siq %xmm2, %rax
  cvtsi2sdq %rax, %xmm1
  ucomisd %xmm1, %xmm2
  jne .Lprimer_main_convert_bad_3
  jmp .Lprimer_main_convert_done_3
.Lprimer_main_convert_bad_3:
  ud2
.Lprimer_main_convert_done_3:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movss -24(%rbp), %xmm0
  ucomiss %xmm0, %xmm0
  jp .Lprimer_main_convert_bad_4
  cvtss2sd %xmm0, %xmm0
  jmp .Lprimer_main_convert_done_4
.Lprimer_main_convert_bad_4:
  ud2
.Lprimer_main_convert_done_4:
  movsd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f64(%rip), %rcx
  callq printf
  movq -8(%rbp), %rax
  movq %rax, %r10
  cvtsi2ssq %rax, %xmm0
  cvtss2sd %xmm0, %xmm2
  movabsq $4890909195324358656, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jae .Lprimer_main_convert_bad_5
  cvttsd2siq %xmm2, %rax
  cmpq %r10, %rax
  jne .Lprimer_main_convert_bad_5
  jmp .Lprimer_main_convert_done_5
.Lprimer_main_convert_bad_5:
  ud2
.Lprimer_main_convert_done_5:
  cvtss2sd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f32(%rip), %rcx
  callq printf
  movabsq $3, %rax
  movq %rax, -32(%rbp)
  movq -32(%rbp), %rcx
  callq primer_fn_measure_0
  movsd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f64(%rip), %rcx
  callq printf
  movsd .Lprimer_f64_0(%rip), %xmm0
  xorpd .Lprimer_sign_f64(%rip), %xmm0
  ucomisd %xmm0, %xmm0
  jp .Lprimer_main_convert_bad_6
  movapd %xmm0, %xmm2
  cvtsd2ss %xmm0, %xmm0
  cvtss2sd %xmm0, %xmm1
  ucomisd %xmm1, %xmm2
  jne .Lprimer_main_convert_bad_6
  jmp .Lprimer_main_convert_done_6
.Lprimer_main_convert_bad_6:
  ud2
.Lprimer_main_convert_done_6:
  cvtss2sd %xmm0, %xmm1
  movq %xmm1, %rdx
  leaq .Lprimer_fmt_f32(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $144, %rsp
  popq %rbp
  retq
