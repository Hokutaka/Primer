(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $main
    (local $primer_value i64)

    i64.const -9223372036854775808
    local.set $primer_value
    local.get $primer_value
    call $print_i64
  )
  (export "main" (func $main))
)
