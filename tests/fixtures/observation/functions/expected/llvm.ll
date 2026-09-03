@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

declare i32 @printf(ptr, ...)

define i64 @primer.fn.add.0(i64 %arg0, i64 %arg1) {
entry:
  %primer_left = alloca i64
  %primer_right = alloca i64
  store i64 %arg0, ptr %primer_left
  store i64 %arg1, ptr %primer_right
  %tmp0 = load i64, ptr %primer_left
  %tmp1 = load i64, ptr %primer_right
  %tmp2 = add i64 %tmp0, %tmp1
  ret i64 %tmp2
}

define void @primer.fn.show.1(i64 %arg0) {
entry:
  %primer_value = alloca i64
  store i64 %arg0, ptr %primer_value
  %tmp0 = load i64, ptr %primer_value
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp0)
  ret void
}

define i32 @main() {
entry:
  %primer_answer = alloca i64
  %tmp0 = call i64 @primer.fn.add.0(i64 20, i64 22)
  store i64 %tmp0, ptr %primer_answer
  %tmp1 = load i64, ptr %primer_answer
  call void @primer.fn.show.1(i64 %tmp1)
  ret i32 0
}
