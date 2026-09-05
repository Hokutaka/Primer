; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %value@0() -> i64 {
  #0 print.i64 #1 7i64
  #2 return #3 42i64
}

#4 %compact@0: i64 = #5 convert.checked.i64->i64[compact](#6 call %value@0():i64)
#7 %explicit@1: i64 = #8 convert.checked.i64->i64[explicit](#9 %compact@0:i64)
#10 print.i64 #11 %explicit@1:i64
