; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %measure@0(%value@0: i16) -> f64 {
  #0 return #1 div.f64(#2 convert.exact.i16->f64[compact](#3 %value@0:i16), #4 convert.exact.i64->f64[compact](#5 2i64))
}

#6 %count@1: u32 = #7 42u32
#8 %wide@2: f64 = #9 convert.exact.u32->f64[explicit](#10 %count@1:u32)
#11 %narrow@3: f32 = #12 convert.exact.f64->f32[compact](#13 %wide@2:f64)
#14 print.i16 #15 convert.exact.f32->i16[compact](#16 %narrow@3:f32)
#17 print.i64 #18 convert.exact.f64->i64[compact](#19 %wide@2:f64)
#20 print.f64 #21 convert.exact.f32->f64[compact](#22 %narrow@3:f32)
#23 print.f32 #24 convert.exact.u32->f32[compact](#25 %count@1:u32)
#26 print.f64 #27 call %measure@0(#28 3i16):f64
#29 print.f32 #30 convert.exact.f64->f32[compact](#31 neg.f64(#32 0.0f64))
