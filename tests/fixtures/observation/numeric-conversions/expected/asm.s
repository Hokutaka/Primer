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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_0(%rip), %rdx
  movl $63, %r8d
  callq _write
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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_1(%rip), %rdx
  movl $63, %r8d
  callq _write
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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_2(%rip), %rdx
  movl $64, %r8d
  callq _write
  ud2
.Lprimer_main_convert_done_0:
  movsd %xmm0, -16(%rbp)
  movsd -16(%rbp), %xmm0
  ucomisd %xmm0, %xmm0
  jp .Lprimer_main_convert_bad_1_nan
  movapd %xmm0, %xmm2
  movabsq $9218868437227405312, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  je .Lprimer_main_convert_bad_1_convert
  movabsq $-4503599627370496, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  je .Lprimer_main_convert_bad_1_convert
  movabsq $5183643170566569984, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  ja .Lprimer_main_convert_bad_1_range
  movabsq $-4039728866288205824, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jb .Lprimer_main_convert_bad_1_range
.Lprimer_main_convert_bad_1_convert:
  cvtsd2ss %xmm0, %xmm0
  cvtss2sd %xmm0, %xmm1
  ucomisd %xmm1, %xmm2
  jne .Lprimer_main_convert_bad_1
  jmp .Lprimer_main_convert_done_1
.Lprimer_main_convert_bad_1:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_3(%rip), %rdx
  movl $66, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_1_range:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_4(%rip), %rdx
  movl $71, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_1_nan:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_5(%rip), %rdx
  movl $62, %r8d
  callq _write
  ud2
.Lprimer_main_convert_done_1:
  movss %xmm0, -24(%rbp)
  movss -24(%rbp), %xmm0
  cvtss2sd %xmm0, %xmm2
  ucomisd %xmm2, %xmm2
  jp .Lprimer_main_convert_bad_2_nonfinite
  movabsq $9218868437227405312, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  je .Lprimer_main_convert_bad_2_nonfinite
  movabsq $-4503599627370496, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  je .Lprimer_main_convert_bad_2_nonfinite
  movq %xmm2, %r11
  movabsq $-9223372036854775808, %r10
  cmpq %r10, %r11
  jne .Lprimer_main_convert_bad_2_finite
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_6(%rip), %rdx
  movl $72, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_2_nonfinite:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_7(%rip), %rdx
  movl $69, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_2_finite:
  movabsq $-4548635623644200960, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jb .Lprimer_main_convert_bad_2_range
  movabsq $4674736413210574848, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jae .Lprimer_main_convert_bad_2_range
  cvttsd2siq %xmm2, %rax
  cvtsi2sdq %rax, %xmm1
  ucomisd %xmm1, %xmm2
  jne .Lprimer_main_convert_bad_2
  jmp .Lprimer_main_convert_done_2
.Lprimer_main_convert_bad_2:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_8(%rip), %rdx
  movl $66, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_2_range:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_9(%rip), %rdx
  movl $71, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_2_nan:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_10(%rip), %rdx
  movl $62, %r8d
  callq _write
  ud2
.Lprimer_main_convert_done_2:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movsd -16(%rbp), %xmm0
  movapd %xmm0, %xmm2
  ucomisd %xmm2, %xmm2
  jp .Lprimer_main_convert_bad_3_nonfinite
  movabsq $9218868437227405312, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  je .Lprimer_main_convert_bad_3_nonfinite
  movabsq $-4503599627370496, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  je .Lprimer_main_convert_bad_3_nonfinite
  movq %xmm2, %r11
  movabsq $-9223372036854775808, %r10
  cmpq %r10, %r11
  jne .Lprimer_main_convert_bad_3_finite
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_11(%rip), %rdx
  movl $72, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_3_nonfinite:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_12(%rip), %rdx
  movl $69, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_3_finite:
  movabsq $-4332462841530417152, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jb .Lprimer_main_convert_bad_3_range
  movabsq $4890909195324358656, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jae .Lprimer_main_convert_bad_3_range
  cvttsd2siq %xmm2, %rax
  cvtsi2sdq %rax, %xmm1
  ucomisd %xmm1, %xmm2
  jne .Lprimer_main_convert_bad_3
  jmp .Lprimer_main_convert_done_3
.Lprimer_main_convert_bad_3:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_13(%rip), %rdx
  movl $66, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_3_range:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_14(%rip), %rdx
  movl $71, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_3_nan:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_15(%rip), %rdx
  movl $62, %r8d
  callq _write
  ud2
.Lprimer_main_convert_done_3:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movss -24(%rbp), %xmm0
  ucomiss %xmm0, %xmm0
  jp .Lprimer_main_convert_bad_4_nan
  cvtss2sd %xmm0, %xmm0
  jmp .Lprimer_main_convert_done_4
