@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)

define i32 @main() {
entry:
  %primer_count = alloca i64
  %primer_sum = alloca i64
  %primer_marker = alloca i1
  store i64 0, ptr %primer_count
  store i64 0, ptr %primer_sum
  br label %block0
block0: ; while_condition
  %tmp0 = load i64, ptr %primer_count
  %tmp1 = icmp slt i64 %tmp0, 4
  br i1 %tmp1, label %block1, label %block2
block1: ; while_body
  %tmp2 = load i64, ptr %primer_sum
  %tmp3 = load i64, ptr %primer_count
  %tmp4 = add i64 %tmp2, %tmp3
  store i64 %tmp4, ptr %primer_sum
  %tmp5 = load i64, ptr %primer_count
  %tmp6 = icmp eq i64 %tmp5, 2
  br i1 %tmp6, label %block3, label %block5
block3: ; if_then
  store i1 1, ptr %primer_marker
  %tmp7 = load i1, ptr %primer_marker
  %tmp8 = select i1 %tmp7, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp8)
  br label %block5
block5: ; if_end
  %tmp9 = load i64, ptr %primer_count
  %tmp10 = add i64 %tmp9, 1
  store i64 %tmp10, ptr %primer_count
  br label %block0
block2: ; while_end
  %tmp11 = load i64, ptr %primer_sum
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp11)
  ret i32 0
}
