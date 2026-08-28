(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $main
    (local $primer_single f32)
    (local $primer_double f64)
    (local $primer_inferred f64)
    (local $primer_suffixed f32)

    f32.const 0.1
    f32.const 0.2
    f32.add
    local.set $primer_single
    f64.const 0.1
    f64.const 0.2
    f64.add
    local.set $primer_double
    f64.const 0.1
    f64.const 0.2
    f64.add
    local.set $primer_inferred
    f32.const 0.1
    f32.const 0.2
    f32.add
    local.set $primer_suffixed
    local.get $primer_single
    call $print_f32
    local.get $primer_double
    call $print_f64
    local.get $primer_inferred
    call $print_f64
    local.get $primer_suffixed
    call $print_f32
  )
  (export "main" (func $main))
)
