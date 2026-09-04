; Primer IR v0.1

mut %matrix@0: [[i64; 3]; 2] = array[array[1i64, 2i64, 3i64]:[i64; 3], array[4i64, 5i64, 6i64]:[i64; 3]]:[[i64; 3]; 2]
%copy@1: [[i64; 3]; 2] = %matrix@0:[[i64; 3]; 2]
set %matrix@0:[[i64; 3]; 2] = array[array[7i64, 8i64, 9i64]:[i64; 3], array[10i64, 11i64, 12i64]:[i64; 3]]:[[i64; 3]; 2]
print.i64 index(index(%copy@1:[[i64; 3]; 2], 1i64):[i64; 3], 2i64):i64
print.i64 index(index(%matrix@0:[[i64; 3]; 2], 0i64):[i64; 3], 1i64):i64
