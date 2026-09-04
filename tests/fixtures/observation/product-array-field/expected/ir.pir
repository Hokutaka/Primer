; Primer IR v0.1

type %Row@0 {
  field %values@0: [i64; 3]
}

mut %first@0: %Row@0 = construct %Row@0 { field %values@0 = array[1i64, 2i64, 3i64]:[i64; 3] [explicit]; }
%second@1: %Row@0 = %first@0:%Row@0
set %first@0:%Row@0 = construct %Row@0 { field %values@0 = array[4i64, 5i64, 6i64]:[i64; 3] [explicit]; }
print.i64 index(field(%second@1:%Row@0, %values@0):[i64; 3], 1i64):i64
print.i64 index(field(%first@0:%Row@0, %values@0):[i64; 3], 2i64):i64
