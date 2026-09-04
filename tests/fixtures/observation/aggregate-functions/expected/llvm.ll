@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
%primer.type.Point.0 = type { i64, i64 }

declare i32 @printf(ptr, ...)
declare void @llvm.trap()

define internal i64 @primer.array.get.i64.2([2 x i64] %value, i64 %index) {
entry:
  %index.low = icmp slt i64 %index, 0
  %index.high = icmp sge i64 %index, 2
  %index.outside = or i1 %index.low, %index.high
  br i1 %index.outside, label %out_of_bounds, label %in_bounds
out_of_bounds:
  call void @llvm.trap()
  unreachable
in_bounds:
  %array = alloca [2 x i64]
  store [2 x i64] %value, ptr %array
  %element = getelementptr inbounds [2 x i64], ptr %array, i64 0, i64 %index
  %result = load i64, ptr %element
  ret i64 %result
}

define internal [2 x i64] @primer.array.get.array.i64.2.2([2 x [2 x i64]] %value, i64 %index) {
entry:
  %index.low = icmp slt i64 %index, 0
  %index.high = icmp sge i64 %index, 2
  %index.outside = or i1 %index.low, %index.high
  br i1 %index.outside, label %out_of_bounds, label %in_bounds
out_of_bounds:
  call void @llvm.trap()
  unreachable
in_bounds:
  %array = alloca [2 x [2 x i64]]
  store [2 x [2 x i64]] %value, ptr %array
  %element = getelementptr inbounds [2 x [2 x i64]], ptr %array, i64 0, i64 %index
  %result = load [2 x i64], ptr %element
  ret [2 x i64] %result
}

define %primer.type.Point.0 @primer.fn.move_x.0(%primer.type.Point.0 %arg0, i64 %arg1) {
entry:
  %primer_point = alloca %primer.type.Point.0
  %primer_amount = alloca i64
  store %primer.type.Point.0 %arg0, ptr %primer_point
  store i64 %arg1, ptr %primer_amount
  %tmp0 = load %primer.type.Point.0, ptr %primer_point
  %tmp1 = extractvalue %primer.type.Point.0 %tmp0, 0
  %tmp2 = load i64, ptr %primer_amount
  %tmp3 = add i64 %tmp1, %tmp2
  %tmp4 = insertvalue %primer.type.Point.0 poison, i64 %tmp3, 0
  %tmp5 = load %primer.type.Point.0, ptr %primer_point
  %tmp6 = extractvalue %primer.type.Point.0 %tmp5, 1
  %tmp7 = insertvalue %primer.type.Point.0 %tmp4, i64 %tmp6, 1
  ret %primer.type.Point.0 %tmp7
}

define %primer.type.Point.0 @primer.fn.move_twice.1(%primer.type.Point.0 %arg0, i64 %arg1) {
entry:
  %primer_point = alloca %primer.type.Point.0
  %primer_amount = alloca i64
  store %primer.type.Point.0 %arg0, ptr %primer_point
  store i64 %arg1, ptr %primer_amount
  %tmp0 = load %primer.type.Point.0, ptr %primer_point
  %tmp1 = load i64, ptr %primer_amount
  %tmp2 = call %primer.type.Point.0 @primer.fn.move_x.0(%primer.type.Point.0 %tmp0, i64 %tmp1)
  %tmp3 = load i64, ptr %primer_amount
  %tmp4 = call %primer.type.Point.0 @primer.fn.move_x.0(%primer.type.Point.0 %tmp2, i64 %tmp3)
  ret %primer.type.Point.0 %tmp4
}

define [2 x i64] @primer.fn.first_row.2([2 x [2 x i64]] %arg0) {
entry:
  %primer_matrix = alloca [2 x [2 x i64]]
  store [2 x [2 x i64]] %arg0, ptr %primer_matrix
  %tmp0 = load [2 x [2 x i64]], ptr %primer_matrix
  %tmp1 = call [2 x i64] @primer.array.get.array.i64.2.2([2 x [2 x i64]] %tmp0, i64 0)
  ret [2 x i64] %tmp1
}

