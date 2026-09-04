(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

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

  (func $primer_i64_mul (param $left i64) (param $right i64) (result i64)
    (local $result i64)
    local.get $left
    i64.eqz
    if
      i64.const 0
      return
    end
    local.get $left
    i64.const -1
    i64.eq
    local.get $right
    i64.const -9223372036854775808
    i64.eq
    i32.and
    local.get $right
    i64.const -1
    i64.eq
    local.get $left
    i64.const -9223372036854775808
    i64.eq
    i32.and
    i32.or
    if
      unreachable
    end
    local.get $left
    local.get $right
    i64.mul
    local.set $result
    local.get $result
    local.get $left
    i64.div_s
    local.get $right
    i64.ne
    if
      unreachable
    end
    local.get $result
  )

  (func $main
    (local $primer_value i64)

    i64.const 8
    local.set $primer_value
    local.get $primer_value
    i64.const 1
    call $primer_i64_add
    call $print_i64
    local.get $primer_value
    i64.const 1
    call $primer_i64_sub
    call $print_i64
    local.get $primer_value
    i64.const 2
    call $primer_i64_mul
    call $print_i64
    local.get $primer_value
    i64.const 2
    i64.div_s
    call $print_i64
    i64.const 0
    local.get $primer_value
    call $primer_i64_sub
    call $print_i64
  )
  (export "main" (func $main))
)
