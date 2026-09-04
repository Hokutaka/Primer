; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 mut %count@0: i64 = #1 0i64
#2 mut %sum@1: i64 = #3 0i64
#4 while.bool #5 lt.i64(#6 %count@0:i64, #7 4i64) {
  #8 set %sum@1:i64 = #9 add.i64(#10 %sum@1:i64, #11 %count@0:i64)
  #12 if.bool #13 eq.i64(#14 %count@0:i64, #15 2i64) {
    #16 %marker@2: bool = #17 true:bool
    #18 print.bool #19 %marker@2:bool
  }
  #20 set %count@0:i64 = #21 add.i64(#22 %count@0:i64, #23 1i64)
}
#24 print.i64 #25 %sum@1:i64
