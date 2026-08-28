(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $main
    (local $primer_x i64)

    i64.const 1
    i64.const 2
    i64.add
    local.set $primer_x
    local.get $primer_x
    call $print_i64
  )
  (export "main" (func $main))
)
