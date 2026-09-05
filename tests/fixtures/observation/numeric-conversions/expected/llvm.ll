@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

declare i32 @printf(ptr, ...)
declare void @llvm.trap()

define internal double @primer_convert_i16_f64(i64 %value) {
entry:
  %result = sitofp i64 %value to double
  %below = fcmp olt double %result, 0xC3E0000000000000
  %above = fcmp oge double %result, 0x43E0000000000000
  %outside = or i1 %below, %above
  br i1 %outside, label %trap, label %convert
convert:
  %back = fptosi double %result to i64
  %changed = icmp ne i64 %back, %value
  br i1 %changed, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret double %result
}

define internal float @primer_convert_u32_f32(i64 %value) {
entry:
  %result = sitofp i64 %value to float
  %number = fpext float %result to double
  %below = fcmp olt double %number, 0xC3E0000000000000
  %above = fcmp oge double %number, 0x43E0000000000000
  %outside = or i1 %below, %above
  br i1 %outside, label %trap, label %convert
convert:
  %back = fptosi double %number to i64
  %changed = icmp ne i64 %back, %value
  br i1 %changed, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret float %result
}

define internal double @primer_convert_u32_f64(i64 %value) {
entry:
  %result = sitofp i64 %value to double
  %below = fcmp olt double %result, 0xC3E0000000000000
  %above = fcmp oge double %result, 0x43E0000000000000
  %outside = or i1 %below, %above
  br i1 %outside, label %trap, label %convert
convert:
  %back = fptosi double %result to i64
  %changed = icmp ne i64 %back, %value
  br i1 %changed, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret double %result
}

define internal double @primer_convert_i64_f64(i64 %value) {
entry:
  %result = sitofp i64 %value to double
  %below = fcmp olt double %result, 0xC3E0000000000000
  %above = fcmp oge double %result, 0x43E0000000000000
  %outside = or i1 %below, %above
  br i1 %outside, label %trap, label %convert
convert:
  %back = fptosi double %result to i64
  %changed = icmp ne i64 %back, %value
  br i1 %changed, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret double %result
}

define internal i64 @primer_convert_f32_i16(float %value) {
entry:
  %number = fpext float %value to double
  %below = fcmp ult double %number, 0xC0E0000000000000
  %above = fcmp uge double %number, 0x40E0000000000000
  %outside = or i1 %below, %above
  %bits = bitcast double %number to i64
  %negative_zero = icmp eq i64 %bits, -9223372036854775808
  %bad = or i1 %outside, %negative_zero
  br i1 %bad, label %trap, label %convert
convert:
  %result = fptosi double %number to i64
  %back = sitofp i64 %result to double
  %changed = fcmp one double %back, %number
  br i1 %changed, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret i64 %result
}

define internal double @primer_convert_f32_f64(float %value) {
entry:
  %nan = fcmp uno float %value, %value
  br i1 %nan, label %trap, label %convert
convert:
  %result = fpext float %value to double
  br label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret double %result
}

define internal i64 @primer_convert_f64_i64(double %value) {
entry:
  %below = fcmp ult double %value, 0xC3E0000000000000
  %above = fcmp uge double %value, 0x43E0000000000000
  %outside = or i1 %below, %above
  %bits = bitcast double %value to i64
  %negative_zero = icmp eq i64 %bits, -9223372036854775808
  %bad = or i1 %outside, %negative_zero
  br i1 %bad, label %trap, label %convert
convert:
  %result = fptosi double %value to i64
  %back = sitofp i64 %result to double
  %changed = fcmp one double %back, %value
  br i1 %changed, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret i64 %result
}

define internal float @primer_convert_f64_f32(double %value) {
entry:
  %nan = fcmp uno double %value, %value
  br i1 %nan, label %trap, label %convert
convert:
  %result = fptrunc double %value to float
  %back = fpext float %result to double
  %changed = fcmp one double %back, %value
  br i1 %changed, label %trap, label %ok
trap:
  call void @llvm.trap()
  unreachable
ok:
  ret float %result
}

define double @primer.fn.measure.0(i64 %arg0) {
entry:
  %primer_value = alloca i64
  store i64 %arg0, ptr %primer_value
  %tmp0 = load i64, ptr %primer_value
  %tmp1 = call double @primer_convert_i16_f64(i64 %tmp0)
  %tmp2 = call double @primer_convert_i64_f64(i64 2)
  %tmp3 = fdiv double %tmp1, %tmp2
  ret double %tmp3
}

define i32 @main() {
entry:
  %primer_count = alloca i64
  %primer_wide = alloca double
  %primer_narrow = alloca float
  store i64 42, ptr %primer_count
  %tmp0 = load i64, ptr %primer_count
  %tmp1 = call double @primer_convert_u32_f64(i64 %tmp0)
  store double %tmp1, ptr %primer_wide
  %tmp2 = load double, ptr %primer_wide
  %tmp3 = call float @primer_convert_f64_f32(double %tmp2)
  store float %tmp3, ptr %primer_narrow
  %tmp4 = load float, ptr %primer_narrow
  %tmp5 = call i64 @primer_convert_f32_i16(float %tmp4)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp5)
  %tmp6 = load double, ptr %primer_wide
  %tmp7 = call i64 @primer_convert_f64_i64(double %tmp6)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp7)
  %tmp8 = load float, ptr %primer_narrow
  %tmp9 = call double @primer_convert_f32_f64(float %tmp8)
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double %tmp9)
  %tmp10 = load i64, ptr %primer_count
  %tmp11 = call float @primer_convert_u32_f32(i64 %tmp10)
  %tmp12 = fpext float %tmp11 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp12)
  %tmp13 = call double @primer.fn.measure.0(i64 3)
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double %tmp13)
  %tmp14 = fneg double 0x0000000000000000
  %tmp15 = call float @primer_convert_f64_f32(double %tmp14)
  %tmp16 = fpext float %tmp15 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp16)
  ret i32 0
}
