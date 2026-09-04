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
.p2align 4
primer_fn_move_x_0:
  pushq %rbp
  movq %rsp, %rbp
  subq $144, %rsp
  movq 0(%rcx), %r10
  movq %r10, -8(%rbp)
  movq -8(%rcx), %r10
  movq %r10, -16(%rbp)
  movq %rdx, -24(%rbp)
  movq %rax, -88(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -32(%rbp)
  movq -24(%rbp), %rax
  movq %rax, %rcx
  movq -32(%rbp), %rax
  addq %rcx, %rax
  jno .Lprimer_fn_0_integer_ok_0
  ud2
.Lprimer_fn_0_integer_ok_0:
  movq %rax, -96(%rbp)
  movq -16(%rbp), %rax
  movq %rax, -104(%rbp)
  movq -88(%rbp), %r11
  movq -96(%rbp), %r10
  movq %r10, 0(%r11)
  movq -104(%rbp), %r10
  movq %r10, -8(%r11)
  addq $144, %rsp
  popq %rbp
  retq

.p2align 4
primer_fn_move_twice_1:
  pushq %rbp
  movq %rsp, %rbp
  subq $144, %rsp
  movq 0(%rcx), %r10
  movq %r10, -8(%rbp)
  movq -8(%rcx), %r10
  movq %r10, -16(%rbp)
  movq %rdx, -24(%rbp)
  movq %rax, -80(%rbp)
  movq -24(%rbp), %rax
  movq %rax, -72(%rbp)
  leaq -8(%rbp), %rcx
  movq -72(%rbp), %rdx
  leaq -88(%rbp), %rax
  callq primer_fn_move_x_0
  movq -24(%rbp), %rax
  movq %rax, -40(%rbp)
  leaq -88(%rbp), %rcx
  movq -40(%rbp), %rdx
  leaq -104(%rbp), %rax
  callq primer_fn_move_x_0
  movq -80(%rbp), %r11
  movq -104(%rbp), %r10
  movq %r10, 0(%r11)
  movq -112(%rbp), %r10
  movq %r10, -8(%r11)
  addq $144, %rsp
  popq %rbp
  retq

.p2align 4
primer_fn_first_row_2:
  pushq %rbp
  movq %rsp, %rbp
  subq $112, %rsp
  movq 0(%rcx), %r10
  movq %r10, -8(%rbp)
  movq -8(%rcx), %r10
  movq %r10, -16(%rbp)
  movq -16(%rcx), %r10
  movq %r10, -24(%rbp)
  movq -24(%rcx), %r10
  movq %r10, -32(%rbp)
  movq %rax, -64(%rbp)
  movabsq $0, %rax
  testq %rax, %rax
  js .Lprimer_fn_2_array_oob_0
  cmpq $2, %rax
  jge .Lprimer_fn_2_array_oob_0
  imulq $-2, %rax
  movq -8(%rbp,%rax,8), %rcx
  movq %rcx, -72(%rbp)
  movq -16(%rbp,%rax,8), %rcx
  movq %rcx, -80(%rbp)
  jmp .Lprimer_fn_2_array_done_0
.Lprimer_fn_2_array_oob_0:
  ud2
.Lprimer_fn_2_array_done_0:
  movq -64(%rbp), %r11
  movq -72(%rbp), %r10
  movq %r10, 0(%r11)
  movq -80(%rbp), %r10
  movq %r10, -8(%r11)
  addq $112, %rsp
  popq %rbp
  retq

.p2align 4
primer_fn_duplicate_3:
  pushq %rbp
  movq %rsp, %rbp
  subq $112, %rsp
  movq 0(%rcx), %r10
  movq %r10, -8(%rbp)
  movq -8(%rcx), %r10
  movq %r10, -16(%rbp)
  movq %rax, -48(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -56(%rbp)
  movq -16(%rbp), %rax
  movq %rax, -64(%rbp)
  movq -8(%rbp), %rax
  movq %rax, -72(%rbp)
  movq -16(%rbp), %rax
  movq %rax, -80(%rbp)
  movq -48(%rbp), %r11
  movq -56(%rbp), %r10
  movq %r10, 0(%r11)
  movq -64(%rbp), %r10
  movq %r10, -8(%r11)
  movq -72(%rbp), %r10
  movq %r10, -16(%r11)
  movq -80(%rbp), %r10
  movq %r10, -24(%r11)
  addq $112, %rsp
  popq %rbp
  retq

.p2align 4
primer_fn_duplicate_first_row_4:
  pushq %rbp
  movq %rsp, %rbp
  subq $160, %rsp
  movq 0(%rcx), %r10
  movq %r10, -8(%rbp)
  movq -8(%rcx), %r10
  movq %r10, -16(%rbp)
  movq -16(%rcx), %r10
  movq %r10, -24(%rbp)
  movq -24(%rcx), %r10
  movq %r10, -32(%rbp)
  movq %rax, -80(%rbp)
  leaq -8(%rbp), %rcx
  leaq -88(%rbp), %rax
  callq primer_fn_first_row_2
  leaq -88(%rbp), %rcx
  leaq -104(%rbp), %rax
  callq primer_fn_duplicate_3
  movq -80(%rbp), %r11
  movq -104(%rbp), %r10
  movq %r10, 0(%r11)
  movq -112(%rbp), %r10
  movq %r10, -8(%r11)
  movq -120(%rbp), %r10
  movq %r10, -16(%r11)
  movq -128(%rbp), %r10
  movq %r10, -24(%r11)
  addq $160, %rsp
  popq %rbp
  retq

.globl main
.p2align 4
main:
  pushq %rbp
  movq %rsp, %rbp
  subq $592, %rsp
  movabsq $2, %rax
  movq %rax, -392(%rbp)
  movabsq $3, %rax
  movq %rax, -400(%rbp)
  movq -392(%rbp), %rax
  movq %rax, -8(%rbp)
  movq -400(%rbp), %rax
  movq %rax, -16(%rbp)
  movabsq $5, %rax
  movq %rax, -112(%rbp)
  leaq -8(%rbp), %rcx
  movq -112(%rbp), %rdx
  leaq -408(%rbp), %rax
  callq primer_fn_move_twice_1
  movq -408(%rbp), %rax
  movq %rax, -24(%rbp)
  movq -416(%rbp), %rax
  movq %rax, -32(%rbp)
  movabsq $1, %rax
  movq %rax, -456(%rbp)
  movabsq $2, %rax
  movq %rax, -464(%rbp)
  movq -456(%rbp), %rax
  movq %rax, -424(%rbp)
  movq -464(%rbp), %rax
  movq %rax, -432(%rbp)
  movabsq $3, %rax
  movq %rax, -472(%rbp)
  movabsq $4, %rax
  movq %rax, -480(%rbp)
  movq -472(%rbp), %rax
  movq %rax, -440(%rbp)
  movq -480(%rbp), %rax
  movq %rax, -448(%rbp)
  movq -424(%rbp), %rax
  movq %rax, -40(%rbp)
  movq -432(%rbp), %rax
  movq %rax, -48(%rbp)
  movq -440(%rbp), %rax
  movq %rax, -56(%rbp)
  movq -448(%rbp), %rax
  movq %rax, -64(%rbp)
  leaq -40(%rbp), %rcx
  leaq -488(%rbp), %rax
  callq primer_fn_duplicate_first_row_4
  movq -488(%rbp), %rax
  movq %rax, -72(%rbp)
  movq -496(%rbp), %rax
  movq %rax, -80(%rbp)
  movq -504(%rbp), %rax
  movq %rax, -88(%rbp)
  movq -512(%rbp), %rax
  movq %rax, -96(%rbp)
  movq -8(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -24(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movq -32(%rbp), %rax
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $1, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_0
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_0
  imulq $-2, %rax
  movq -40(%rbp,%rax,8), %rcx
  movq %rcx, -520(%rbp)
  movq -48(%rbp,%rax,8), %rcx
  movq %rcx, -528(%rbp)
  jmp .Lprimer_main_array_done_0
.Lprimer_main_array_oob_0:
  ud2
.Lprimer_main_array_done_0:
  movabsq $0, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_1
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_1
  negq %rax
  movq -520(%rbp,%rax,8), %rax
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
  imulq $-2, %rax
  movq -72(%rbp,%rax,8), %rcx
  movq %rcx, -536(%rbp)
  movq -80(%rbp,%rax,8), %rcx
  movq %rcx, -544(%rbp)
  jmp .Lprimer_main_array_done_2
.Lprimer_main_array_oob_2:
  ud2
.Lprimer_main_array_done_2:
  movabsq $1, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_3
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_3
  negq %rax
  movq -536(%rbp,%rax,8), %rax
  jmp .Lprimer_main_array_done_3
.Lprimer_main_array_oob_3:
  ud2
.Lprimer_main_array_done_3:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  movabsq $1, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_4
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_4
  imulq $-2, %rax
  movq -72(%rbp,%rax,8), %rcx
  movq %rcx, -552(%rbp)
  movq -80(%rbp,%rax,8), %rcx
  movq %rcx, -560(%rbp)
  jmp .Lprimer_main_array_done_4
.Lprimer_main_array_oob_4:
  ud2
.Lprimer_main_array_done_4:
  movabsq $0, %rax
  testq %rax, %rax
  js .Lprimer_main_array_oob_5
  cmpq $2, %rax
  jge .Lprimer_main_array_oob_5
  negq %rax
  movq -552(%rbp,%rax,8), %rax
  jmp .Lprimer_main_array_done_5
.Lprimer_main_array_oob_5:
  ud2
.Lprimer_main_array_done_5:
  movq %rax, %rdx
  leaq .Lprimer_fmt_i64(%rip), %rcx
  callq printf
  xorl %eax, %eax
  addq $592, %rsp
  popq %rbp
  retq
