; Primer IR v0.1

type %Point@0 {
  field %x@0: i64
  field %y@1: i64
}

fn %move_x@0(%point@0: %Point@0, %amount@1: i64) -> %Point@0 {
  return construct %Point@0 { field %x@0 = add.i64(field(%point@0:%Point@0, %x@0):i64, %amount@1:i64) [explicit]; field %y@1 = field(%point@0:%Point@0, %y@1):i64 [explicit]; }
}

fn %move_twice@1(%point@2: %Point@0, %amount@3: i64) -> %Point@0 {
  return call %move_x@0(call %move_x@0(%point@2:%Point@0, %amount@3:i64):%Point@0, %amount@3:i64):%Point@0
}

fn %first_row@2(%matrix@4: [[i64; 2]; 2]) -> [i64; 2] {
  return index(%matrix@4:[[i64; 2]; 2], 0i64):[i64; 2]
}

fn %duplicate@3(%row@5: [i64; 2]) -> [[i64; 2]; 2] {
  return array[%row@5:[i64; 2], %row@5:[i64; 2]]:[[i64; 2]; 2]
}

fn %duplicate_first_row@4(%matrix@6: [[i64; 2]; 2]) -> [[i64; 2]; 2] {
  return call %duplicate@3(call %first_row@2(%matrix@6:[[i64; 2]; 2]):[i64; 2]):[[i64; 2]; 2]
}

%original@7: %Point@0 = construct %Point@0 { field %x@0 = 2i64 [explicit]; field %y@1 = 3i64 [explicit]; }
%moved@8: %Point@0 = call %move_twice@1(%original@7:%Point@0, 5i64):%Point@0
%matrix@9: [[i64; 2]; 2] = array[array[1i64, 2i64]:[i64; 2], array[3i64, 4i64]:[i64; 2]]:[[i64; 2]; 2]
%rows@10: [[i64; 2]; 2] = call %duplicate_first_row@4(%matrix@9:[[i64; 2]; 2]):[[i64; 2]; 2]
print.i64 field(%original@7:%Point@0, %x@0):i64
print.i64 field(%moved@8:%Point@0, %x@0):i64
print.i64 field(%moved@8:%Point@0, %y@1):i64
print.i64 index(index(%matrix@9:[[i64; 2]; 2], 1i64):[i64; 2], 0i64):i64
print.i64 index(index(%rows@10:[[i64; 2]; 2], 0i64):[i64; 2], 1i64):i64
print.i64 index(index(%rows@10:[[i64; 2]; 2], 1i64):[i64; 2], 0i64):i64
