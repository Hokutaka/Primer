target triple = "x86_64-unknown-linux-gnu"

@primer.failure.9.70.79.0 = private unnamed_addr constant [61 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\74\65\67\65\72\2D\6F\76\65\72\66\6C\6F\77\20\6E\6F\64\65\3D\39\20\62\79\74\65\73\3D\37\30\2E\2E\37\39\0A"
@primer.failure.9.70.79 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } { ptr @primer.failure.9.70.79.0, i64 61 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
@primer.failure.23.177.188.0 = private unnamed_addr constant [64 x i8] c"\70\72\69\6D\65\72\3A\20\72\75\6E\74\69\6D\65\2D\76\31\20\63\6F\64\65\3D\69\6E\74\65\67\65\72\2D\6F\76\65\72\66\6C\6F\77\20\6E\6F\64\65\3D\32\33\20\62\79\74\65\73\3D\31\37\37\2E\2E\31\38\38\0A"
@primer.failure.23.177.188 = private constant [12 x { ptr, i64 }] [{ ptr, i64 } { ptr @primer.failure.23.177.188.0, i64 64 }, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer, { ptr, i64 } zeroinitializer]
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
  %primer_value = alloca i64
  %primer_sum = alloca i64
  store i64 0, ptr %primer_value
  store i64 0, ptr %primer_sum
  br label %block0
block0: ; while_condition
  %tmp0 = load i64, ptr %primer_value
  %tmp1 = icmp slt i64 %tmp0, 10
  br i1 %tmp1, label %block1, label %block2
block1: ; while_body
  %tmp2 = load i64, ptr %primer_value
  %tmp3 = call i64 @primer_i64_add(i64 %tmp2, i64 1, ptr @primer.failure.9.70.79)
  store i64 %tmp3, ptr %primer_value
  %tmp4 = load i64, ptr %primer_value
  %tmp5 = icmp slt i64 %tmp4, 3
  br i1 %tmp5, label %block3, label %block5
block3: ; if_then
  br label %block0
block5: ; if_end
  %tmp6 = load i64, ptr %primer_value
  %tmp7 = icmp sgt i64 %tmp6, 5
  br i1 %tmp7, label %block6, label %block8
block6: ; if_then
  br label %block2
block8: ; if_end
  %tmp8 = load i64, ptr %primer_sum
  %tmp9 = load i64, ptr %primer_value
  %tmp10 = call i64 @primer_i64_add(i64 %tmp8, i64 %tmp9, ptr @primer.failure.23.177.188)
  store i64 %tmp10, ptr %primer_sum
  br label %block0
block2: ; while_end
  %tmp11 = load i64, ptr %primer_sum
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp11)
  %tmp12 = load i64, ptr %primer_value
  call i32 (ptr, ...) @printf(ptr @.fmt_i64, i64 %tmp12)
  ret i32 0
}
