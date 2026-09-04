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
  %primer_value = alloca i64
  %primer_sum = alloca i64
  store i64 0, ptr %primer_value
  store i64 0, ptr %primer_sum
  br label %block0
block0: ; while_condition
  %tmp0 = load i64, ptr %primer_value
  %tmp1 = icmp slt i64 %tmp0, 10
  br i1 %tmp1, label %block1, label %block2
block1: ; while_body
  %tmp2 = load i64, ptr %primer_value
  %tmp3 = call i64 @primer_i64_add(i64 %tmp2, i64 1)
  store i64 %tmp3, ptr %primer_value
  %tmp4 = load i64, ptr %primer_value
  %tmp5 = icmp slt i64 %tmp4, 3
  br i1 %tmp5, label %block3, label %block5
block3: ; if_then
  br label %block0
block5: ; if_end
  %tmp6 = load i64, ptr %primer_value
  %tmp7 = icmp sgt i64 %tmp6, 5
  br i1 %tmp7, label %block6, label %block8
block6: ; if_then
  br label %block2
block8: ; if_end
  %tmp8 = load i64, ptr %primer_sum
  %tmp9 = load i64, ptr %primer_value
  %tmp10 = call i64 @primer_i64_add(i64 %tmp8, i64 %tmp9)
  store i64 %tmp10, ptr %primer_sum
  br label %block0
block2: ; while_end
  %tmp11 = load i64, ptr %primer_sum
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp11)
  %tmp12 = load i64, ptr %primer_value
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp12)
  ret i32 0
}
