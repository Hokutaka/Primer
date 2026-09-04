; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 mut %matrix@0: [[i64; 3]; 2] = #1 array[#2 array[#3 1i64, #4 2i64, #5 3i64]:[i64; 3], #6 array[#7 4i64, #8 5i64, #9 6i64]:[i64; 3]]:[[i64; 3]; 2]
#10 %copy@1: [[i64; 3]; 2] = #11 %matrix@0:[[i64; 3]; 2]
#12 set %matrix@0:[[i64; 3]; 2] = #13 array[#14 array[#15 7i64, #16 8i64, #17 9i64]:[i64; 3], #18 array[#19 10i64, #20 11i64, #21 12i64]:[i64; 3]]:[[i64; 3]; 2]
#22 print.i64 #23 index(#24 index(#25 %copy@1:[[i64; 3]; 2], #26 1i64):[i64; 3], #27 2i64):i64
#28 print.i64 #29 index(#30 index(#31 %matrix@0:[[i64; 3]; 2], #32 0i64):[i64; 3], #33 1i64):i64
