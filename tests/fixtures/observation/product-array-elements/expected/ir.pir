; Primer IR v0.1

type %Point@0 {
  field %x@0: i64
  field %y@1: i64
}

mut %points@0: [%Point@0; 2] = array[construct %Point@0 { field %x@0 = 1i64 [explicit]; field %y@1 = 2i64 [explicit]; }, construct %Point@0 { field %x@0 = 3i64 [explicit]; field %y@1 = 4i64 [explicit]; }]:[%Point@0; 2]
%copy@1: [%Point@0; 2] = %points@0:[%Point@0; 2]
set %points@0:[%Point@0; 2] = array[construct %Point@0 { field %x@0 = 5i64 [explicit]; field %y@1 = 6i64 [explicit]; }, construct %Point@0 { field %x@0 = 7i64 [explicit]; field %y@1 = 8i64 [explicit]; }]:[%Point@0; 2]
print.i64 field(index(%copy@1:[%Point@0; 2], 1i64):%Point@0, %x@0):i64
print.i64 field(index(%points@0:[%Point@0; 2], 0i64):%Point@0, %y@1):i64
