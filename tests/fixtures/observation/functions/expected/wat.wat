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

  (func $primer_fn_add_0 (param $primer_left i64) (param $primer_right i64) (result i64)
    local.get $primer_left
    local.get $primer_right
    call $primer_i64_add
    return
  )
  (func $primer_fn_show_1 (param $primer_value i64)
    local.get $primer_value
    call $print_i64
    return
  )
  (func $main
    (local $primer_answer i64)

    i64.const 20
    i64.const 22
    call $primer_fn_add_0
    local.set $primer_answer
    local.get $primer_answer
    call $primer_fn_show_1
  )
  (export "main" (func $main))
)
