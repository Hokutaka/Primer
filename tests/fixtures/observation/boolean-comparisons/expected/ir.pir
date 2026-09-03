; Primer IR v0.1

%truth@0: bool = true:bool
%negated@1: bool = not.bool(%truth@0:bool)
%same@2: bool = eq.bool(%truth@0:bool, true:bool)
%integer_order@3: bool = lt.i64(add.i64(1i64, 2i64), 4i64)
%float_difference@4: bool = ne.f32(0.1f32, 0.2f32)
print.bool %truth@0:bool
print.bool %negated@1:bool
print.bool %same@2:bool
print.bool %integer_order@3:bool
print.bool %float_difference@4:bool
