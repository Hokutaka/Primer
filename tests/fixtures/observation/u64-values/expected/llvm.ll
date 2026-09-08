@.fmt_u64 = private unnamed_addr constant [6 x i8] c"%llu\0A\00"
@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
declare void @llvm.trap()

define internal i64 @primer_convert_i64_u64(i64 %value) {
entry:
  %bad = icmp slt i64 %value, 0
  br i1 %bad, label %trap, label %ok
ok:
  ret i64 %value
trap:
  call void @llvm.trap()
  unreachable
}

define internal i64 @primer_u64_div(i64 %left, i64 %right) {
entry:
  %bad = icmp eq i64 %right, 0
  br i1 %bad, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  %result = udiv i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_u64_shr(i64 %left, i64 %right) {
entry:
  %wide = icmp uge i64 %right, 64
  br i1 %wide, label %trap, label %bounds
bounds:
  %bad = icmp ne i64 0, 0
  br i1 %bad, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  %result = lshr i64 %left, %right
  ret i64 %result
}

define i32 @main() {
entry:
  %primer_maximum = alloca i64
  store i64 -1, ptr %primer_maximum
  %tmp0 = load i64, ptr %primer_maximum
  call i32 (ptr, ...) @printf(ptr @.fmt_u64, i64 %tmp0)
  %tmp1 = load i64, ptr %primer_maximum
  %tmp2 = icmp ugt i64 %tmp1, 0
  %tmp3 = select i1 %tmp2, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp3)
  %tmp4 = load i64, ptr %primer_maximum
  %tmp5 = call i64 @primer_u64_div(i64 %tmp4, i64 2)
  call i32 (ptr, ...) @printf(ptr @.fmt_u64, i64 %tmp5)
  %tmp6 = load i64, ptr %primer_maximum
  %tmp7 = call i64 @primer_u64_shr(i64 %tmp6, i64 63)
  call i32 (ptr, ...) @printf(ptr @.fmt_u64, i64 %tmp7)
  %tmp8 = call i64 @primer_convert_i64_u64(i64 42)
  call i32 (ptr, ...) @printf(ptr @.fmt_u64, i64 %tmp8)
  ret i32 0
}
