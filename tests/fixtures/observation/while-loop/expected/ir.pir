; Primer IR v0.1

mut %count@0: i64 = 0i64
mut %sum@1: i64 = 0i64
while.bool lt.i64(%count@0:i64, 4i64) {
  set %sum@1:i64 = add.i64(%sum@1:i64, %count@0:i64)
  if.bool eq.i64(%count@0:i64, 2i64) {
    %marker@2: bool = true:bool
    print.bool %marker@2:bool
  }
  set %count@0:i64 = add.i64(%count@0:i64, 1i64)
}
print.i64 %sum@1:i64
