; Primer IR v0.2
; #N identifies one statement or expression in this compilation

type %Point@0 {
  field %x@0: i64
  field %y@1: i64
}

fn %move_x@0(%point@0: %Point@0, %amount@1: i64) -> %Point@0 {
  #0 return #1 construct %Point@0 { field %x@0 = #2 add.i64(#3 field(#4 %point@0:%Point@0, %x@0):i64, #5 %amount@1:i64) [explicit]; field %y@1 = #6 field(#7 %point@0:%Point@0, %y@1):i64 [explicit]; }
}

fn %move_twice@1(%point@2: %Point@0, %amount@3: i64) -> %Point@0 {
  #8 return #9 call %move_x@0(#10 call %move_x@0(#11 %point@2:%Point@0, #12 %amount@3:i64):%Point@0, #13 %amount@3:i64):%Point@0
}

fn %first_row@2(%matrix@4: [[i64; 2]; 2]) -> [i64; 2] {
  #14 return #15 index(#16 %matrix@4:[[i64; 2]; 2], #17 0i64):[i64; 2]
}

fn %duplicate@3(%row@5: [i64; 2]) -> [[i64; 2]; 2] {
  #18 return #19 array[#20 %row@5:[i64; 2], #21 %row@5:[i64; 2]]:[[i64; 2]; 2]
}

fn %duplicate_first_row@4(%matrix@6: [[i64; 2]; 2]) -> [[i64; 2]; 2] {
  #22 return #23 call %duplicate@3(#24 call %first_row@2(#25 %matrix@6:[[i64; 2]; 2]):[i64; 2]):[[i64; 2]; 2]
}

#26 %original@7: %Point@0 = #27 construct %Point@0 { field %x@0 = #28 2i64 [explicit]; field %y@1 = #29 3i64 [explicit]; }
#30 %moved@8: %Point@0 = #31 call %move_twice@1(#32 %original@7:%Point@0, #33 5i64):%Point@0
#34 %matrix@9: [[i64; 2]; 2] = #35 array[#36 array[#37 1i64, #38 2i64]:[i64; 2], #39 array[#40 3i64, #41 4i64]:[i64; 2]]:[[i64; 2]; 2]
#42 %rows@10: [[i64; 2]; 2] = #43 call %duplicate_first_row@4(#44 %matrix@9:[[i64; 2]; 2]):[[i64; 2]; 2]
#45 print.i64 #46 field(#47 %original@7:%Point@0, %x@0):i64
#48 print.i64 #49 field(#50 %moved@8:%Point@0, %x@0):i64
#51 print.i64 #52 field(#53 %moved@8:%Point@0, %y@1):i64
#54 print.i64 #55 index(#56 index(#57 %matrix@9:[[i64; 2]; 2], #58 1i64):[i64; 2], #59 0i64):i64
#60 print.i64 #61 index(#62 index(#63 %rows@10:[[i64; 2]; 2], #64 0i64):[i64; 2], #65 1i64):i64
#66 print.i64 #67 index(#68 index(#69 %rows@10:[[i64; 2]; 2], #70 1i64):[i64; 2], #71 0i64):i64
