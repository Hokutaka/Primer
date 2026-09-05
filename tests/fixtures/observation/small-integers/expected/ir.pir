; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %average@0(%left@0: u8, %right@1: u8) -> u8 {
  #0 return #1 convert.checked.u16->u8[compact](#2 div.u16(#3 add.u16(#4 convert.checked.u8->u16[compact](#5 %left@0:u8), #6 convert.checked.u8->u16[compact](#7 %right@1:u8)), #8 2u16))
}

#9 %offset@2: i8 = #10 neg.i8(#11 3i8)
#12 %reading@3: i16 = #13 neg.i16(#14 32000i16)
#15 print.i16 #16 add.i16(#17 %reading@3:i16, #18 convert.checked.i8->i16[compact](#19 %offset@2:i8))
#20 print.u8 #21 call %average@0(#22 240u8, #23 80u8):u8
#24 print.bool #25 gt.i8(#26 127i8, #27 -128i8)
#28 print.u16 #29 convert.checked.u8->u16[explicit](#30 255u8)
