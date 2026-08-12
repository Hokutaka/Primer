.data
.balign 8
fmt_i64:
	.ascii "%lld"
	.byte 10
	.byte 0
/* end data */

.data
.balign 8
fmt_f32:
	.ascii "%.9g"
	.byte 10
	.byte 0
/* end data */

.data
.balign 8
fmt_f64:
	.ascii "%.17g"
	.byte 10
	.byte 0
/* end data */

.text
.balign 16
.globl main
main:
	endbr64
	pushq %rbp
	movq %rsp, %rbp
	subq $32, %rsp
	movsd "Lfp1"(%rip), %xmm1
	leaq fmt_f32(%rip), %rcx
	movq %xmm1, %rdx
	callq printf
	subq $-32, %rsp
	subq $32, %rsp
	movsd "Lfp0"(%rip), %xmm1
	leaq fmt_f64(%rip), %rcx
	movq %xmm1, %rdx
	callq printf
	subq $-32, %rsp
	subq $32, %rsp
	movsd "Lfp0"(%rip), %xmm1
	leaq fmt_f64(%rip), %rcx
	movq %xmm1, %rdx
	callq printf
	subq $-32, %rsp
	movl $0, %eax
	leave
	ret
/* end function main */

/* floating point constants */
.section .rodata
.p2align 3
Lfp0:
	.quad 4599075939470750516 /* 0.300000 */

.section .rodata
.p2align 3
Lfp1:
	.quad 4599075939685498880 /* 0.300000 */

