; Primer IR v0.1

mut %value@0: i64 = 0i64
mut %sum@1: i64 = 0i64
while.bool lt.i64(%value@0:i64, 10i64) {
  set %value@0:i64 = add.i64(%value@0:i64, 1i64)
  if.bool lt.i64(%value@0:i64, 3i64) {
    continue
  }
  if.bool gt.i64(%value@0:i64, 5i64) {
    break
  }
  set %sum@1:i64 = add.i64(%sum@1:i64, %value@0:i64)
}
print.i64 %sum@1:i64
print.i64 %value@0:i64
