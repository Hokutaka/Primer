; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 mut %values@0: [i64; 3] = #1 array[#2 2i64, #3 4i64, #4 6i64]:[i64; 3]
#5 %copy@1: [i64; 3] = #6 %values@0:[i64; 3]
#7 set %values@0:[i64; 3] = #8 array[#9 1i64, #10 3i64, #11 5i64]:[i64; 3]
#12 print.i64 #13 index(#14 %copy@1:[i64; 3], #15 2i64):i64
#16 print.i64 #17 index(#18 %values@0:[i64; 3], #19 1i64):i64
