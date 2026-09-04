; Primer IR v0.2
; #N identifies one statement or expression in this compilation

type %Row@0 {
  field %values@0: [i64; 3]
}

#0 mut %first@0: %Row@0 = #1 construct %Row@0 { field %values@0 = #2 array[#3 1i64, #4 2i64, #5 3i64]:[i64; 3] [explicit]; }
#6 %second@1: %Row@0 = #7 %first@0:%Row@0
#8 set %first@0:%Row@0 = #9 construct %Row@0 { field %values@0 = #10 array[#11 4i64, #12 5i64, #13 6i64]:[i64; 3] [explicit]; }
#14 print.i64 #15 index(#16 field(#17 %second@1:%Row@0, %values@0):[i64; 3], #18 1i64):i64
#19 print.i64 #20 index(#21 field(#22 %first@0:%Row@0, %values@0):[i64; 3], #23 2i64):i64
