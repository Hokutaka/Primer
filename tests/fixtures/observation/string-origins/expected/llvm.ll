; primer-origins v1: UTF-8 byte ranges, end exclusive
; primer-origin: synthetic
target triple = "x86_64-unknown-linux-gnu"

%primer.string = type { ptr, i64 }
@primer.string.0 = private unnamed_addr constant [10 x i8] c"\E6\97\A5\E6\9C\AC\E8\AA\9E\00"
@primer.string.1 = private unnamed_addr constant [10 x i8] c"\E6\97\A5\E6\9C\AC\E8\AA\9E\00"
@primer.string.2 = private unnamed_addr constant [7 x i8] c"\73\6B\69\70\70\65\64"

@.fmt_i64 = private unnamed_addr constant [6 x i8] c"%lld\0A\00"
@.fmt_f32 = private unnamed_addr constant [6 x i8] c"%.9g\0A\00"
@.fmt_f64 = private unnamed_addr constant [7 x i8] c"%.17g\0A\00"
@.bool_true = private unnamed_addr constant [5 x i8] c"true\00"
@.bool_false = private unnamed_addr constant [6 x i8] c"false\00"

declare i32 @printf(ptr, ...)
declare i32 @puts(ptr)

declare i32 @putchar(i32)

define internal i1 @primer.string.equal(%primer.string %left, %primer.string %right) {
entry:
  %left.data = extractvalue %primer.string %left, 0
  %left.length = extractvalue %primer.string %left, 1
  %right.data = extractvalue %primer.string %right, 0
  %right.length = extractvalue %primer.string %right, 1
  %same.length = icmp eq i64 %left.length, %right.length
  br i1 %same.length, label %condition, label %different
condition:
  %index = phi i64 [ 0, %entry ], [ %next, %advance ]
  %done = icmp eq i64 %index, %left.length
  br i1 %done, label %equal, label %compare
compare:
  %left.ptr = getelementptr inbounds i8, ptr %left.data, i64 %index
  %right.ptr = getelementptr inbounds i8, ptr %right.data, i64 %index
  %left.byte = load i8, ptr %left.ptr
  %right.byte = load i8, ptr %right.ptr
  %same.byte = icmp eq i8 %left.byte, %right.byte
  br i1 %same.byte, label %advance, label %different
advance:
  %next = add i64 %index, 1
  br label %condition
equal:
  ret i1 true
different:
  ret i1 false
}

define internal void @primer.print.string(%primer.string %value) {
entry:
  %data = extractvalue %primer.string %value, 0
  %length = extractvalue %primer.string %value, 1
  br label %condition
condition:
  %index = phi i64 [ 0, %entry ], [ %next, %write ]
  %done = icmp eq i64 %index, %length
  br i1 %done, label %newline, label %write
write:
  %ptr = getelementptr inbounds i8, ptr %data, i64 %index
  %byte = load i8, ptr %ptr
  %character = zext i8 %byte to i32
  call i32 @putchar(i32 %character)
  %next = add i64 %index, 1
  br label %condition
newline:
  call i32 @putchar(i32 10)
  ret void
}

; primer-origin: synthetic
define %primer.string @primer.fn.echo.0(%primer.string %arg0) {
entry:
  %primer_value = alloca %primer.string
  store %primer.string %arg0, ptr %primer_value
; primer-origin: #1 bytes 136..141
  %tmp0 = load %primer.string, ptr %primer_value
; primer-origin: #0 bytes 130..143
  call void @primer.print.string(%primer.string %tmp0)
; primer-origin: #3 bytes 155..160
  %tmp1 = load %primer.string, ptr %primer_value
; primer-origin: #2 bytes 148..161
  ret %primer.string %tmp1
}

; primer-origin: synthetic
define i32 @main() {
entry:
  %primer_left = alloca %primer.string
  %primer_same = alloca i1
  %primer_logical_result2 = alloca i1
; primer-origin: #4 bytes 165..194
  store %primer.string { ptr @primer.string.0, i64 10 }, ptr %primer_left
; primer-origin: #9 bytes 213..217
  %tmp0 = load %primer.string, ptr %primer_left
; primer-origin: #8 bytes 208..218
  %tmp1 = call %primer.string @primer.fn.echo.0(%primer.string %tmp0)
; primer-origin: #7 bytes 208..235
  %tmp2 = call i1 @primer.string.equal(%primer.string %tmp1, %primer.string { ptr @primer.string.1, i64 10 })
; primer-origin: #6 bytes 195..236
  store i1 %tmp2, ptr %primer_same
; primer-origin: #12 bytes 243..247
  %tmp3 = load i1, ptr %primer_same
; primer-origin: #11 bytes 237..249
  %tmp4 = select i1 %tmp3, ptr @.bool_true, ptr @.bool_false
; primer-origin: #11 bytes 237..249
  call i32 @puts(ptr %tmp4)
; primer-origin: #14 bytes 256..288
  store i1 0, ptr %primer_logical_result2
; primer-origin: #14 bytes 256..288
  br i1 0, label %block0, label %block1
; primer-origin: #14 bytes 256..288
block0: ; logical_rhs
; primer-origin: #17 bytes 265..280
  %tmp5 = call %primer.string @primer.fn.echo.0(%primer.string { ptr @primer.string.2, i64 7 })
; primer-origin: #19 bytes 284..288
  %tmp6 = load %primer.string, ptr %primer_left
; primer-origin: #16 bytes 265..288
  %tmp7 = call i1 @primer.string.equal(%primer.string %tmp5, %primer.string %tmp6)
; primer-origin: #14 bytes 256..288
  store i1 %tmp7, ptr %primer_logical_result2
; primer-origin: #14 bytes 256..288
  br label %block1
; primer-origin: #14 bytes 256..288
block1: ; logical_end
; primer-origin: #14 bytes 256..288
  %tmp8 = load i1, ptr %primer_logical_result2
; primer-origin: #13 bytes 250..290
  %tmp9 = select i1 %tmp8, ptr @.bool_true, ptr @.bool_false
; primer-origin: #13 bytes 250..290
  call i32 @puts(ptr %tmp9)
; primer-origin: synthetic
  ret i32 0
}
