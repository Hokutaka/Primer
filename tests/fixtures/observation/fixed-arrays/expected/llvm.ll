target triple = "x86_64-unknown-linux-gnu"

@primer.failure.13.85.92.11 = private unnamed_addr constant [71 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\61\72\72\61\79\2D\69\6E\64\65\78\2D\6F\75\74\2D\6F\66\2D\62\6F\75\6E\64\73\20\6E\6F\64\65\3D\31\33\20\62\79\74\65\73\3D\38\35\2E\2E\39\32\0A"
@primer.failure.13.85.92 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.13.85.92.11, i64 71 }]
@primer.failure.17.101.110.11 = private unnamed_addr constant [73 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\61\72\72\61\79\2D\69\6E\64\65\78\2D\6F\75\74\2D\6F\66\2D\62\6F\75\6E\64\73\20\6E\6F\64\65\3D\31\37\20\62\79\74\65\73\3D\31\30\31\2E\2E\31\31\30\0A"
@primer.failure.17.101.110 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.17.101.110.11, i64 73 }]
@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

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

define internal i64 @primer.array.get.i64.3([3 x i64] %value, i64 %index, ptr %failure) {
entry:
  %index.low = icmp slt i64 %index, 0
  %index.high = icmp sge i64 %index, 3
  %index.outside = or i1 %index.low, %index.high
  br i1 %index.outside, label %out_of_bounds, label %in_bounds
out_of_bounds:
  call void @primer.runtime.fail(ptr %failure, i64 11)
  unreachable
in_bounds:
  %array = alloca [3 x i64]
  store [3 x i64] %value, ptr %array
  %element = getelementptr inbounds [3 x i64], ptr %array, i64 0, i64 %index
  %result = load i64, ptr %element
  ret i64 %result
}

define i32 @main() {
entry:
  %primer_values = alloca [3 x i64]
  %primer_copy = alloca [3 x i64]
  %tmp0 = insertvalue [3 x i64] poison, i64 2, 0
  %tmp1 = insertvalue [3 x i64] %tmp0, i64 4, 1
  %tmp2 = insertvalue [3 x i64] %tmp1, i64 6, 2
  store [3 x i64] %tmp2, ptr %primer_values
  %tmp3 = load [3 x i64], ptr %primer_values
  store [3 x i64] %tmp3, ptr %primer_copy
  %tmp4 = insertvalue [3 x i64] poison, i64 1, 0
  %tmp5 = insertvalue [3 x i64] %tmp4, i64 3, 1
  %tmp6 = insertvalue [3 x i64] %tmp5, i64 5, 2
  store [3 x i64] %tmp6, ptr %primer_values
  %tmp7 = load [3 x i64], ptr %primer_copy
  %tmp8 = call i64 @primer.array.get.i64.3([3 x i64] %tmp7, i64 2, ptr @primer.failure.13.85.92)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp8)
  %tmp9 = load [3 x i64], ptr %primer_values
  %tmp10 = call i64 @primer.array.get.i64.3([3 x i64] %tmp9, i64 1, ptr @primer.failure.17.101.110)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp10)
  ret i32 0
}
