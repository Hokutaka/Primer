(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (memory 1)

  (func $main
    i32.const 48
    i64.const 2
    i64.store
    i32.const 56
    i64.const 4
    i64.store
    i32.const 64
    i64.const 6
    i64.store
    i32.const 0
    i32.const 48
    i64.load
    i64.store
    i32.const 8
    i32.const 56
    i64.load
    i64.store
    i32.const 16
    i32.const 64
    i64.load
    i64.store
    i32.const 24
    i32.const 0
    i64.load
    i64.store
    i32.const 32
    i32.const 8
    i64.load
    i64.store
    i32.const 40
    i32.const 16
    i64.load
    i64.store
    i32.const 72
    i64.const 1
    i64.store
    i32.const 80
    i64.const 3
    i64.store
    i32.const 88
    i64.const 5
    i64.store
    i32.const 0
    i32.const 72
    i64.load
    i64.store
    i32.const 8
    i32.const 80
    i64.load
    i64.store
    i32.const 16
    i32.const 88
    i64.load
    i64.store
    i32.const 96
    i64.const 2
    i64.store
    i32.const 96
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 96
    i64.load
    i64.const 3
    i64.ge_s
    if
      unreachable
    end
    i32.const 24
    i32.const 96
    i64.load
    i32.wrap_i64
    i32.const 8
    i32.mul
    i32.add
    i64.load
    call $print_i64
    i32.const 104
    i64.const 1
    i64.store
    i32.const 104
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 104
    i64.load
    i64.const 3
    i64.ge_s
    if
      unreachable
    end
    i32.const 0
    i32.const 104
    i64.load
    i32.wrap_i64
    i32.const 8
    i32.mul
    i32.add
    i64.load
    call $print_i64
  )
  (export "main" (func $main))
)
