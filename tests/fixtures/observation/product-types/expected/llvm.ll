@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
%primer.type.Point.0 = type { double, double }
%primer.type.Segment.1 = type { %primer.type.Point.0, %primer.type.Point.0 }

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %primer_current = alloca %primer.type.Point.0
  %primer_saved = alloca %primer.type.Point.0
  %primer_segment = alloca %primer.type.Segment.1
  %tmp0 = insertvalue %primer.type.Point.0 poison, double 0x4000000000000000, 1
  %tmp1 = insertvalue %primer.type.Point.0 %tmp0, double 0x0000000000000000, 0
  store %primer.type.Point.0 %tmp1, ptr %primer_current
  %tmp2 = load %primer.type.Point.0, ptr %primer_current
  store %primer.type.Point.0 %tmp2, ptr %primer_saved
  %tmp3 = insertvalue %primer.type.Point.0 poison, double 0x4010000000000000, 0
  %tmp4 = insertvalue %primer.type.Point.0 %tmp3, double 0x4014000000000000, 1
  store %primer.type.Point.0 %tmp4, ptr %primer_current
  %tmp5 = load %primer.type.Point.0, ptr %primer_saved
  %tmp6 = insertvalue %primer.type.Segment.1 poison, %primer.type.Point.0 %tmp5, 0
  %tmp7 = load %primer.type.Point.0, ptr %primer_current
  %tmp8 = insertvalue %primer.type.Segment.1 %tmp6, %primer.type.Point.0 %tmp7, 1
  store %primer.type.Segment.1 %tmp8, ptr %primer_segment
  %tmp9 = load %primer.type.Point.0, ptr %primer_saved
  %tmp10 = extractvalue %primer.type.Point.0 %tmp9, 0
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double %tmp10)
  %tmp11 = load %primer.type.Point.0, ptr %primer_saved
  %tmp12 = extractvalue %primer.type.Point.0 %tmp11, 1
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double %tmp12)
  %tmp13 = load %primer.type.Segment.1, ptr %primer_segment
  %tmp14 = extractvalue %primer.type.Segment.1 %tmp13, 0
  %tmp15 = extractvalue %primer.type.Point.0 %tmp14, 1
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double %tmp15)
  %tmp16 = load %primer.type.Segment.1, ptr %primer_segment
  %tmp17 = extractvalue %primer.type.Segment.1 %tmp16, 1
  %tmp18 = extractvalue %primer.type.Point.0 %tmp17, 0
  call i32 (ptr, ...) @printf(ptr @.fmt_f64, double %tmp18)
  ret i32 0
}
