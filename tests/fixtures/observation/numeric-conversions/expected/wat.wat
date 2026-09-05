(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $primer_convert_i16_f64 (param $value i64) (result f64)
    (local $result f64)
    (local $number f64)
    local.get $value
    f64.convert_i64_s
    local.set $result
    local.get $result
    local.set $number
    local.get $number
    f64.const -9223372036854775808
    f64.lt
    local.get $number
    f64.const 9223372036854775808
    f64.ge
    i32.or
    if
      unreachable
    end
    local.get $number
    i64.trunc_f64_s
    local.get $value
    i64.ne
    if
      unreachable
    end
    local.get $result
  )
  (func $primer_convert_u32_f32 (param $value i64) (result f32)
    (local $result f32)
    (local $number f64)
    local.get $value
    f32.convert_i64_s
    local.set $result
    local.get $result
    f64.promote_f32
    local.set $number
    local.get $number
    f64.const -9223372036854775808
    f64.lt
    local.get $number
    f64.const 9223372036854775808
    f64.ge
    i32.or
    if
      unreachable
    end
    local.get $number
    i64.trunc_f64_s
    local.get $value
    i64.ne
    if
      unreachable
    end
    local.get $result
  )
  (func $primer_convert_u32_f64 (param $value i64) (result f64)
    (local $result f64)
    (local $number f64)
    local.get $value
    f64.convert_i64_s
    local.set $result
    local.get $result
    local.set $number
    local.get $number
    f64.const -9223372036854775808
    f64.lt
    local.get $number
    f64.const 9223372036854775808
    f64.ge
    i32.or
    if
      unreachable
    end
    local.get $number
    i64.trunc_f64_s
    local.get $value
    i64.ne
    if
      unreachable
    end
    local.get $result
  )
  (func $primer_convert_i64_f64 (param $value i64) (result f64)
    (local $result f64)
    (local $number f64)
    local.get $value
    f64.convert_i64_s
    local.set $result
    local.get $result
    local.set $number
    local.get $number
    f64.const -9223372036854775808
    f64.lt
    local.get $number
    f64.const 9223372036854775808
    f64.ge
    i32.or
    if
      unreachable
    end
    local.get $number
    i64.trunc_f64_s
    local.get $value
    i64.ne
    if
      unreachable
    end
    local.get $result
  )
  (func $primer_convert_f32_i16 (param $value f32) (result i64)
    (local $result i64)
    (local $number f64)
    local.get $value
    f64.promote_f32
    local.set $number
    local.get $number
    f64.const -32768
    f64.ge
    local.get $number
    f64.const 32768
    f64.lt
    i32.and
    i32.eqz
    if
      unreachable
    end
    local.get $number
    i64.reinterpret_f64
    i64.const -9223372036854775808
    i64.eq
    if
      unreachable
    end
    local.get $number
    i64.trunc_f64_s
    local.set $result
    local.get $result
    f64.convert_i64_s
    local.get $number
    f64.ne
    if
      unreachable
    end
    local.get $result
  )
  (func $primer_convert_f32_f64 (param $value f32) (result f64)
    (local $result f64)
    (local $number f64)
    local.get $value
    local.get $value
    f32.ne
    if
      unreachable
    end
    local.get $value
    f64.promote_f32
    local.set $result
    local.get $result
  )
  (func $primer_convert_f64_i64 (param $value f64) (result i64)
    (local $result i64)
    (local $number f64)
    local.get $value
    local.set $number
    local.get $number
    f64.const -9223372036854775808
    f64.ge
    local.get $number
    f64.const 9223372036854775808
    f64.lt
    i32.and
    i32.eqz
    if
      unreachable
    end
    local.get $number
    i64.reinterpret_f64
    i64.const -9223372036854775808
    i64.eq
    if
      unreachable
    end
    local.get $number
    i64.trunc_f64_s
    local.set $result
    local.get $result
    f64.convert_i64_s
    local.get $number
    f64.ne
    if
      unreachable
    end
    local.get $result
  )
  (func $primer_convert_f64_f32 (param $value f64) (result f32)
    (local $result f32)
    (local $number f64)
    local.get $value
    local.get $value
    f64.ne
    if
      unreachable
    end
    local.get $value
    f32.demote_f64
    local.set $result
    local.get $result
    f64.promote_f32
    local.get $value
    f64.ne
    if
      unreachable
    end
    local.get $result
  )
  (func $primer_fn_measure_0 (param $primer_value i64) (result f64)
    local.get $primer_value
    call $primer_convert_i16_f64
    i64.const 2
    call $primer_convert_i64_f64
    f64.div
    return
  )
  (func $main
    (local $primer_count i64)
    (local $primer_wide f64)
    (local $primer_narrow f32)

    i64.const 42
    local.set $primer_count
    local.get $primer_count
    call $primer_convert_u32_f64
    local.set $primer_wide
    local.get $primer_wide
    call $primer_convert_f64_f32
    local.set $primer_narrow
    local.get $primer_narrow
    call $primer_convert_f32_i16
    call $print_i64
    local.get $primer_wide
    call $primer_convert_f64_i64
    call $print_i64
    local.get $primer_narrow
    call $primer_convert_f32_f64
    call $print_f64
    local.get $primer_count
    call $primer_convert_u32_f32
    call $print_f32
    i64.const 3
    call $primer_fn_measure_0
    call $print_f64
    f64.const 0.0
    f64.neg
    call $primer_convert_f64_f32
    call $print_f32
  )
  (export "main" (func $main))
)
