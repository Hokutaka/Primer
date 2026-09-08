; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %echo@0(%value@0: u64) -> u64 {
  #0 return #1 %value@0:u64
}

#2 print.string #3 "日\0":string
#4 print.u64 #5 call %echo@0(#6 18446744073709551615u64):u64
