(module
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $primer_check_i8 (param $value i64) (result i64)
    local.get $value
    i64.const -128
    i64.lt_s
    local.get $value
    i64.const 127
    i64.gt_s
    i32.or
    if
      unreachable
    end
    local.get $value
  )

  (func $primer_check_u8 (param $value i64) (result i64)
    local.get $value
    i64.const 0
    i64.lt_s
    local.get $value
    i64.const 255
    i64.gt_s
    i32.or
    if
      unreachable
    end
    local.get $value
  )

  (func $primer_check_i16 (param $value i64) (result i64)
    local.get $value
    i64.const -32768
    i64.lt_s
    local.get $value
    i64.const 32767
    i64.gt_s
    i32.or
    if
      unreachable
    end
    local.get $value
  )

  (func $primer_check_u16 (param $value i64) (result i64)
    local.get $value
    i64.const 0
    i64.lt_s
    local.get $value
    i64.const 65535
    i64.gt_s
    i32.or
    if
      unreachable
    end
    local.get $value
  )

  (func $primer_i64_add (param $left i64) (param $right i64) (result i64)
    (local $result i64)
    local.get $left
    local.get $right
    i64.add
    local.set $result
    local.get $result
    local.get $left
    i64.xor
    local.get $result
    local.get $right
    i64.xor
    i64.and
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    local.get $result
  )

  (func $primer_i64_sub (param $left i64) (param $right i64) (result i64)
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
      unreachable
    end
    local.get $result
  )

  (func $primer_fn_average_0 (param $primer_left i64) (param $primer_right i64) (result i64)
    local.get $primer_left
    call $primer_check_u16
    local.get $primer_right
    call $primer_check_u16
    call $primer_i64_add
    call $primer_check_u16
    i64.const 2
    i64.div_s
    call $primer_check_u16
    call $primer_check_u8
    return
  )
  (func $main
    (local $primer_offset i64)
    (local $primer_reading i64)

    i64.const 0
    i64.const 3
    call $primer_i64_sub
    call $primer_check_i8
    local.set $primer_offset
    i64.const 0
    i64.const 32000
    call $primer_i64_sub
    call $primer_check_i16
    local.set $primer_reading
    local.get $primer_reading
    local.get $primer_offset
    call $primer_check_i16
    call $primer_i64_add
    call $primer_check_i16
    call $print_i64
    i64.const 240
    i64.const 80
    call $primer_fn_average_0
    call $print_i64
    i64.const 127
    i64.const -128
    i64.gt_s
    call $print_bool
    i64.const 255
    call $primer_check_u16
    call $print_i64
  )
  (export "main" (func $main))
)
