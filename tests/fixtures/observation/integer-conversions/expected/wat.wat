(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $primer_fn_value_0 (result i64)
    i64.const 7
    call $print_i64
    i64.const 42
    return
  )
  (func $main
    (local $primer_compact i64)
    (local $primer_explicit i64)

    call $primer_fn_value_0
    local.set $primer_compact
    local.get $primer_compact
    local.set $primer_explicit
    local.get $primer_explicit
    call $print_i64
  )
  (export "main" (func $main))
)
