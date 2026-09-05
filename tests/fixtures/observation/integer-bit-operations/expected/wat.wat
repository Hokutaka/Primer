(module
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $primer_i64_rem (param $left i64) (param $right i64) (result i64)
    local.get $left
    local.get $right
    i64.rem_s
  )

  (func $primer_u8_bit_and (param $left i64) (param $right i64) (result i64)
    local.get $left
    local.get $right
    i64.and
  )

  (func $primer_u8_bit_or (param $left i64) (param $right i64) (result i64)
    local.get $left
    local.get $right
    i64.or
  )

  (func $primer_u8_bit_xor (param $left i64) (param $right i64) (result i64)
    local.get $left
    local.get $right
    i64.xor
  )

  (func $primer_u8_shl (param $left i64) (param $right i64) (result i64)
    local.get $right
    i64.const 0
    i64.lt_s
    local.get $right
    i64.const 8
    i64.ge_s
    i32.or
    if
      unreachable
    end
    local.get $left
    i64.const 0
    local.get $right
    i64.shr_s
    i64.lt_s
    local.get $left
    i64.const 255
    local.get $right
    i64.shr_u
    i64.gt_s
    i32.or
    if
      unreachable
    end
    local.get $left
    local.get $right
    i64.shl
  )

  (func $primer_i8_shr (param $left i64) (param $right i64) (result i64)
    local.get $right
    i64.const 0
    i64.lt_s
    local.get $right
    i64.const 8
    i64.ge_s
    i32.or
    if
      unreachable
    end
    local.get $left
    local.get $right
    i64.shr_s
  )

  (func $primer_u8_shr (param $left i64) (param $right i64) (result i64)
    local.get $right
    i64.const 0
    i64.lt_s
    local.get $right
    i64.const 8
    i64.ge_s
    i32.or
    if
      unreachable
    end
    local.get $left
    local.get $right
    i64.shr_s
  )

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

  (func $primer_fn_mark_0 (param $primer_value i64) (result i64)
    local.get $primer_value
    call $print_i64
    local.get $primer_value
    return
  )
  (func $main
    (local $primer_bits i64)

    i64.const 1
    i64.const 7
    call $primer_u8_shl
    call $primer_check_u8
    local.set $primer_bits
    local.get $primer_bits
    call $print_i64
    local.get $primer_bits
    i64.const 7
    call $primer_u8_shr
    call $primer_check_u8
    call $print_i64
    i64.const 0
    i64.const 255
    call $primer_u8_bit_xor
    call $primer_check_u8
    call $print_i64
    i64.const 1
    call $primer_fn_mark_0
    i64.const 2
    call $primer_fn_mark_0
    i64.const 3
    call $primer_fn_mark_0
    call $primer_u8_bit_xor
    call $primer_check_u8
    call $primer_u8_bit_or
    call $primer_check_u8
    call $print_i64
    local.get $primer_bits
    i64.const 127
    call $primer_u8_bit_and
    call $primer_check_u8
    call $print_i64
    i64.const 0
    i64.const 7
    call $primer_i64_sub
    i64.const 3
    call $primer_i64_rem
    call $print_i64
    i64.const -9223372036854775808
    i64.const 0
    i64.const 1
    call $primer_i64_sub
    call $primer_i64_rem
    call $print_i64
    i64.const 0
    i64.const 3
    call $primer_i64_sub
    call $primer_check_i8
    i64.const 1
    call $primer_i8_shr
    call $primer_check_i8
    call $print_i64
    i32.const 0
    if (result i32)
      local.get $primer_bits
      i64.const 1
      call $primer_u8_shl
      call $primer_check_u8
      i64.const 0
      i64.eq
    else
      i32.const 0
    end
    call $print_bool
  )
  (export "main" (func $main))
)
