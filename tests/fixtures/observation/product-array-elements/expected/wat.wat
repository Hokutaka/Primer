(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (memory 1)

  (func $main
    i32.const 96
    i64.const 1
    i64.store
    i32.const 104
    i64.const 2
    i64.store
    i32.const 64
    i32.const 96
    i64.load
    i64.store
    i32.const 72
    i32.const 104
    i64.load
    i64.store
    i32.const 112
    i64.const 3
    i64.store
    i32.const 120
    i64.const 4
    i64.store
    i32.const 80
    i32.const 112
    i64.load
    i64.store
    i32.const 88
    i32.const 120
    i64.load
    i64.store
    i32.const 0
    i32.const 64
    i64.load
    i64.store
    i32.const 8
    i32.const 72
    i64.load
    i64.store
    i32.const 16
    i32.const 80
    i64.load
    i64.store
    i32.const 24
    i32.const 88
    i64.load
    i64.store
    i32.const 32
    i32.const 0
    i64.load
    i64.store
    i32.const 40
    i32.const 8
    i64.load
    i64.store
    i32.const 48
    i32.const 16
    i64.load
    i64.store
    i32.const 56
    i32.const 24
    i64.load
    i64.store
    i32.const 160
    i64.const 5
    i64.store
    i32.const 168
    i64.const 6
    i64.store
    i32.const 128
    i32.const 160
    i64.load
    i64.store
    i32.const 136
    i32.const 168
    i64.load
    i64.store
    i32.const 176
    i64.const 7
    i64.store
    i32.const 184
    i64.const 8
    i64.store
    i32.const 144
    i32.const 176
    i64.load
    i64.store
    i32.const 152
    i32.const 184
    i64.load
    i64.store
    i32.const 0
    i32.const 128
    i64.load
    i64.store
    i32.const 8
    i32.const 136
    i64.load
    i64.store
    i32.const 16
    i32.const 144
    i64.load
    i64.store
    i32.const 24
    i32.const 152
    i64.load
    i64.store
    i32.const 192
    i64.const 1
    i64.store
    i32.const 192
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 192
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 200
    i32.const 32
    i32.const 192
    i64.load
    i32.wrap_i64
    i32.const 16
    i32.mul
    i32.add
    i32.store
    i32.const 200
    i32.load
    i64.load
    call $print_i64
    i32.const 204
    i64.const 0
    i64.store
    i32.const 204
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 204
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 212
    i32.const 0
    i32.const 204
    i64.load
    i32.wrap_i64
    i32.const 16
    i32.mul
    i32.add
    i32.store
    i32.const 216
    i32.const 212
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 216
    i32.load
    i64.load
    call $print_i64
  )
  (export "main" (func $main))
)
