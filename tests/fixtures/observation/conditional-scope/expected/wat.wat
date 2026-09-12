(module
  (import "primer" "write_error_byte" (func $write_error_byte (param i32)))
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $primer_i64_sub_n13_b115_117 (param $left i64) (param $right i64) (result i64)
    (local $result i64)
    local.get $left
    local.get $right
    i64.sub
    local.set $result
    local.get $left
    local.get $right
    i64.xor
    local.get $left
    local.get $result
    i64.xor
    i64.and
    i64.const 0
    i64.lt_s
    if
      i32.const 112
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 58
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 118
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 99
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 103
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 118
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 102
      call $write_error_byte
      i32.const 108
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 119
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 51
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 53
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 55
      call $write_error_byte
      i32.const 10
      call $write_error_byte
      unreachable
    end
    local.get $result
  )

  (func $main
    (local $primer_value i64)
    (local $primer_value_1 i32)

    i64.const 1
    local.set $primer_value
    local.get $primer_value
    i64.const 2
    i64.lt_s
    if
      i64.const 42
      local.set $primer_value
      i32.const 1
      local.set $primer_value_1
      local.get $primer_value_1
      call $print_bool
    else
      i64.const 0
      i64.const 1
      call $primer_i64_sub_n13_b115_117
      local.set $primer_value
    end
    local.get $primer_value
    call $print_i64
  )
  (export "main" (func $main))
)
