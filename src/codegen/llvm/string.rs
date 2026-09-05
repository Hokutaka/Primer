use std::fmt::Write;

use super::{Target, ir::Module};

pub(super) fn emit_data(module: &Module, output: &mut String) {
    output.push_str("%primer.string = type { ptr, i64 }\n");
    for (id, value) in module.strings.iter().enumerate() {
        write!(
            output,
            "@primer.string.{id} = private unnamed_addr constant [{} x i8] c\"",
            value.len()
        )
        .unwrap();
        // UTF-8の各バイトを固定2桁の16進表記にし、NULもそのまま保持します。
        for byte in value.bytes() {
            write!(output, "\\{byte:02X}").unwrap();
        }
        output.push_str("\"\n");
    }
    output.push('\n');
}

pub(super) fn emit_support(module: &Module, output: &mut String) {
    if module.target == Some(Target::X86_64PcWindowsMsvc) {
        output.push_str("declare i32 @_setmode(i32, i32)\n");
    }
    output.push_str(SUPPORT);
}

// 比較と出力をバイト数で制御します。空文字列ではポインタを読みません。
// putcharはprintf/putsと同じ標準出力を使うため、混在時にも出力順を保てます。
const SUPPORT: &str = r#"declare i32 @putchar(i32)

define internal i1 @primer.string.equal(%primer.string %left, %primer.string %right) {
entry:
  %left.data = extractvalue %primer.string %left, 0
  %left.length = extractvalue %primer.string %left, 1
  %right.data = extractvalue %primer.string %right, 0
  %right.length = extractvalue %primer.string %right, 1
  %same.length = icmp eq i64 %left.length, %right.length
  br i1 %same.length, label %condition, label %different
condition:
  %index = phi i64 [ 0, %entry ], [ %next, %advance ]
  %done = icmp eq i64 %index, %left.length
  br i1 %done, label %equal, label %compare
compare:
  %left.ptr = getelementptr inbounds i8, ptr %left.data, i64 %index
  %right.ptr = getelementptr inbounds i8, ptr %right.data, i64 %index
  %left.byte = load i8, ptr %left.ptr
  %right.byte = load i8, ptr %right.ptr
  %same.byte = icmp eq i8 %left.byte, %right.byte
  br i1 %same.byte, label %advance, label %different
advance:
  %next = add i64 %index, 1
  br label %condition
equal:
  ret i1 true
different:
  ret i1 false
}

define internal void @primer.print.string(%primer.string %value) {
entry:
  %data = extractvalue %primer.string %value, 0
  %length = extractvalue %primer.string %value, 1
  br label %condition
condition:
  %index = phi i64 [ 0, %entry ], [ %next, %write ]
  %done = icmp eq i64 %index, %length
  br i1 %done, label %newline, label %write
write:
  %ptr = getelementptr inbounds i8, ptr %data, i64 %index
  %byte = load i8, ptr %ptr
  %character = zext i8 %byte to i32
  call i32 @putchar(i32 %character)
  %next = add i64 %index, 1
  br label %condition
newline:
  call i32 @putchar(i32 10)
  ret void
}

"#;
