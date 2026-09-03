(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $main
    (local $primer_count i64)
    (local $primer_ratio f32)

    i64.const 40
    local.set $primer_count
    local.get $primer_count
    i64.const 2
    i64.add
    local.set $primer_count
    f32.const 0.25
    local.set $primer_ratio
    local.get $primer_ratio
    f32.const 2.0
    f32.mul
    local.set $primer_ratio
    local.get $primer_count
    call $print_i64
    local.get $primer_ratio
    call $print_f32
  )
  (export "main" (func $main))
)
