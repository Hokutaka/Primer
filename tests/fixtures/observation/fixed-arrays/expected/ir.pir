; Primer IR v0.1

mut %values@0: [i64; 3] = array[2i64, 4i64, 6i64]:[i64; 3]
%copy@1: [i64; 3] = %values@0:[i64; 3]
set %values@0:[i64; 3] = array[1i64, 3i64, 5i64]:[i64; 3]
print.i64 index(%copy@1:[i64; 3], 2i64):i64
print.i64 index(%values@0:[i64; 3], 1i64):i64
