@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

declare i32 @printf(ptr, ...)
declare void @llvm.trap()
declare { i64, i1 } @llvm.sadd.with.overflow.i64(i64, i64)
declare { i64, i1 } @llvm.ssub.with.overflow.i64(i64, i64)
declare { i64, i1 } @llvm.smul.with.overflow.i64(i64, i64)

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

define internal i64 @primer_i64_mul(i64 %left, i64 %right) {
entry:
  %checked = call { i64, i1 } @llvm.smul.with.overflow.i64(i64 %left, i64 %right)
  %result = extractvalue { i64, i1 } %checked, 0
  %overflow = extractvalue { i64, i1 } %checked, 1
  br i1 %overflow, label %trap, label %ok

trap:
  call void @llvm.trap()
  unreachable

ok:
  ret i64 %result
}

define internal i64 @primer_i64_div(i64 %left, i64 %right) {
entry:
  %is_zero = icmp eq i64 %right, 0
  %is_min = icmp eq i64 %left, -9223372036854775808
  %is_negative_one = icmp eq i64 %right, -1
  %overflows = and i1 %is_min, %is_negative_one
  %invalid = or i1 %is_zero, %overflows
  br i1 %invalid, label %trap, label %ok

trap:
  call void @llvm.trap()
  unreachable

ok:
  %result = sdiv i64 %left, %right
  ret i64 %result
}

define i32 @main() {
entry:
  %primer_value = alloca i64
  store i64 8, ptr %primer_value
  %tmp0 = load i64, ptr %primer_value
  %tmp1 = call i64 @primer_i64_add(i64 %tmp0, i64 1)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp1)
  %tmp2 = load i64, ptr %primer_value
  %tmp3 = call i64 @primer_i64_sub(i64 %tmp2, i64 1)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp3)
  %tmp4 = load i64, ptr %primer_value
  %tmp5 = call i64 @primer_i64_mul(i64 %tmp4, i64 2)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp5)
  %tmp6 = load i64, ptr %primer_value
  %tmp7 = call i64 @primer_i64_div(i64 %tmp6, i64 2)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp7)
  %tmp8 = load i64, ptr %primer_value
  %tmp9 = call i64 @primer_i64_sub(i64 0, i64 %tmp8)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp9)
  ret i32 0
}