define [2 x [2 x i64]] @primer.fn.duplicate.3([2 x i64] %arg0) {
entry:
  %primer_row = alloca [2 x i64]
  store [2 x i64] %arg0, ptr %primer_row
  %tmp0 = load [2 x i64], ptr %primer_row
  %tmp1 = insertvalue [2 x [2 x i64]] poison, [2 x i64] %tmp0, 0
  %tmp2 = load [2 x i64], ptr %primer_row
  %tmp3 = insertvalue [2 x [2 x i64]] %tmp1, [2 x i64] %tmp2, 1
  ret [2 x [2 x i64]] %tmp3
}

define [2 x [2 x i64]] @primer.fn.duplicate_first_row.4([2 x [2 x i64]] %arg0) {
entry:
  %primer_matrix = alloca [2 x [2 x i64]]
  store [2 x [2 x i64]] %arg0, ptr %primer_matrix
  %tmp0 = load [2 x [2 x i64]], ptr %primer_matrix
  %tmp1 = call [2 x i64] @primer.fn.first_row.2([2 x [2 x i64]] %tmp0)
  %tmp2 = call [2 x [2 x i64]] @primer.fn.duplicate.3([2 x i64] %tmp1)
  ret [2 x [2 x i64]] %tmp2
}

define i32 @main() {
entry:
  %primer_original = alloca %primer.type.Point.0
  %primer_moved = alloca %primer.type.Point.0
  %primer_matrix = alloca [2 x [2 x i64]]
  %primer_rows = alloca [2 x [2 x i64]]
  %tmp0 = insertvalue %primer.type.Point.0 poison, i64 2, 0
  %tmp1 = insertvalue %primer.type.Point.0 %tmp0, i64 3, 1
  store %primer.type.Point.0 %tmp1, ptr %primer_original
  %tmp2 = load %primer.type.Point.0, ptr %primer_original
  %tmp3 = call %primer.type.Point.0 @primer.fn.move_twice.1(%primer.type.Point.0 %tmp2, i64 5)
  store %primer.type.Point.0 %tmp3, ptr %primer_moved
  %tmp4 = insertvalue [2 x i64] poison, i64 1, 0
  %tmp5 = insertvalue [2 x i64] %tmp4, i64 2, 1
  %tmp6 = insertvalue [2 x [2 x i64]] poison, [2 x i64] %tmp5, 0
  %tmp7 = insertvalue [2 x i64] poison, i64 3, 0
  %tmp8 = insertvalue [2 x i64] %tmp7, i64 4, 1
  %tmp9 = insertvalue [2 x [2 x i64]] %tmp6, [2 x i64] %tmp8, 1
  store [2 x [2 x i64]] %tmp9, ptr %primer_matrix
  %tmp10 = load [2 x [2 x i64]], ptr %primer_matrix
  %tmp11 = call [2 x [2 x i64]] @primer.fn.duplicate_first_row.4([2 x [2 x i64]] %tmp10)
  store [2 x [2 x i64]] %tmp11, ptr %primer_rows
  %tmp12 = load %primer.type.Point.0, ptr %primer_original
  %tmp13 = extractvalue %primer.type.Point.0 %tmp12, 0
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp13)
  %tmp14 = load %primer.type.Point.0, ptr %primer_moved
  %tmp15 = extractvalue %primer.type.Point.0 %tmp14, 0
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp15)
  %tmp16 = load %primer.type.Point.0, ptr %primer_moved
  %tmp17 = extractvalue %primer.type.Point.0 %tmp16, 1
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp17)
  %tmp18 = load [2 x [2 x i64]], ptr %primer_matrix
  %tmp19 = call [2 x i64] @primer.array.get.array.i64.2.2([2 x [2 x i64]] %tmp18, i64 1)
  %tmp20 = call i64 @primer.array.get.i64.2([2 x i64] %tmp19, i64 0)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp20)
  %tmp21 = load [2 x [2 x i64]], ptr %primer_rows
  %tmp22 = call [2 x i64] @primer.array.get.array.i64.2.2([2 x [2 x i64]] %tmp21, i64 0)
  %tmp23 = call i64 @primer.array.get.i64.2([2 x i64] %tmp22, i64 1)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp23)
  %tmp24 = load [2 x [2 x i64]], ptr %primer_rows
  %tmp25 = call [2 x i64] @primer.array.get.array.i64.2.2([2 x [2 x i64]] %tmp24, i64 1)
  %tmp26 = call i64 @primer.array.get.i64.2([2 x i64] %tmp25, i64 0)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp26)
  ret i32 0
}