.Lprimer_main_convert_bad_4:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_16(%rip), %rdx
  movl $66, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_4_range:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_17(%rip), %rdx
  movl $71, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_4_nan:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_18(%rip), %rdx
  movl $62, %r8d
  callq _write
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
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_19(%rip), %rdx
  movl $66, %r8d
  callq _write
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
  jp .Lprimer_main_convert_bad_6_nan
  movapd %xmm0, %xmm2
  movabsq $9218868437227405312, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  je .Lprimer_main_convert_bad_6_convert
  movabsq $-4503599627370496, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  je .Lprimer_main_convert_bad_6_convert
  movabsq $5183643170566569984, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  ja .Lprimer_main_convert_bad_6_range
  movabsq $-4039728866288205824, %r11
  movq %r11, %xmm1
  ucomisd %xmm1, %xmm2
  jb .Lprimer_main_convert_bad_6_range
.Lprimer_main_convert_bad_6_convert:
  cvtsd2ss %xmm0, %xmm0
  cvtss2sd %xmm0, %xmm1
  ucomisd %xmm1, %xmm2
  jne .Lprimer_main_convert_bad_6
  jmp .Lprimer_main_convert_done_6
.Lprimer_main_convert_bad_6:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_20(%rip), %rdx
  movl $66, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_6_range:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_21(%rip), %rdx
  movl $71, %r8d
  callq _write
  ud2
.Lprimer_main_convert_bad_6_nan:
  xorl %ecx, %ecx
  callq fflush
  movl $2, %ecx
  leaq .Lprimer_failure_22(%rip), %rdx
  movl $62, %r8d
  callq _write
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

.section .rdata,"dr"
.Lprimer_failure_0:
  .asciz "primer: runtime-v1 code=conversion-inexact node=2 bytes=43..53\n"
.Lprimer_failure_1:
  .asciz "primer: runtime-v1 code=conversion-inexact node=4 bytes=56..62\n"
.Lprimer_failure_2:
  .asciz "primer: runtime-v1 code=conversion-inexact node=9 bytes=95..114\n"
.Lprimer_failure_3:
  .asciz "primer: runtime-v1 code=conversion-inexact node=12 bytes=130..139\n"
.Lprimer_failure_4:
  .asciz "primer: runtime-v1 code=conversion-out-of-range node=12 bytes=130..139\n"
.Lprimer_failure_5:
  .asciz "primer: runtime-v1 code=conversion-nan node=12 bytes=130..139\n"
.Lprimer_failure_6:
  .asciz "primer: runtime-v1 code=conversion-negative-zero node=15 bytes=147..158\n"
.Lprimer_failure_7:
  .asciz "primer: runtime-v1 code=conversion-not-finite node=15 bytes=147..158\n"
.Lprimer_failure_8:
  .asciz "primer: runtime-v1 code=conversion-inexact node=15 bytes=147..158\n"
.Lprimer_failure_9:
  .asciz "primer: runtime-v1 code=conversion-out-of-range node=15 bytes=147..158\n"
.Lprimer_failure_10:
  .asciz "primer: runtime-v1 code=conversion-nan node=15 bytes=147..158\n"
.Lprimer_failure_11:
  .asciz "primer: runtime-v1 code=conversion-negative-zero node=18 bytes=167..176\n"
.Lprimer_failure_12:
  .asciz "primer: runtime-v1 code=conversion-not-finite node=18 bytes=167..176\n"
.Lprimer_failure_13:
  .asciz "primer: runtime-v1 code=conversion-inexact node=18 bytes=167..176\n"
.Lprimer_failure_14:
  .asciz "primer: runtime-v1 code=conversion-out-of-range node=18 bytes=167..176\n"
.Lprimer_failure_15:
  .asciz "primer: runtime-v1 code=conversion-nan node=18 bytes=167..176\n"
.Lprimer_failure_16:
  .asciz "primer: runtime-v1 code=conversion-inexact node=21 bytes=185..196\n"
.Lprimer_failure_17:
  .asciz "primer: runtime-v1 code=conversion-out-of-range node=21 bytes=185..196\n"
.Lprimer_failure_18:
  .asciz "primer: runtime-v1 code=conversion-nan node=21 bytes=185..196\n"
.Lprimer_failure_19:
  .asciz "primer: runtime-v1 code=conversion-inexact node=24 bytes=205..215\n"
.Lprimer_failure_20:
  .asciz "primer: runtime-v1 code=conversion-inexact node=30 bytes=243..252\n"
.Lprimer_failure_21:
  .asciz "primer: runtime-v1 code=conversion-out-of-range node=30 bytes=243..252\n"
.Lprimer_failure_22:
  .asciz "primer: runtime-v1 code=conversion-nan node=30 bytes=243..252\n"
