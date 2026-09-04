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

define internal [3 x i64] @primer.array.get.array.i64.3.2([2 x [3 x i64]] %value, i64 %index) {
entry:
  %index.low = icmp slt i64 %index, 0
  %index.high = icmp sge i64 %index, 2
  %index.outside = or i1 %index.low, %index.high
  br i1 %index.outside, label %out_of_bounds, label %in_bounds
out_of_bounds:
  call void @llvm.trap()
  unreachable
in_bounds:
  %array = alloca [2 x [3 x i64]]
  store [2 x [3 x i64]] %value, ptr %array
  %element = getelementptr inbounds [2 x [3 x i64]], ptr %array, i64 0, i64 %index
  %result = load [3 x i64], ptr %element
  ret [3 x i64] %result
}

define i32 @main() {
entry:
  %primer_matrix = alloca [2 x [3 x i64]]
  %primer_copy = alloca [2 x [3 x i64]]
  %tmp0 = insertvalue [3 x i64] poison, i64 1, 0
  %tmp1 = insertvalue [3 x i64] %tmp0, i64 2, 1
  %tmp2 = insertvalue [3 x i64] %tmp1, i64 3, 2
  %tmp3 = insertvalue [2 x [3 x i64]] poison, [3 x i64] %tmp2, 0
  %tmp4 = insertvalue [3 x i64] poison, i64 4, 0
  %tmp5 = insertvalue [3 x i64] %tmp4, i64 5, 1
  %tmp6 = insertvalue [3 x i64] %tmp5, i64 6, 2
  %tmp7 = insertvalue [2 x [3 x i64]] %tmp3, [3 x i64] %tmp6, 1
  store [2 x [3 x i64]] %tmp7, ptr %primer_matrix
  %tmp8 = load [2 x [3 x i64]], ptr %primer_matrix
  store [2 x [3 x i64]] %tmp8, ptr %primer_copy
  %tmp9 = insertvalue [3 x i64] poison, i64 7, 0
  %tmp10 = insertvalue [3 x i64] %tmp9, i64 8, 1
  %tmp11 = insertvalue [3 x i64] %tmp10, i64 9, 2
  %tmp12 = insertvalue [2 x [3 x i64]] poison, [3 x i64] %tmp11, 0
  %tmp13 = insertvalue [3 x i64] poison, i64 10, 0
  %tmp14 = insertvalue [3 x i64] %tmp13, i64 11, 1
  %tmp15 = insertvalue [3 x i64] %tmp14, i64 12, 2
  %tmp16 = insertvalue [2 x [3 x i64]] %tmp12, [3 x i64] %tmp15, 1
  store [2 x [3 x i64]] %tmp16, ptr %primer_matrix
  %tmp17 = load [2 x [3 x i64]], ptr %primer_copy
  %tmp18 = call [3 x i64] @primer.array.get.array.i64.3.2([2 x [3 x i64]] %tmp17, i64 1)
  %tmp19 = call i64 @primer.array.get.i64.3([3 x i64] %tmp18, i64 2)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp19)
  %tmp20 = load [2 x [3 x i64]], ptr %primer_matrix
  %tmp21 = call [3 x i64] @primer.array.get.array.i64.3.2([2 x [3 x i64]] %tmp20, i64 0)
  %tmp22 = call i64 @primer.array.get.i64.3([3 x i64] %tmp21, i64 1)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp22)
  ret i32 0
}
