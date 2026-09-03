@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %primer_sum = alloca i64
  %primer_i = alloca i64
  store i64 0, ptr %primer_sum
  store i64 0, ptr %primer_i
  br label %block0
block0: ; for_condition
  %tmp0 = load i64, ptr %primer_i
  %tmp1 = icmp slt i64 %tmp0, 6
  br i1 %tmp1, label %block1, label %block3
block1: ; for_body
  %tmp2 = load i64, ptr %primer_i
  %tmp3 = icmp slt i64 %tmp2, 2
  br i1 %tmp3, label %block4, label %block6
block4: ; if_then
  br label %block2
block6: ; if_end
  %tmp4 = load i64, ptr %primer_sum
  %tmp5 = load i64, ptr %primer_i
  %tmp6 = add i64 %tmp4, %tmp5
  store i64 %tmp6, ptr %primer_sum
  br label %block2
block2: ; for_update
  %tmp7 = load i64, ptr %primer_i
  %tmp8 = add i64 %tmp7, 1
  store i64 %tmp8, ptr %primer_i
  br label %block0
block3: ; for_end
  %tmp9 = load i64, ptr %primer_sum
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp9)
  ret i32 0
}
