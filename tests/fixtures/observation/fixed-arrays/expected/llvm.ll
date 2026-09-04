@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

declare i32 @printf(ptr, ...)
declare void @llvm.trap()

define internal i64 @primer.array.get.i64.3([3 x i64] %value, i64 %index) {
entry:
  %index.low = icmp slt i64 %index, 0
  %index.high = icmp sge i64 %index, 3
  %index.outside = or i1 %index.low, %index.high
  br i1 %index.outside, label %out_of_bounds, label %in_bounds
out_of_bounds:
  call void @llvm.trap()
  unreachable
in_bounds:
  %array = alloca [3 x i64]
  store [3 x i64] %value, ptr %array
  %element = getelementptr inbounds [3 x i64], ptr %array, i64 0, i64 %index
  %result = load i64, ptr %element
  ret i64 %result
}

define i32 @main() {
entry:
  %primer_values = alloca [3 x i64]
  %primer_copy = alloca [3 x i64]
  %tmp0 = insertvalue [3 x i64] poison, i64 2, 0
  %tmp1 = insertvalue [3 x i64] %tmp0, i64 4, 1
  %tmp2 = insertvalue [3 x i64] %tmp1, i64 6, 2
  store [3 x i64] %tmp2, ptr %primer_values
  %tmp3 = load [3 x i64], ptr %primer_values
  store [3 x i64] %tmp3, ptr %primer_copy
  %tmp4 = insertvalue [3 x i64] poison, i64 1, 0
  %tmp5 = insertvalue [3 x i64] %tmp4, i64 3, 1
  %tmp6 = insertvalue [3 x i64] %tmp5, i64 5, 2
  store [3 x i64] %tmp6, ptr %primer_values
  %tmp7 = load [3 x i64], ptr %primer_copy
  %tmp8 = call i64 @primer.array.get.i64.3([3 x i64] %tmp7, i64 2)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp8)
  %tmp9 = load [3 x i64], ptr %primer_values
  %tmp10 = call i64 @primer.array.get.i64.3([3 x i64] %tmp9, i64 1)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp10)
  ret i32 0
}
