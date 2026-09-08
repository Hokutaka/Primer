; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 %maximum@0: u64 = #1 18446744073709551615u64
#2 print.u64 #3 %maximum@0:u64
#4 print.bool #5 gt.u64(#6 %maximum@0:u64, #7 0u64)
#8 print.u64 #9 div.u64(#10 %maximum@0:u64, #11 2u64)
#12 print.u64 #13 shr.u64(#14 %maximum@0:u64, #15 63u64)
#16 print.u64 #17 convert.checked.i64->u64[compact](#18 42i64)
