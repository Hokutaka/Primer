@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)

define i32 @main() {
entry:
  %tmp0 = fpext float 0x3BC79CA100000000 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp0)
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double 0x3BC79CA10C924223)
  %tmp1 = fcmp une double 0x3BC79CA10C924223, 0x0000000000000000
  %tmp2 = select i1 %tmp1, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp2)
  %tmp3 = fpext float 0x36A0000000000000 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp3)
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double 0x0000000000000001)
  %tmp4 = fpext float 0x47EFFFFFE0000000 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp4)
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double 0x7FEFFFFFFFFFFFFF)
  %tmp5 = fneg float 0x0000000000000000
  %tmp6 = fpext float %tmp5 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp6)
  %tmp7 = fneg double 0x0000000000000000
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double %tmp7)
  %tmp8 = fpext float 0x0000000000000000 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp8)
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double 0x0000000000000000)
  %tmp9 = fpext float 0x3F1A36E2E0000000 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp9)
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double 0x3F1A36E2EB1C432D)
  %tmp10 = fpext float 0x41CDCD6500000000 to double
  call i32 (ptr, ...) @printf(ptr @.fmt_f32, double %tmp10)
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double 0x4376345785D8A000)
  ret i32 0
}
