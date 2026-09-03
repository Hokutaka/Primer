; Primer IR v0.1

%single@0: f32 = add.f32(0.1f32, 0.2f32)
%double@1: f64 = add.f64(0.1f64, 0.2f64)
%inferred@2: f64 = add.f64(0.1f64, 0.2f64)
%suffixed@3: f32 = add.f32(0.1f32, 0.2f32)
print.f32 %single@0:f32
print.f64 %double@1:f64
print.f64 %inferred@2:f64
print.f32 %suffixed@3:f32
