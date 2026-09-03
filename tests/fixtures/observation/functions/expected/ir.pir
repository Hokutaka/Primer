; Primer IR v0.1

fn %add@0(%left@0: i64, %right@1: i64) -> i64 {
  return add.i64(%left@0:i64, %right@1:i64)
}

fn %show@1(%value@2: i64) -> void {
  print.i64 %value@2:i64
}

%answer@3: i64 = call %add@0(20i64, 22i64):i64
call %show@1(%answer@3:i64)
