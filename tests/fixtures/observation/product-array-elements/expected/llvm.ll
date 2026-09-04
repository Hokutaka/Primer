@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
%primer.type.Point.0 = type { i64, i64 }

declare i32 @printf(ptr, ...)
declare void @llvm.trap()

define internal %primer.type.Point.0 @primer.array.get.type.Point.0.2([2 x %primer.type.Point.0] %value, i64 %index) {
entry:
  %index.low = icmp slt i64 %index, 0
  %index.high = icmp sge i64 %index, 2
  %index.outside = or i1 %index.low, %index.high
  br i1 %index.outside, label %out_of_bounds, label %in_bounds
out_of_bounds:
  call void @llvm.trap()
  unreachable
in_bounds:
  %array = alloca [2 x %primer.type.Point.0]
  store [2 x %primer.type.Point.0] %value, ptr %array
  %element = getelementptr inbounds [2 x %primer.type.Point.0], ptr %array, i64 0, i64 %index
  %result = load %primer.type.Point.0, ptr %element
  ret %primer.type.Point.0 %result
}

define i32 @main() {
entry:
  %primer_points = alloca [2 x %primer.type.Point.0]
  %primer_copy = alloca [2 x %primer.type.Point.0]
  %tmp0 = insertvalue %primer.type.Point.0 poison, i64 1, 0
  %tmp1 = insertvalue %primer.type.Point.0 %tmp0, i64 2, 1
  %tmp2 = insertvalue [2 x %primer.type.Point.0] poison, %primer.type.Point.0 %tmp1, 0
  %tmp3 = insertvalue %primer.type.Point.0 poison, i64 3, 0
  %tmp4 = insertvalue %primer.type.Point.0 %tmp3, i64 4, 1
  %tmp5 = insertvalue [2 x %primer.type.Point.0] %tmp2, %primer.type.Point.0 %tmp4, 1
  store [2 x %primer.type.Point.0] %tmp5, ptr %primer_points
  %tmp6 = load [2 x %primer.type.Point.0], ptr %primer_points
  store [2 x %primer.type.Point.0] %tmp6, ptr %primer_copy
  %tmp7 = insertvalue %primer.type.Point.0 poison, i64 5, 0
  %tmp8 = insertvalue %primer.type.Point.0 %tmp7, i64 6, 1
  %tmp9 = insertvalue [2 x %primer.type.Point.0] poison, %primer.type.Point.0 %tmp8, 0
  %tmp10 = insertvalue %primer.type.Point.0 poison, i64 7, 0
  %tmp11 = insertvalue %primer.type.Point.0 %tmp10, i64 8, 1
  %tmp12 = insertvalue [2 x %primer.type.Point.0] %tmp9, %primer.type.Point.0 %tmp11, 1
  store [2 x %primer.type.Point.0] %tmp12, ptr %primer_points
  %tmp13 = load [2 x %primer.type.Point.0], ptr %primer_copy
  %tmp14 = call %primer.type.Point.0 @primer.array.get.type.Point.0.2([2 x %primer.type.Point.0] %tmp13, i64 1)
  %tmp15 = extractvalue %primer.type.Point.0 %tmp14, 0
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp15)
  %tmp16 = load [2 x %primer.type.Point.0], ptr %primer_points
  %tmp17 = call %primer.type.Point.0 @primer.array.get.type.Point.0.2([2 x %primer.type.Point.0] %tmp16, i64 0)
  %tmp18 = extractvalue %primer.type.Point.0 %tmp17, 1
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp18)
  ret i32 0
}
