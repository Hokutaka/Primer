@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
declare void @llvm.trap()
declare { i64, i1 } @llvm.sadd.with.overflow.i64(i64, i64)
declare { i64, i1 } @llvm.ssub.with.overflow.i64(i64, i64)

define internal i64 @primer_check_i32(i64 %value) {
entry:
  %below = icmp slt i64 %value, -2147483648
  %above = icmp sgt i64 %value, 2147483647
  %bad = or i1 %below, %above
  br i1 %bad, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret i64 %value
}

define internal i64 @primer_check_u32(i64 %value) {
entry:
  %below = icmp slt i64 %value, 0
  %above = icmp sgt i64 %value, 4294967295
  %bad = or i1 %below, %above
  br i1 %bad, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret i64 %value
}

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

define i64 @primer.fn.add.0(i64 %arg0, i64 %arg1) {
entry:
  %primer_left = alloca i64
  %primer_right = alloca i64
  store i64 %arg0, ptr %primer_left
  store i64 %arg1, ptr %primer_right
  %tmp0 = load i64, ptr %primer_left
  %tmp1 = load i64, ptr %primer_right
  %tmp2 = call i64 @primer_i64_add(i64 %tmp0, i64 %tmp1)
  %tmp3 = call i64 @primer_check_i32(i64 %tmp2)
  ret i64 %tmp3
}

define i32 @main() {
entry:
  %primer_small = alloca i64
  %primer_large = alloca i64
  %tmp0 = call i64 @primer_i64_sub(i64 0, i64 3)
  %tmp1 = call i64 @primer_check_i32(i64 %tmp0)
  %tmp2 = call i64 @primer.fn.add.0(i64 %tmp1, i64 5)
  store i64 %tmp2, ptr %primer_small
  store i64 4294967295, ptr %primer_large
  %tmp3 = load i64, ptr %primer_small
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp3)
  %tmp4 = load i64, ptr %primer_large
  %tmp5 = call i64 @primer_i64_div(i64 %tmp4, i64 2)
  %tmp6 = call i64 @primer_check_u32(i64 %tmp5)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp6)
  %tmp7 = load i64, ptr %primer_large
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp7)
  %tmp8 = load i64, ptr %primer_large
  %tmp9 = icmp sgt i64 %tmp8, 2147483648
  %tmp10 = select i1 %tmp9, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp10)
  %tmp11 = load i64, ptr %primer_small
  %tmp12 = call i64 @primer_check_u32(i64 %tmp11)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp12)
  ret i32 0
}
