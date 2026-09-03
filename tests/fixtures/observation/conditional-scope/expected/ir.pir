; Primer IR v0.1

mut %value@0: i64 = 1i64
if.bool lt.i64(%value@0:i64, 2i64) {
  set %value@0:i64 = 42i64
  %value@1: bool = true:bool
  print.bool %value@1:bool
} else {
  set %value@0:i64 = neg.i64(1i64)
}
print.i64 %value@0:i64
