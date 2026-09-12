target triple = "x86_64-unknown-linux-gnu"

@primer.failure.6.83.85.0 = private unnamed_addr constant [61 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\74\65\67\65\72\2D\6F\76\65\72\66\6C\6F\77\20\6E\6F\64\65\3D\36\20\62\79\74\65\73\3D\38\33\2E\2E\38\35\0A"
@primer.failure.6.83.85 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } { ptr @primer.failure.6.83.85.0, i64 61 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@primer.failure.14.136.145.1 = private unnamed_addr constant [64 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\64\69\76\69\73\69\6F\6E\2D\62\79\2D\7A\65\72\6F\20\6E\6F\64\65\3D\31\34\20\62\79\74\65\73\3D\31\33\36\2E\2E\31\34\35\0A"
@primer.failure.14.136.145.2 = private unnamed_addr constant [65 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\64\69\76\69\73\69\6F\6E\2D\6F\76\65\72\66\6C\6F\77\20\6E\6F\64\65\3D\31\34\20\62\79\74\65\73\3D\31\33\36\2E\2E\31\34\35\0A"
@primer.failure.14.136.145 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.14.136.145.1, i64 64 }, { ptr, i64 } { ptr @primer.failure.14.136.145.2, i64 65 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@primer.failure.25.200.219.5 = private unnamed_addr constant [79 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\74\65\67\65\72\2D\63\6F\6E\76\65\72\73\69\6F\6E\2D\6F\75\74\2D\6F\66\2D\72\61\6E\67\65\20\6E\6F\64\65\3D\32\35\20\62\79\74\65\73\3D\32\30\30\2E\2E\32\31\39\0A"
@primer.failure.25.200.219 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } { ptr @primer.failure.25.200.219.5, i64 79 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@primer.failure.1.50.62.0 = private unnamed_addr constant [61 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\74\65\67\65\72\2D\6F\76\65\72\66\6C\6F\77\20\6E\6F\64\65\3D\31\20\62\79\74\65\73\3D\35\30\2E\2E\36\32\0A"
@primer.failure.1.50.62 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } { ptr @primer.failure.1.50.62.0, i64 61 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)
declare void @llvm.trap()
declare { i64, i1 } @llvm.sadd.with.overflow.i64(i64, i64)
declare { i64, i1 } @llvm.ssub.with.overflow.i64(i64, i64)

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

define internal i64 @primer_check_i32(i64 %value, ptr %failure, i64 %code) {
entry:
  %below = icmp slt i64 %value, -2147483648
  %above = icmp sgt i64 %value, 2147483647
  %bad = or i1 %below, %above
  br i1 %bad, label %trap, label %ok
trap:
  call void @primer.runtime.fail(ptr %failure, i64 %code)
  unreachable
ok:
  ret i64 %value
}

define internal i64 @primer_check_u32(i64 %value, ptr %failure, i64 %code) {
entry:
  %below = icmp slt i64 %value, 0
  %above = icmp sgt i64 %value, 4294967295
  %bad = or i1 %below, %above
  br i1 %bad, label %trap, label %ok
trap:
  call void @primer.runtime.fail(ptr %failure, i64 %code)
  unreachable
ok:
  ret i64 %value
}

define internal i64 @primer_i64_add(i64 %left, i64 %right, ptr %failure) {
entry:
  %checked = call { i64, i1 } @llvm.sadd.with.overflow.i64(i64 %left, i64 %right)
  %result = extractvalue { i64, i1 } %checked, 0
  %overflow = extractvalue { i64, i1 } %checked, 1
  br i1 %overflow, label %trap, label %ok

trap:
  call void @primer.runtime.fail(ptr %failure, i64 0)
  unreachable
ok:
  ret i64 %result
}

define internal i64 @primer_i64_sub(i64 %left, i64 %right, ptr %failure) {
entry:
  %checked = call { i64, i1 } @llvm.ssub.with.overflow.i64(i64 %left, i64 %right)
  %result = extractvalue { i64, i1 } %checked, 0
  %overflow = extractvalue { i64, i1 } %checked, 1
  br i1 %overflow, label %trap, label %ok

trap:
  call void @primer.runtime.fail(ptr %failure, i64 0)
  unreachable
ok:
  ret i64 %result
}

define internal i64 @primer_i64_div(i64 %left, i64 %right, ptr %failure) {
entry:
  %is_zero = icmp eq i64 %right, 0
  br i1 %is_zero, label %zero, label %bounds
zero:
  call void @primer.runtime.fail(ptr %failure, i64 1)
  unreachable
bounds:
  %is_min = icmp eq i64 %left, -9223372036854775808
  %is_negative_one = icmp eq i64 %right, -1
  %overflows = and i1 %is_min, %is_negative_one
  br i1 %overflows, label %overflow, label %ok
overflow:
  call void @primer.runtime.fail(ptr %failure, i64 2)
  unreachable
ok:
  %result = sdiv i64 %left, %right
  ret i64 %result
}

define i64 @primer.fn.add.0(i64 %arg0, i64 %arg1) {
entry:
  %primer_left = alloca i64
  %primer_right = alloca i64
  store i64 %arg0, ptr %primer_left
  store i64 %arg1, ptr %primer_right
  %tmp0 = load i64, ptr %primer_left
  %tmp1 = load i64, ptr %primer_right
  %tmp2 = call i64 @primer_i64_add(i64 %tmp0, i64 %tmp1, ptr @primer.failure.1.50.62)
  %tmp3 = call i64 @primer_check_i32(i64 %tmp2, ptr @primer.failure.1.50.62, i64 0)
  ret i64 %tmp3
}

define i32 @main() {
entry:
  %primer_small = alloca i64
  %primer_large = alloca i64
  %tmp0 = call i64 @primer_i64_sub(i64 0, i64 3, ptr @primer.failure.6.83.85)
  %tmp1 = call i64 @primer_check_i32(i64 %tmp0, ptr @primer.failure.6.83.85, i64 0)
  %tmp2 = call i64 @primer.fn.add.0(i64 %tmp1, i64 5)
  store i64 %tmp2, ptr %primer_small
  store i64 4294967295, ptr %primer_large
  %tmp3 = load i64, ptr %primer_small
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp3)
  %tmp4 = load i64, ptr %primer_large
  %tmp5 = call i64 @primer_i64_div(i64 %tmp4, i64 2, ptr @primer.failure.14.136.145)
  %tmp6 = call i64 @primer_check_u32(i64 %tmp5, ptr @primer.failure.14.136.145, i64 2)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp6)
  %tmp7 = load i64, ptr %primer_large
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp7)
  %tmp8 = load i64, ptr %primer_large
  %tmp9 = icmp sgt i64 %tmp8, 2147483648
  %tmp10 = select i1 %tmp9, ptr @.bool_true, ptr @.bool_false
  call i32 @puts(ptr %tmp10)
  %tmp11 = load i64, ptr %primer_small
  %tmp12 = call i64 @primer_check_u32(i64 %tmp11, ptr @primer.failure.25.200.219, i64 5)
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp12)
  ret i32 0
}
