@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
declare void @llvm.trap()
declare { i64, i1 } @llvm.ssub.with.overflow.i64(i64, i64)

define internal i64 @primer_i64_sub(i64 %left, i64 %right) {
entry:
  %checked = call { i64, i1 } @llvm.ssub.with.overflow.i64(i64 %left, i64 %right)
  %result = extractvalue { i64, i1 } %checked, 0
  %overflow = extractvalue { i64, i1 } %checked, 1
  br i1 %overflow, label %trap, label %ok

trap:
  call void @llvm.trap()
  unreachable

ok:
  ret i64 %result
}

define i32 @main() {
entry:
  %primer_value = alloca i64
  %primer_value_1 = alloca i1
  store i64 1, ptr %primer_value
  %tmp0 = load i64, ptr %primer_value
  %tmp1 = icmp slt i64 %tmp0, 2
  br i1 %tmp1, label %block0, label %block1
block0: ; if_then
  store i64 42, ptr %primer_value
  store i1 1, ptr %primer_value_1
  %tmp2 = load i1, ptr %primer_value_1
  %tmp3 = select i1 %tmp2, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp3)
  br label %block2
block1: ; if_else
  %tmp4 = call i64 @primer_i64_sub(i64 0, i64 1)
  store i64 %tmp4, ptr %primer_value
  br label %block2
block2: ; if_end
  %tmp5 = load i64, ptr %primer_value
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp5)
  ret i32 0
}
