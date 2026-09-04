; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 %truth@0: bool = #1 true:bool
#2 %negated@1: bool = #3 not.bool(#4 %truth@0:bool)
#5 %same@2: bool = #6 eq.bool(#7 %truth@0:bool, #8 true:bool)
#9 %integer_order@3: bool = #10 lt.i64(#11 add.i64(#12 1i64, #13 2i64), #14 4i64)
#15 %float_difference@4: bool = #16 ne.f32(#17 0.1f32, #18 0.2f32)
#19 print.bool #20 %truth@0:bool
#21 print.bool #22 %negated@1:bool
#23 print.bool #24 %same@2:bool
#25 print.bool #26 %integer_order@3:bool
#27 print.bool #28 %float_difference@4:bool
