; Primer IR v0.1

%single: f32 = add.f32(0.1f32, 0.2f32)
%double: f64 = add.f64(0.1f64, 0.2f64)
%inferred: f64 = add.f64(0.1f64, 0.2f64)
%suffixed: f32 = add.f32(0.1f32, 0.2f32)
print.f32 %single:f32
print.f64 %double:f64
print.f64 %inferred:f64
print.f32 %suffixed:f32
