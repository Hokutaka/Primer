target triple = "x86_64-unknown-linux-gnu"

@.fmt_u64 = private unnamed_addr constant [6 x i8] c"%llu\0A\00"
@primer.failure.9.79.90.1 = private unnamed_addr constant [61 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\64\69\76\69\73\69\6F\6E\2D\62\79\2D\7A\65\72\6F\20\6E\6F\64\65\3D\39\20\62\79\74\65\73\3D\37\39\2E\2E\39\30\0A"
@primer.failure.9.79.90 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.9.79.90.1, i64 61 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@primer.failure.13.99.112.4 = private unnamed_addr constant [66 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\76\61\6C\69\64\2D\73\68\69\66\74\2D\63\6F\75\6E\74\20\6E\6F\64\65\3D\31\33\20\62\79\74\65\73\3D\39\39\2E\2E\31\31\32\0A"
@primer.failure.13.99.112 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.13.99.112.4, i64 66 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@primer.failure.17.121.131.5 = private unnamed_addr constant [79 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\74\65\67\65\72\2D\63\6F\6E\76\65\72\73\69\6F\6E\2D\6F\75\74\2D\6F\66\2D\72\61\6E\67\65\20\6E\6F\64\65\3D\31\37\20\62\79\74\65\73\3D\31\32\31\2E\2E\31\33\31\0A"
@primer.failure.17.121.131 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.17.121.131.5, i64 79 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
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

define internal i64 @primer_convert_i64_u64(i64 %value, ptr %failure) {
entry:
  %bad = icmp slt i64 %value, 0
  br i1 %bad, label %trap, label %ok
trap:
  call void @primer.runtime.fail(ptr %failure, i64 5)
  unreachable
ok:
  ret i64 %value
}

define internal i64 @primer_u64_div(i64 %left, i64 %right, ptr %failure) {
entry:
  %bad = icmp eq i64 %right, 0
  br i1 %bad, label %trap, label %ok
trap:
  call void @primer.runtime.fail(ptr %failure, i64 1)
  unreachable
ok:
  %result = udiv i64 %left, %right
  ret i64 %result
}

define internal i64 @primer_u64_shr(i64 %left, i64 %right, ptr %failure) {
entry:
  %wide = icmp uge i64 %right, 64
  br i1 %wide, label %count, label %bounds
count:
  call void @primer.runtime.fail(ptr %failure, i64 4)
  unreachable
bounds:
  %result = lshr i64 %left, %right
  ret i64 %result
}

define i32 @main() {
entry:
  %primer_maximum = alloca i64
  store i64 -1, ptr %primer_maximum
  %tmp0 = load i64, ptr %primer_maximum
  call i32 (ptr, ...) @printf(ptr @.fmt_u64, i64 %tmp0)
  %tmp1 = load i64, ptr %primer_maximum
  %tmp2 = icmp ugt i64 %tmp1, 0
  %tmp3 = select i1 %tmp2, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp3)
  %tmp4 = load i64, ptr %primer_maximum
  %tmp5 = call i64 @primer_u64_div(i64 %tmp4, i64 2, ptr @primer.failure.9.79.90)
  call i32 (ptr, ...) @printf(ptr @.fmt_u64, i64 %tmp5)
  %tmp6 = load i64, ptr %primer_maximum
  %tmp7 = call i64 @primer_u64_shr(i64 %tmp6, i64 63, ptr @primer.failure.13.99.112)
  call i32 (ptr, ...) @printf(ptr @.fmt_u64, i64 %tmp7)
  %tmp8 = call i64 @primer_convert_i64_u64(i64 42, ptr @primer.failure.17.121.131)
  call i32 (ptr, ...) @printf(ptr @.fmt_u64, i64 %tmp8)
  ret i32 0
}
