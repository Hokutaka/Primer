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

  (memory 1)

  (func $primer_fn_move_x_0 (param $primer_abi.result i32) (param $primer_point i32) (param $primer_amount i64)
    i32.const 0
    local.get $primer_abi.result
    i32.store
    i32.const 4
    local.get $primer_point
    i32.store
    i32.const 8
    i32.const 4
    i32.load
    i64.load
    i64.store
    i32.const 24
    i32.const 4
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 16
    i32.const 24
    i32.load
    i64.load
    i64.store
    i32.const 28
    i32.const 8
    i64.load
    local.get $primer_amount
    call $primer_i64_add
    i64.store
    i32.const 36
    i32.const 16
    i64.load
    i64.store
    i32.const 0
    i32.load
    i32.const 28
    i64.load
    i64.store
    i32.const 44
    i32.const 0
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 44
    i32.load
    i32.const 36
    i64.load
    i64.store
    return
  )
  (func $primer_fn_move_twice_1 (param $primer_abi.result i32) (param $primer_point i32) (param $primer_amount i64)
    i32.const 48
    local.get $primer_abi.result
    i32.store
    i32.const 52
    local.get $primer_point
    i32.store
    i32.const 56
    i32.const 52
    i32.load
    i64.load
    i64.store
    i32.const 72
    i32.const 52
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 64
    i32.const 72
    i32.load
    i64.load
    i64.store
    i32.const 76
    i32.const 92
    i32.const 56
    local.get $primer_amount
    call $primer_fn_move_x_0
    i32.const 92
    local.get $primer_amount
    call $primer_fn_move_x_0
    i32.const 48
    i32.load
    i32.const 76
    i64.load
    i64.store
    i32.const 108
    i32.const 48
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 108
    i32.load
    i32.const 84
    i64.load
    i64.store
    return
  )
  (func $primer_fn_first_row_2 (param $primer_abi.result i32) (param $primer_matrix i32)
    i32.const 112
    local.get $primer_abi.result
    i32.store
    i32.const 116
    local.get $primer_matrix
    i32.store
    i32.const 120
    i32.const 116
    i32.load
    i64.load
    i64.store
    i32.const 152
    i32.const 116
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 128
    i32.const 152
    i32.load
    i64.load
    i64.store
    i32.const 156
    i32.const 116
    i32.load
    i32.const 16
    i32.add
    i32.store
    i32.const 136
    i32.const 156
    i32.load
    i64.load
    i64.store
    i32.const 160
    i32.const 156
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 144
    i32.const 160
    i32.load
    i64.load
    i64.store
    i32.const 164
    i64.const 0
    i64.store
    i32.const 164
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 164
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 172
    i32.const 120
    i32.const 164
    i64.load
    i32.wrap_i64
    i32.const 16
    i32.mul
    i32.add
    i32.store
    i32.const 112
    i32.load
    i32.const 172
    i32.load
    i64.load
    i64.store
    i32.const 176
    i32.const 172
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 180
    i32.const 112
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 180
    i32.load
    i32.const 176
    i32.load
    i64.load
    i64.store
    return
  )
  (func $primer_fn_duplicate_3 (param $primer_abi.result i32) (param $primer_row i32)
    i32.const 184
    local.get $primer_abi.result
    i32.store
    i32.const 188
    local.get $primer_row
    i32.store
    i32.const 192
    i32.const 188
    i32.load
    i64.load
    i64.store
    i32.const 208
    i32.const 188
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 200
    i32.const 208
    i32.load
    i64.load
    i64.store
    i32.const 212
    i32.const 192
    i64.load
    i64.store
    i32.const 220
    i32.const 200
    i64.load
    i64.store
    i32.const 228
    i32.const 192
    i64.load
    i64.store
    i32.const 236
    i32.const 200
    i64.load
    i64.store
    i32.const 184
    i32.load
    i32.const 212
    i64.load
    i64.store
    i32.const 244
    i32.const 184
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 244
    i32.load
    i32.const 220
    i64.load
    i64.store
    i32.const 248
    i32.const 184
    i32.load
    i32.const 16
    i32.add
    i32.store
    i32.const 248
    i32.load
    i32.const 228
    i64.load
    i64.store
    i32.const 252
    i32.const 248
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 252
    i32.load
    i32.const 236
    i64.load
    i64.store
    return
  )
  (func $primer_fn_duplicate_first_row_4 (param $primer_abi.result i32) (param $primer_matrix i32)
    i32.const 256
    local.get $primer_abi.result
    i32.store
    i32.const 260
    local.get $primer_matrix
    i32.store
    i32.const 264
    i32.const 260
    i32.load
    i64.load
    i64.store
    i32.const 296
    i32.const 260
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 272
    i32.const 296
    i32.load
    i64.load
    i64.store
    i32.const 300
    i32.const 260
    i32.load
    i32.const 16
    i32.add
    i32.store
    i32.const 280
    i32.const 300
    i32.load
    i64.load
    i64.store
    i32.const 304
    i32.const 300
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 288
    i32.const 304
    i32.load
    i64.load
    i64.store
    i32.const 308
    i32.const 340
    i32.const 264
    call $primer_fn_first_row_2
    i32.const 340
    call $primer_fn_duplicate_3
    i32.const 256
    i32.load
    i32.const 308
    i64.load
    i64.store
    i32.const 356
    i32.const 256
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 356
    i32.load
    i32.const 316
    i64.load
    i64.store
    i32.const 360
    i32.const 256
    i32.load
    i32.const 16
    i32.add
    i32.store
    i32.const 360
    i32.load
    i32.const 324
    i64.load
    i64.store
    i32.const 364
    i32.const 360
    i32.load
    i32.const 8
    i32.add
    i32.store
    i32.const 364
    i32.load
    i32.const 332
    i64.load
    i64.store
    return
  )
  (func $main
    i32.const 464
    i64.const 2
    i64.store
    i32.const 472
    i64.const 3
    i64.store
    i32.const 368
    i32.const 464
    i64.load
    i64.store
    i32.const 376
    i32.const 472
    i64.load
    i64.store
    i32.const 480
    i32.const 368
    i64.const 5
    call $primer_fn_move_twice_1
    i32.const 384
    i32.const 480
    i64.load
    i64.store
    i32.const 392
    i32.const 488
    i64.load
    i64.store
    i32.const 528
    i64.const 1
    i64.store
    i32.const 536
    i64.const 2
    i64.store
    i32.const 496
    i32.const 528
    i64.load
    i64.store
    i32.const 504
    i32.const 536
    i64.load
    i64.store
    i32.const 544
    i64.const 3
    i64.store
    i32.const 552
    i64.const 4
    i64.store
    i32.const 512
    i32.const 544
    i64.load
    i64.store
    i32.const 520
    i32.const 552
    i64.load
    i64.store
    i32.const 400
    i32.const 496
    i64.load
    i64.store
    i32.const 408
    i32.const 504
    i64.load
    i64.store
    i32.const 416
    i32.const 512
    i64.load
    i64.store
    i32.const 424
    i32.const 520
    i64.load
    i64.store
    i32.const 560
    i32.const 400
    call $primer_fn_duplicate_first_row_4
    i32.const 432
    i32.const 560
    i64.load
    i64.store
    i32.const 440
    i32.const 568
    i64.load
    i64.store
    i32.const 448
    i32.const 576
    i64.load
    i64.store
    i32.const 456
    i32.const 584
    i64.load
    i64.store
    i32.const 368
    i64.load
    call $print_i64
    i32.const 384
    i64.load
    call $print_i64
    i32.const 392
    i64.load
    call $print_i64
    i32.const 592
    i64.const 1
    i64.store
    i32.const 592
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 592
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 600
    i32.const 400
    i32.const 592
    i64.load
    i32.wrap_i64
    i32.const 16
    i32.mul
    i32.add
    i32.store
    i32.const 604
    i64.const 0
    i64.store
    i32.const 604
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 604
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 600
    i32.load
    i32.const 604
    i64.load
    i32.wrap_i64
    i32.const 8
    i32.mul
    i32.add
    i64.load
    call $print_i64
    i32.const 612
    i64.const 0
    i64.store
    i32.const 612
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 612
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 620
    i32.const 432
    i32.const 612
    i64.load
    i32.wrap_i64
    i32.const 16
    i32.mul
    i32.add
    i32.store
    i32.const 624
    i64.const 1
    i64.store
    i32.const 624
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 624
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 620
    i32.load
    i32.const 624
    i64.load
    i32.wrap_i64
    i32.const 8
    i32.mul
    i32.add
    i64.load
    call $print_i64
    i32.const 632
    i64.const 1
    i64.store
    i32.const 632
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 632
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 640
    i32.const 432
    i32.const 632
    i64.load
    i32.wrap_i64
    i32.const 16
    i32.mul
    i32.add
    i32.store
    i32.const 644
    i64.const 0
    i64.store
    i32.const 644
    i64.load
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    i32.const 644
    i64.load
    i64.const 2
    i64.ge_s
    if
      unreachable
    end
    i32.const 640
    i32.load
    i32.const 644
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
