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
	movsd ".Lfp1"(%rip), %xmm0
	leaq fmt_f32(%rip), %rdi
	movl $1, %eax
	callq printf
	movsd ".Lfp0"(%rip), %xmm0
	leaq fmt_f64(%rip), %rdi
	movl $1, %eax
	callq printf
	movsd ".Lfp0"(%rip), %xmm0
	leaq fmt_f64(%rip), %rdi
	movl $1, %eax
	callq printf
	movl $0, %eax
	leave
	ret
.type main, @function
.size main, .-main
/* end function main */

/* floating point constants */
.section .rodata
.p2align 3
.Lfp0:
	.quad 4599075939470750516 /* 0.300000 */

.section .rodata
.p2align 3
.Lfp1:
	.quad 4599075939685498880 /* 0.300000 */

.section .note.GNU-stack,"",@progbits
