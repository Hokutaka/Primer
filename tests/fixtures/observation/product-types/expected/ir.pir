; Primer IR v0.1

type %Point@0 {
  field %x@0: f64 = 0.0f64
  field %y@1: f64
}

type %Segment@1 {
  field %start@0: %Point@0
  field %end@1: %Point@0
}

mut %current@0: %Point@0 = construct %Point@0 { field %y@1 = 2.0f64 [explicit]; field %x@0 = 0.0f64 [default]; }
%saved@1: %Point@0 = %current@0:%Point@0
set %current@0:%Point@0 = construct %Point@0 { field %x@0 = 4.0f64 [explicit]; field %y@1 = 5.0f64 [explicit]; }
%segment@2: %Segment@1 = construct %Segment@1 { field %start@0 = %saved@1:%Point@0 [explicit]; field %end@1 = %current@0:%Point@0 [explicit]; }
print.f64 field(%saved@1:%Point@0, %x@0):f64
print.f64 field(%saved@1:%Point@0, %y@1):f64
print.f64 field(field(%segment@2:%Segment@1, %start@0):%Point@0, %y@1):f64
print.f64 field(field(%segment@2:%Segment@1, %end@1):%Point@0, %x@0):f64
