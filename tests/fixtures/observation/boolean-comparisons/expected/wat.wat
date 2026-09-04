(module
  (import "primer" "print_bool" (func $print_bool (param i32)))
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

  (func $main
    (local $primer_truth i32)
    (local $primer_negated i32)
    (local $primer_same i32)
    (local $primer_integer_order i32)
    (local $primer_float_difference i32)

    i32.const 1
    local.set $primer_truth
    local.get $primer_truth
    i32.eqz
    local.set $primer_negated
    local.get $primer_truth
    i32.const 1
    i32.eq
    local.set $primer_same
    i64.const 1
    i64.const 2
    call $primer_i64_add
    i64.const 4
    i64.lt_s
    local.set $primer_integer_order
    f32.const 0.1
    f32.const 0.2
    f32.ne
    local.set $primer_float_difference
    local.get $primer_truth
    call $print_bool
    local.get $primer_negated
    call $print_bool
    local.get $primer_same
    call $print_bool
    local.get $primer_integer_order
    call $print_bool
    local.get $primer_float_difference
    call $print_bool
  )
  (export "main" (func $main))
)
