; Primer IR v0.1

mut %sum@0: i64 = 0i64
for.loop {
  start {
    mut %i@1: i64 = 0i64
  }
  condition.bool lt.i64(%i@1:i64, 6i64)
  body {
    if.bool lt.i64(%i@1:i64, 2i64) {
      continue
    }
    set %sum@0:i64 = add.i64(%sum@0:i64, %i@1:i64)
  }
  update {
    set %i@1:i64 = add.i64(%i@1:i64, 1i64)
  }
}
print.i64 %sum@0:i64
