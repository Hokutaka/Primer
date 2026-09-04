; Primer IR v0.2
; #N identifies one statement or expression in this compilation

type %Point@0 {
  field %x@0: f64 = #0 0.0f64
  field %y@1: f64
}

type %Segment@1 {
  field %start@0: %Point@0
  field %end@1: %Point@0
}

#1 mut %current@0: %Point@0 = #2 construct %Point@0 { field %y@1 = #3 2.0f64 [explicit]; field %x@0 = #4 0.0f64 [default]; }
#5 %saved@1: %Point@0 = #6 %current@0:%Point@0
#7 set %current@0:%Point@0 = #8 construct %Point@0 { field %x@0 = #9 4.0f64 [explicit]; field %y@1 = #10 5.0f64 [explicit]; }
#11 %segment@2: %Segment@1 = #12 construct %Segment@1 { field %start@0 = #13 %saved@1:%Point@0 [explicit]; field %end@1 = #14 %current@0:%Point@0 [explicit]; }
#15 print.f64 #16 field(#17 %saved@1:%Point@0, %x@0):f64
#18 print.f64 #19 field(#20 %saved@1:%Point@0, %y@1):f64
#21 print.f64 #22 field(#23 field(#24 %segment@2:%Segment@1, %start@0):%Point@0, %y@1):f64
#25 print.f64 #26 field(#27 field(#28 %segment@2:%Segment@1, %end@1):%Point@0, %x@0):f64
