; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %mark@0(%value@0: u8) -> u8 {
  #0 print.u8 #1 %value@0:u8
  #2 return #3 %value@0:u8
}

#4 %bits@1: u8 = #5 shl.checked.u8(#6 1u8, #7 7u8)
#8 print.u8 #9 %bits@1:u8
#10 print.u8 #11 shr.u8(#12 %bits@1:u8, #13 7u8)
#14 print.u8 #15 bit_not.u8(#16 0u8)
#17 print.u8 #18 bit_or.u8(#19 call %mark@0(#20 1u8):u8, #21 bit_xor.u8(#22 call %mark@0(#23 2u8):u8, #24 call %mark@0(#25 3u8):u8))
#26 print.u8 #27 bit_and.u8(#28 %bits@1:u8, #29 127u8)
#30 print.i64 #31 rem.i64(#32 neg.i64(#33 7i64), #34 3i64)
#35 print.i64 #36 rem.i64(#37 -9223372036854775808i64, #38 neg.i64(#39 1i64))
#40 print.i8 #41 shr.i8(#42 neg.i8(#43 3i8), #44 1i8)
#45 print.bool #46 and.short_circuit.bool(#47 false:bool, #48 eq.u8(#49 shl.checked.u8(#50 %bits@1:u8, #51 1u8), #52 0u8))
