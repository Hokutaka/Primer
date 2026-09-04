; Primer IR v0.2
; #N identifies one statement or expression in this compilation

#0 %single@0: f32 = #1 add.f32(#2 0.1f32, #3 0.2f32)
#4 %double@1: f64 = #5 add.f64(#6 0.1f64, #7 0.2f64)
#8 %inferred@2: f64 = #9 add.f64(#10 0.1f64, #11 0.2f64)
#12 %suffixed@3: f32 = #13 add.f32(#14 0.1f32, #15 0.2f32)
#16 print.f32 #17 %single@0:f32
#18 print.f64 #19 %double@1:f64
#20 print.f64 #21 %inferred@2:f64
#22 print.f32 #23 %suffixed@3:f32
