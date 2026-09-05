(module
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $primer_check_i32 (param $value i64) (result i64)
    local.get $value
    i64.const -2147483648
    i64.lt_s
    local.get $value
    i64.const 2147483647
    i64.gt_s
    i32.or
    if
      unreachable
    end
    local.get $value
  )

  (func $primer_check_u32 (param $value i64) (result i64)
    local.get $value
    i64.const 0
    i64.lt_s
    local.get $value
    i64.const 4294967295
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

  (func $primer_fn_add_0 (param $primer_left i64) (param $primer_right i64) (result i64)
    local.get $primer_left
    local.get $primer_right
    call $primer_i64_add
    call $primer_check_i32
    return
  )
  (func $main
    (local $primer_small i64)
    (local $primer_large i64)

    i64.const 0
    i64.const 3
    call $primer_i64_sub
    call $primer_check_i32
    i64.const 5
    call $primer_fn_add_0
    local.set $primer_small
    i64.const 4294967295
    local.set $primer_large
    local.get $primer_small
    call $print_i64
    local.get $primer_large
    i64.const 2
    i64.div_s
    call $primer_check_u32
    call $print_i64
    local.get $primer_large
    call $print_i64
    local.get $primer_large
    i64.const 2147483648
    i64.gt_s
    call $print_bool
    local.get $primer_small
    call $primer_check_u32
    call $print_i64
  )
  (export "main" (func $main))
)
