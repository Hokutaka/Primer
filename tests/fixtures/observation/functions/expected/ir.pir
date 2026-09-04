; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %add@0(%left@0: i64, %right@1: i64) -> i64 {
  #0 return #1 add.i64(#2 %left@0:i64, #3 %right@1:i64)
}

fn %show@1(%value@2: i64) -> void {
  #4 print.i64 #5 %value@2:i64
}

#6 %answer@3: i64 = #7 call %add@0(#8 20i64, #9 22i64):i64
#10 call %show@1(#11 %answer@3:i64)
