target triple = "x86_64-unknown-linux-gnu"

@primer.failure.14.110.117.0 = private unnamed_addr constant [64 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\74\65\67\65\72\2D\6F\76\65\72\66\6C\6F\77\20\6E\6F\64\65\3D\31\34\20\62\79\74\65\73\3D\31\31\30\2E\2E\31\31\37\0A"
@primer.failure.14.110.117 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } { ptr @primer.failure.14.110.117.0, i64 64 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@primer.failure.18.51.56.0 = private unnamed_addr constant [62 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\74\65\67\65\72\2D\6F\76\65\72\66\6C\6F\77\20\6E\6F\64\65\3D\31\38\20\62\79\74\65\73\3D\35\31\2E\2E\35\36\0A"
@primer.failure.18.51.56 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } { ptr @primer.failure.18.51.56.0, i64 62 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"

declare i32 @printf(ptr, ...)
declare void @llvm.trap()
declare { i64, i1 } @llvm.sadd.with.overflow.i64(i64, i64)

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

define i32 @main() {
entry:
  %primer_sum = alloca i64
  %primer_i = alloca i64
  store i64 0, ptr %primer_sum
  store i64 0, ptr %primer_i
  br label %block0
block0: ; for_condition
  %tmp0 = load i64, ptr %primer_i
  %tmp1 = icmp slt i64 %tmp0, 6
  br i1 %tmp1, label %block1, label %block3
block1: ; for_body
  %tmp2 = load i64, ptr %primer_i
  %tmp3 = icmp slt i64 %tmp2, 2
  br i1 %tmp3, label %block4, label %block6
block4: ; if_then
  br label %block2
block6: ; if_end
  %tmp4 = load i64, ptr %primer_sum
  %tmp5 = load i64, ptr %primer_i
  %tmp6 = call i64 @primer_i64_add(i64 %tmp4, i64 %tmp5, ptr @primer.failure.14.110.117)
  store i64 %tmp6, ptr %primer_sum
  br label %block2
block2: ; for_update
  %tmp7 = load i64, ptr %primer_i
  %tmp8 = call i64 @primer_i64_add(i64 %tmp7, i64 1, ptr @primer.failure.18.51.56)
  store i64 %tmp8, ptr %primer_i
  br label %block0
block3: ; for_end
  %tmp9 = load i64, ptr %primer_sum
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp9)
  ret i32 0
}
