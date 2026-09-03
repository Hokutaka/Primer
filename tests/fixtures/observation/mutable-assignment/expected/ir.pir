; Primer IR v0.1

mut %count@0: i64 = 40i64
set %count@0:i64 = add.i64(%count@0:i64, 2i64)
mut %ratio@1: f32 = 0.25f32
set %ratio@1:f32 = mul.f32(%ratio@1:f32, 2.0f32)
print.i64 %count@0:i64
print.f32 %ratio@1:f32
