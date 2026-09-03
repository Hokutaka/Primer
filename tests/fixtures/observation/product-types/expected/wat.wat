(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (memory 1)

  (func $main
    i32.const 72
    f64.const 2.0
    f64.store
    i32.const 64
    f64.const 0.0
    f64.store
    i32.const 0
    i32.const 64
    f64.load
    f64.store
    i32.const 8
    i32.const 72
    f64.load
    f64.store
    i32.const 16
    i32.const 0
    f64.load
    f64.store
    i32.const 24
    i32.const 8
    f64.load
    f64.store
    i32.const 80
    f64.const 4.0
    f64.store
    i32.const 88
    f64.const 5.0
    f64.store
    i32.const 0
    i32.const 80
    f64.load
    f64.store
    i32.const 8
    i32.const 88
    f64.load
    f64.store
    i32.const 96
    i32.const 16
    f64.load
    f64.store
    i32.const 104
    i32.const 24
    f64.load
    f64.store
    i32.const 112
    i32.const 0
    f64.load
    f64.store
    i32.const 120
    i32.const 8
    f64.load
    f64.store
    i32.const 32
    i32.const 96
    f64.load
    f64.store
    i32.const 40
    i32.const 104
    f64.load
    f64.store
    i32.const 48
    i32.const 112
    f64.load
    f64.store
    i32.const 56
    i32.const 120
    f64.load
    f64.store
    i32.const 16
    f64.load
    call $print_f64
    i32.const 24
    f64.load
    call $print_f64
    i32.const 40
    f64.load
    call $print_f64
    i32.const 48
    f64.load
    call $print_f64
  )
  (export "main" (func $main))
)
