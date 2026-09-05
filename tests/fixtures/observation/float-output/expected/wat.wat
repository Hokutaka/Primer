(module
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $main
    f32.const 1e-20
    call $print_f32
    f64.const 1e-20
    call $print_f64
    f64.const 1e-20
    f64.const 0.0
    f64.ne
    call $print_bool
    f32.const 1e-45
    call $print_f32
    f64.const 5e-324
    call $print_f64
    f32.const 3.4028234663852886e38
    call $print_f32
    f64.const 1.7976931348623157e308
    call $print_f64
    f32.const 0.0
    f32.neg
    call $print_f32
    f64.const 0.0
    f64.neg
    call $print_f64
    f32.const 0.0
    call $print_f32
    f64.const 0.0
    call $print_f64
    f32.const 0.0001
    call $print_f32
    f64.const 0.0001
    call $print_f64
    f32.const 1e9
    call $print_f32
    f64.const 1e17
    call $print_f64
  )
  (export "main" (func $main))
)
