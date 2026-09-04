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

define i32 @main() {
entry:
  %primer_count = alloca i64
  %primer_ratio = alloca float
  store i64 40, ptr %primer_count
  %tmp0 = load i64, ptr %primer_count
  %tmp1 = call i64 @primer_i64_add(i64 %tmp0, i64 2)
  store i64 %tmp1, ptr %primer_count
  store float 0x3FD0000000000000, ptr %primer_ratio
  %tmp2 = load float, ptr %primer_ratio
  %tmp3 = fmul float %tmp2, 0x4000000000000000
  store float %tmp3, ptr %primer_ratio
  %tmp4 = load i64, ptr %primer_count
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp4)
  %tmp5 = load float, ptr %primer_ratio
  %tmp6 = fpext float %tmp5 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp6)
  ret i32 0
}
