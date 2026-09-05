; Primer IR v0.2
; #N identifies one statement or expression in this compilation

fn %report@0(%value@0: bool) -> bool {
  #0 print.bool #1 %value@0:bool
  #2 return #3 %value@0:bool
}

#4 %values@1: [i64; 2] = #5 array[#6 4i64, #7 9i64]:[i64; 2]
#8 %index@2: i64 = #9 2i64
#10 print.bool #11 and.short_circuit.bool(#12 lt.i64(#13 %index@2:i64, #14 2i64), #15 gt.i64(#16 index(#17 %values@1:[i64; 2], #18 %index@2:i64):i64, #19 0i64))
#20 print.bool #21 or.short_circuit.bool(#22 eq.i64(#23 %index@2:i64, #24 2i64), #25 call %report@0(#26 false:bool):bool)
#27 print.bool #28 or.short_circuit.bool(#29 false:bool, #30 and.short_circuit.bool(#31 call %report@0(#32 true:bool):bool, #33 or.short_circuit.bool(#34 gt.i64(#35 %index@2:i64, #36 0i64), #37 call %report@0(#38 false:bool):bool)))
