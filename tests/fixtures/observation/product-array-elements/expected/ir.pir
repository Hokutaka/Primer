; Primer IR v0.2
; #N identifies one statement or expression in this compilation

type %Point@0 {
  field %x@0: i64
  field %y@1: i64
}

#0 mut %points@0: [%Point@0; 2] = #1 array[#2 construct %Point@0 { field %x@0 = #3 1i64 [explicit]; field %y@1 = #4 2i64 [explicit]; }, #5 construct %Point@0 { field %x@0 = #6 3i64 [explicit]; field %y@1 = #7 4i64 [explicit]; }]:[%Point@0; 2]
#8 %copy@1: [%Point@0; 2] = #9 %points@0:[%Point@0; 2]
#10 set %points@0:[%Point@0; 2] = #11 array[#12 construct %Point@0 { field %x@0 = #13 5i64 [explicit]; field %y@1 = #14 6i64 [explicit]; }, #15 construct %Point@0 { field %x@0 = #16 7i64 [explicit]; field %y@1 = #17 8i64 [explicit]; }]:[%Point@0; 2]
#18 print.i64 #19 field(#20 index(#21 %copy@1:[%Point@0; 2], #22 1i64):%Point@0, %x@0):i64
#23 print.i64 #24 field(#25 index(#26 %points@0:[%Point@0; 2], #27 0i64):%Point@0, %y@1):i64
