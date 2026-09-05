@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
declare void @llvm.trap()
declare { i64, i1 } @llvm.ssub.with.overflow.i64(i64, i64)

define internal i64 @primer_i64_rem(i64 %left, i64 %right) {
entry:
  %zero = icmp eq i64 %right, 0
  br i1 %zero, label %trap, label %special
special:
  %negative_one = icmp eq i64 %right, -1
  br i1 %negative_one, label %zero_result, label %ok
zero_result:
  ret i64 0
trap:
  call void @llvm.trap()
  unreachable
ok:
  %result = srem i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_u8_bit_and(i64 %left, i64 %right) {
entry:
  %result = and i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_u8_bit_or(i64 %left, i64 %right) {
entry:
  %result = or i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_u8_bit_xor(i64 %left, i64 %right) {
entry:
  %result = xor i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_u8_shl(i64 %left, i64 %right) {
entry:
  %negative = icmp slt i64 %right, 0
  %wide = icmp sge i64 %right, 8
  %bad_count = or i1 %negative, %wide
  br i1 %bad_count, label %trap, label %bounds
trap:
  call void @llvm.trap()
  unreachable
bounds:
  %minimum = ashr i64 0, %right
  %maximum = lshr i64 255, %right
  %below = icmp slt i64 %left, %minimum
  %above = icmp sgt i64 %left, %maximum
  %overflow = or i1 %below, %above
  br i1 %overflow, label %trap, label %ok
ok:
  %result = shl i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_i8_shr(i64 %left, i64 %right) {
entry:
  %negative = icmp slt i64 %right, 0
  %wide = icmp sge i64 %right, 8
  %bad_count = or i1 %negative, %wide
  br i1 %bad_count, label %trap, label %bounds
trap:
  call void @llvm.trap()
  unreachable
bounds:
  %result = ashr i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_u8_shr(i64 %left, i64 %right) {
entry:
  %negative = icmp slt i64 %right, 0
  %wide = icmp sge i64 %right, 8
  %bad_count = or i1 %negative, %wide
  br i1 %bad_count, label %trap, label %bounds
trap:
  call void @llvm.trap()
  unreachable
bounds:
  %result = ashr i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_check_i8(i64 %value) {
entry:
  %below = icmp slt i64 %value, -128
  %above = icmp sgt i64 %value, 127
  %bad = or i1 %below, %above
  br i1 %bad, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret i64 %value
}

define internal i64 @primer_check_u8(i64 %value) {
entry:
  %below = icmp slt i64 %value, 0
  %above = icmp sgt i64 %value, 255
  %bad = or i1 %below, %above
  br i1 %bad, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret i64 %value
}

define internal i64 @primer_i64_sub(i64 %left, i64 %right) {
entry:
  %checked = call { i64, i1 } @llvm.ssub.with.overflow.i64(i64 %left, i64 %right)
  %result = extractvalue { i64, i1 } %checked, 0
  %overflow = extractvalue { i64, i1 } %checked, 1
  br i1 %overflow, label %trap, label %ok

trap:
  call void @llvm.trap()
  unreachable

ok:
  ret i64 %result
}

define i64 @primer.fn.mark.0(i64 %arg0) {
entry:
  %primer_value = alloca i64
  store i64 %arg0, ptr %primer_value
  %tmp0 = load i64, ptr %primer_value
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp0)
  %tmp1 = load i64, ptr %primer_value
  ret i64 %tmp1
}

define i32 @main() {
entry:
  %primer_bits = alloca i64
  %primer_logical_result1 = alloca i1
  %tmp0 = call i64 @primer_u8_shl(i64 1, i64 7)
  %tmp1 = call i64 @primer_check_u8(i64 %tmp0)
  store i64 %tmp1, ptr %primer_bits
  %tmp2 = load i64, ptr %primer_bits
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp2)
  %tmp3 = load i64, ptr %primer_bits
  %tmp4 = call i64 @primer_u8_shr(i64 %tmp3, i64 7)
  %tmp5 = call i64 @primer_check_u8(i64 %tmp4)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp5)
  %tmp6 = call i64 @primer_u8_bit_xor(i64 0, i64 255)
  %tmp7 = call i64 @primer_check_u8(i64 %tmp6)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp7)
  %tmp8 = call i64 @primer.fn.mark.0(i64 1)
  %tmp9 = call i64 @primer.fn.mark.0(i64 2)
  %tmp10 = call i64 @primer.fn.mark.0(i64 3)
  %tmp11 = call i64 @primer_u8_bit_xor(i64 %tmp9, i64 %tmp10)
  %tmp12 = call i64 @primer_check_u8(i64 %tmp11)
  %tmp13 = call i64 @primer_u8_bit_or(i64 %tmp8, i64 %tmp12)
  %tmp14 = call i64 @primer_check_u8(i64 %tmp13)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp14)
  %tmp15 = load i64, ptr %primer_bits
  %tmp16 = call i64 @primer_u8_bit_and(i64 %tmp15, i64 127)
  %tmp17 = call i64 @primer_check_u8(i64 %tmp16)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp17)
  %tmp18 = call i64 @primer_i64_sub(i64 0, i64 7)
  %tmp19 = call i64 @primer_i64_rem(i64 %tmp18, i64 3)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp19)
  %tmp20 = call i64 @primer_i64_sub(i64 0, i64 1)
  %tmp21 = call i64 @primer_i64_rem(i64 -9223372036854775808, i64 %tmp20)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp21)
  %tmp22 = call i64 @primer_i64_sub(i64 0, i64 3)
  %tmp23 = call i64 @primer_check_i8(i64 %tmp22)
  %tmp24 = call i64 @primer_i8_shr(i64 %tmp23, i64 1)
  %tmp25 = call i64 @primer_check_i8(i64 %tmp24)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp25)
  store i1 0, ptr %primer_logical_result1
  br i1 0, label %block0, label %block1
block0: ; logical_rhs
  %tmp26 = load i64, ptr %primer_bits
  %tmp27 = call i64 @primer_u8_shl(i64 %tmp26, i64 1)
  %tmp28 = call i64 @primer_check_u8(i64 %tmp27)
  %tmp29 = icmp eq i64 %tmp28, 0
  store i1 %tmp29, ptr %primer_logical_result1
  br label %block1
block1: ; logical_end
  %tmp30 = load i1, ptr %primer_logical_result1
  %tmp31 = select i1 %tmp30, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp31)
  ret i32 0
}
