target triple = "x86_64-unknown-linux-gnu"

@primer.failure.24.125.132.11 = private unnamed_addr constant [73 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\61\72\72\61\79\2D\69\6E\64\65\78\2D\6F\75\74\2D\6F\66\2D\62\6F\75\6E\64\73\20\6E\6F\64\65\3D\32\34\20\62\79\74\65\73\3D\31\32\35\2E\2E\31\33\32\0A"
@primer.failure.24.125.132 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.24.125.132.11, i64 73 }]
@primer.failure.23.125.135.11 = private unnamed_addr constant [73 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\61\72\72\61\79\2D\69\6E\64\65\78\2D\6F\75\74\2D\6F\66\2D\62\6F\75\6E\64\73\20\6E\6F\64\65\3D\32\33\20\62\79\74\65\73\3D\31\32\35\2E\2E\31\33\35\0A"
@primer.failure.23.125.135 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.23.125.135.11, i64 73 }]
@primer.failure.30.144.153.11 = private unnamed_addr constant [73 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\61\72\72\61\79\2D\69\6E\64\65\78\2D\6F\75\74\2D\6F\66\2D\62\6F\75\6E\64\73\20\6E\6F\64\65\3D\33\30\20\62\79\74\65\73\3D\31\34\34\2E\2E\31\35\33\0A"
@primer.failure.30.144.153 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.30.144.153.11, i64 73 }]
@primer.failure.29.144.156.11 = private unnamed_addr constant [73 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\61\72\72\61\79\2D\69\6E\64\65\78\2D\6F\75\74\2D\6F\66\2D\62\6F\75\6E\64\73\20\6E\6F\64\65\3D\32\39\20\62\79\74\65\73\3D\31\34\34\2E\2E\31\35\36\0A"
@primer.failure.29.144.156 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.29.144.156.11, i64 73 }]
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

define internal [3 x i64] @primer.array.get.array.i64.3.2([2 x [3 x i64]] %value, i64 %index, ptr %failure) {
entry:
  %index.low = icmp slt i64 %index, 0
  %index.high = icmp sge i64 %index, 2
  %index.outside = or i1 %index.low, %index.high
  br i1 %index.outside, label %out_of_bounds, label %in_bounds
out_of_bounds:
  call void @primer.runtime.fail(ptr %failure, i64 11)
  unreachable
in_bounds:
  %array = alloca [2 x [3 x i64]]
  store [2 x [3 x i64]] %value, ptr %array
  %element = getelementptr inbounds [2 x [3 x i64]], ptr %array, i64 0, i64 %index
  %result = load [3 x i64], ptr %element
  ret [3 x i64] %result
}

define i32 @main() {
entry:
  %primer_matrix = alloca [2 x [3 x i64]]
  %primer_copy = alloca [2 x [3 x i64]]
  %tmp0 = insertvalue [3 x i64] poison, i64 1, 0
  %tmp1 = insertvalue [3 x i64] %tmp0, i64 2, 1
  %tmp2 = insertvalue [3 x i64] %tmp1, i64 3, 2
  %tmp3 = insertvalue [2 x [3 x i64]] poison, [3 x i64] %tmp2, 0
  %tmp4 = insertvalue [3 x i64] poison, i64 4, 0
  %tmp5 = insertvalue [3 x i64] %tmp4, i64 5, 1
  %tmp6 = insertvalue [3 x i64] %tmp5, i64 6, 2
  %tmp7 = insertvalue [2 x [3 x i64]] %tmp3, [3 x i64] %tmp6, 1
  store [2 x [3 x i64]] %tmp7, ptr %primer_matrix
  %tmp8 = load [2 x [3 x i64]], ptr %primer_matrix
  store [2 x [3 x i64]] %tmp8, ptr %primer_copy
  %tmp9 = insertvalue [3 x i64] poison, i64 7, 0
  %tmp10 = insertvalue [3 x i64] %tmp9, i64 8, 1
  %tmp11 = insertvalue [3 x i64] %tmp10, i64 9, 2
  %tmp12 = insertvalue [2 x [3 x i64]] poison, [3 x i64] %tmp11, 0
  %tmp13 = insertvalue [3 x i64] poison, i64 10, 0
  %tmp14 = insertvalue [3 x i64] %tmp13, i64 11, 1
  %tmp15 = insertvalue [3 x i64] %tmp14, i64 12, 2
  %tmp16 = insertvalue [2 x [3 x i64]] %tmp12, [3 x i64] %tmp15, 1
  store [2 x [3 x i64]] %tmp16, ptr %primer_matrix
  %tmp17 = load [2 x [3 x i64]], ptr %primer_copy
  %tmp18 = call [3 x i64] @primer.array.get.array.i64.3.2([2 x [3 x i64]] %tmp17, i64 1, ptr @primer.failure.24.125.132)
  %tmp19 = call i64 @primer.array.get.i64.3([3 x i64] %tmp18, i64 2, ptr @primer.failure.23.125.135)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp19)
  %tmp20 = load [2 x [3 x i64]], ptr %primer_matrix
  %tmp21 = call [3 x i64] @primer.array.get.array.i64.3.2([2 x [3 x i64]] %tmp20, i64 0, ptr @primer.failure.30.144.153)
  %tmp22 = call i64 @primer.array.get.i64.3([3 x i64] %tmp21, i64 1, ptr @primer.failure.29.144.156)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp22)
  ret i32 0
}
