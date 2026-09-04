; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 mut %value@0: i64 = #1 0i64
#2 mut %sum@1: i64 = #3 0i64
#4 while.bool #5 lt.i64(#6 %value@0:i64, #7 10i64) {
  #8 set %value@0:i64 = #9 add.i64(#10 %value@0:i64, #11 1i64)
  #12 if.bool #13 lt.i64(#14 %value@0:i64, #15 3i64) {
    #16 continue
  }
  #17 if.bool #18 gt.i64(#19 %value@0:i64, #20 5i64) {
    #21 break
  }
  #22 set %sum@1:i64 = #23 add.i64(#24 %sum@1:i64, #25 %value@0:i64)
}
#26 print.i64 #27 %sum@1:i64
#28 print.i64 #29 %value@0:i64
