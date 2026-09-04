; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 mut %sum@0: i64 = #1 0i64
#2 for.loop {
  start {
    #3 mut %i@1: i64 = #4 0i64
  }
  condition.bool #5 lt.i64(#6 %i@1:i64, #7 6i64)
  body {
    #8 if.bool #9 lt.i64(#10 %i@1:i64, #11 2i64) {
      #12 continue
    }
    #13 set %sum@0:i64 = #14 add.i64(#15 %sum@0:i64, #16 %i@1:i64)
  }
  update {
    #17 set %i@1:i64 = #18 add.i64(#19 %i@1:i64, #20 1i64)
  }
}
#21 print.i64 #22 %sum@0:i64
