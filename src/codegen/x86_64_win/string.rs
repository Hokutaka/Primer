use super::ir::Module;
use std::fmt::Write;

pub(super) fn emit_data(module: &Module, output: &mut String) {
    for (id, value) in module.strings.iter().enumerate() {
        writeln!(
            output,
            ".p2align 3\n.Lprimer_string_{id}:\n  .quad {}",
            value.len()
        )
        .unwrap();
        for byte in value.bytes() {
            writeln!(output, "  .byte {byte}").unwrap();
        }
    }
}

// 比較は揮発レジスタだけで行い、出力はshadow spaceと16バイト境界を守ります。
pub(super) const SUPPORT: &str = r#"
.p2align 4
primer_string_equal:
  movq (%rcx), %r8
  cmpq (%rdx), %r8
  jne .Lstring_different
  xorq %r9, %r9
.Lstring_compare:
  cmpq %r8, %r9
  je .Lstring_equal
  movzbl 8(%rcx,%r9), %eax
  cmpb 8(%rdx,%r9), %al
  jne .Lstring_different
  incq %r9
  jmp .Lstring_compare
.Lstring_equal:
  movl $1, %eax
  retq
.Lstring_different:
  xorl %eax, %eax
  retq

.p2align 4
primer_print_string:
  subq $56, %rsp
  movq %rcx, 32(%rsp)
  movq $0, 40(%rsp)
.Lstring_write:
  movq 32(%rsp), %rax
  movq 40(%rsp), %rdx
  cmpq (%rax), %rdx
  je .Lstring_newline
  movzbl 8(%rax,%rdx), %ecx
  callq putchar
  incq 40(%rsp)
  jmp .Lstring_write
.Lstring_newline:
  movl $10, %ecx
  callq putchar
  addq $56, %rsp
  retq

"#;
