; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 mut %count@0: i64 = #1 40i64
#2 set %count@0:i64 = #3 add.i64(#4 %count@0:i64, #5 2i64)
#6 mut %ratio@1: f32 = #7 0.25f32
#8 set %ratio@1:f32 = #9 mul.f32(#10 %ratio@1:f32, #11 2.0f32)
#12 print.i64 #13 %count@0:i64
#14 print.f32 #15 %ratio@1:f32
