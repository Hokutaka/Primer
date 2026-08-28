@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %primer_x = alloca i64
  %tmp0 = add i64 1, 2
  store i64 %tmp0, ptr %primer_x
  %tmp1 = load i64, ptr %primer_x
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp1)
  ret i32 0
}
