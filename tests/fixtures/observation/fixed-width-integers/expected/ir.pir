; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %add@0(%left@0: i32, %right@1: i32) -> i32 {
  #0 return #1 add.i32(#2 %left@0:i32, #3 %right@1:i32)
}

#4 %small@2: i32 = #5 call %add@0(#6 neg.i32(#7 3i32), #8 5i32):i32
#9 %large@3: u32 = #10 4294967295u32
#11 print.i32 #12 %small@2:i32
#13 print.u32 #14 div.u32(#15 %large@3:u32, #16 2u32)
#17 print.i64 #18 convert.checked.u32->i64[compact](#19 %large@3:u32)
#20 print.bool #21 gt.u32(#22 %large@3:u32, #23 2147483648u32)
#24 print.u32 #25 convert.checked.i32->u32[explicit](#26 %small@2:i32)
