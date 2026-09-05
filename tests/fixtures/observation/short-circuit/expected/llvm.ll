@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
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

define i1 @primer.fn.report.0(i1 %arg0) {
entry:
  %primer_value = alloca i1
  store i1 %arg0, ptr %primer_value
  %tmp0 = load i1, ptr %primer_value
  %tmp1 = select i1 %tmp0, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp1)
  %tmp2 = load i1, ptr %primer_value
  ret i1 %tmp2
}

define i32 @main() {
entry:
  %primer_values = alloca [2 x i64]
  %primer_index = alloca i64
  %primer_logical_result2 = alloca i1
  %primer_logical_result3 = alloca i1
  %primer_logical_result4 = alloca i1
  %primer_logical_result5 = alloca i1
  %primer_logical_result6 = alloca i1
  %tmp0 = insertvalue [2 x i64] poison, i64 4, 0
  %tmp1 = insertvalue [2 x i64] %tmp0, i64 9, 1
  store [2 x i64] %tmp1, ptr %primer_values
  store i64 2, ptr %primer_index
  %tmp2 = load i64, ptr %primer_index
  %tmp3 = icmp slt i64 %tmp2, 2
  store i1 %tmp3, ptr %primer_logical_result2
  br i1 %tmp3, label %block0, label %block1
block0: ; logical_rhs
  %tmp4 = load [2 x i64], ptr %primer_values
  %tmp5 = load i64, ptr %primer_index
  %tmp6 = call i64 @primer.array.get.i64.2([2 x i64] %tmp4, i64 %tmp5)
  %tmp7 = icmp sgt i64 %tmp6, 0
  store i1 %tmp7, ptr %primer_logical_result2
  br label %block1
block1: ; logical_end
  %tmp8 = load i1, ptr %primer_logical_result2
  %tmp9 = select i1 %tmp8, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp9)
  %tmp10 = load i64, ptr %primer_index
  %tmp11 = icmp eq i64 %tmp10, 2
  store i1 %tmp11, ptr %primer_logical_result3
  br i1 %tmp11, label %block3, label %block2
block2: ; logical_rhs
  %tmp12 = call i1 @primer.fn.report.0(i1 0)
  store i1 %tmp12, ptr %primer_logical_result3
  br label %block3
block3: ; logical_end
  %tmp13 = load i1, ptr %primer_logical_result3
  %tmp14 = select i1 %tmp13, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp14)
  store i1 0, ptr %primer_logical_result4
  br i1 0, label %block5, label %block4
block4: ; logical_rhs
  %tmp15 = call i1 @primer.fn.report.0(i1 1)
  store i1 %tmp15, ptr %primer_logical_result5
  br i1 %tmp15, label %block6, label %block7
block6: ; logical_rhs
  %tmp16 = load i64, ptr %primer_index
  %tmp17 = icmp sgt i64 %tmp16, 0
  store i1 %tmp17, ptr %primer_logical_result6
  br i1 %tmp17, label %block9, label %block8
block8: ; logical_rhs
  %tmp18 = call i1 @primer.fn.report.0(i1 0)
  store i1 %tmp18, ptr %primer_logical_result6
  br label %block9
block9: ; logical_end
  %tmp19 = load i1, ptr %primer_logical_result6
  store i1 %tmp19, ptr %primer_logical_result5
  br label %block7
block7: ; logical_end
  %tmp20 = load i1, ptr %primer_logical_result5
  store i1 %tmp20, ptr %primer_logical_result4
  br label %block5
block5: ; logical_end
  %tmp21 = load i1, ptr %primer_logical_result4
  %tmp22 = select i1 %tmp21, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp22)
  ret i32 0
}
