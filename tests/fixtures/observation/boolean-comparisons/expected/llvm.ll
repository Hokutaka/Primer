@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
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

define i32 @main() {
entry:
  %primer_truth = alloca i1
  %primer_negated = alloca i1
  %primer_same = alloca i1
  %primer_integer_order = alloca i1
  %primer_float_difference = alloca i1
  store i1 1, ptr %primer_truth
  %tmp0 = load i1, ptr %primer_truth
  %tmp1 = xor i1 %tmp0, 1
  store i1 %tmp1, ptr %primer_negated
  %tmp2 = load i1, ptr %primer_truth
  %tmp3 = icmp eq i1 %tmp2, 1
  store i1 %tmp3, ptr %primer_same
  %tmp4 = call i64 @primer_i64_add(i64 1, i64 2)
  %tmp5 = icmp slt i64 %tmp4, 4
  store i1 %tmp5, ptr %primer_integer_order
  %tmp6 = fcmp une float 0x3FB99999A0000000, 0x3FC99999A0000000
  store i1 %tmp6, ptr %primer_float_difference
  %tmp7 = load i1, ptr %primer_truth
  %tmp8 = select i1 %tmp7, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp8)
  %tmp9 = load i1, ptr %primer_negated
  %tmp10 = select i1 %tmp9, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp10)
  %tmp11 = load i1, ptr %primer_same
  %tmp12 = select i1 %tmp11, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp12)
  %tmp13 = load i1, ptr %primer_integer_order
  %tmp14 = select i1 %tmp13, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp14)
  %tmp15 = load i1, ptr %primer_float_difference
  %tmp16 = select i1 %tmp15, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp16)
  ret i32 0
}
