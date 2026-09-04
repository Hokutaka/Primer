; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 mut %value@0: i64 = #1 1i64
#2 if.bool #3 lt.i64(#4 %value@0:i64, #5 2i64) {
  #6 set %value@0:i64 = #7 42i64
  #8 %value@1: bool = #9 true:bool
  #10 print.bool #11 %value@1:bool
} else {
  #12 set %value@0:i64 = #13 neg.i64(#14 1i64)
}
#15 print.i64 #16 %value@0:i64
