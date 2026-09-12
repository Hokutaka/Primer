target triple = "x86_64-unknown-linux-gnu"

@primer.failure.20.204.211.11 = private unnamed_addr constant [73 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\61\72\72\61\79\2D\69\6E\64\65\78\2D\6F\75\74\2D\6F\66\2D\62\6F\75\6E\64\73\20\6E\6F\64\65\3D\32\30\20\62\79\74\65\73\3D\32\30\34\2E\2E\32\31\31\0A"
@primer.failure.20.204.211 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.20.204.211.11, i64 73 }]
@primer.failure.25.222.231.11 = private unnamed_addr constant [73 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\61\72\72\61\79\2D\69\6E\64\65\78\2D\6F\75\74\2D\6F\66\2D\62\6F\75\6E\64\73\20\6E\6F\64\65\3D\32\35\20\62\79\74\65\73\3D\32\32\32\2E\2E\32\33\31\0A"
@primer.failure.25.222.231 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.25.222.231.11, i64 73 }]
@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
%primer.type.Point.0 = type { i64, i64 }

declare i32 @printf(ptr, ...)
declare void @llvm.trap()

declare i32 @fflush(ptr)
declare i64 @write(i32, ptr, i64)

define internal void @primer.runtime.fail(ptr %failure, i64 %code) {
entry:
  call i32 @fflush(ptr null)
  %slot = getelementptr inbounds { ptr, i64 }, ptr %failure, i64 %code
  %record = load { ptr, i64 }, ptr %slot
  %data = extractvalue { ptr, i64 } %record, 0
  %length = extractvalue { ptr, i64 } %record, 1
  call i64 @write(i32 2, ptr %data, i64 %length)
  call void @llvm.trap()
  unreachable
}

define internal %primer.type.Point.0 @primer.array.get.type.Point.0.2([2 x %primer.type.Point.0] %value, i64 %index, ptr %failure) {
entry:
  %index.low = icmp slt i64 %index, 0
  %index.high = icmp sge i64 %index, 2
  %index.outside = or i1 %index.low, %index.high
  br i1 %index.outside, label %out_of_bounds, label %in_bounds
out_of_bounds:
  call void @primer.runtime.fail(ptr %failure, i64 11)
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
  %tmp14 = call %primer.type.Point.0 @primer.array.get.type.Point.0.2([2 x %primer.type.Point.0] %tmp13, i64 1, ptr @primer.failure.20.204.211)
  %tmp15 = extractvalue %primer.type.Point.0 %tmp14, 0
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp15)
  %tmp16 = load [2 x %primer.type.Point.0], ptr %primer_points
  %tmp17 = call %primer.type.Point.0 @primer.array.get.type.Point.0.2([2 x %primer.type.Point.0] %tmp16, i64 0, ptr @primer.failure.25.222.231)
  %tmp18 = extractvalue %primer.type.Point.0 %tmp17, 1
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp18)
  ret i32 0
}
