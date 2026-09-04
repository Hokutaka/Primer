@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

declare i32 @printf(ptr, ...)
declare void @llvm.trap()
declare { i64, i1 } @llvm.sadd.with.overflow.i64(i64, i64)

define internal i64 @primer_i64_add(i64 %left, i64 %right) {
entry:
  %checked = call { i64, i1 } @llvm.sadd.with.overflow.i64(i64 %left, i64 %right)
  %result = extractvalue { i64, i1 } %checked, 0
  %overflow = extractvalue { i64, i1 } %checked, 1
  br i1 %overflow, label %trap, label %ok

trap:
  call void @llvm.trap()
  unreachable

ok:
  ret i64 %result
}

define i64 @primer.fn.add.0(i64 %arg0, i64 %arg1) {
entry:
  %primer_left = alloca i64
  %primer_right = alloca i64
  store i64 %arg0, ptr %primer_left
  store i64 %arg1, ptr %primer_right
  %tmp0 = load i64, ptr %primer_left
  %tmp1 = load i64, ptr %primer_right
  %tmp2 = call i64 @primer_i64_add(i64 %tmp0, i64 %tmp1)
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
