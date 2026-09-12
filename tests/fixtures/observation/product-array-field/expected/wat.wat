(module
  (import "primer" "write_error_byte" (func $write_error_byte (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (memory 1)

  (func $main
    i32.const 72
    i64.const 1
    i64.store
    i32.const 80
    i64.const 2
    i64.store
    i32.const 88
    i64.const 3
    i64.store
    i32.const 48
    i32.const 72
    i64.load
    i64.store
    i32.const 56
    i32.const 80
    i64.load
    i64.store
    i32.const 64
    i32.const 88
    i64.load
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
    i32.const 120
    i64.const 4
    i64.store
    i32.const 128
    i64.const 5
    i64.store
    i32.const 136
    i64.const 6
    i64.store
    i32.const 96
    i32.const 120
    i64.load
    i64.store
    i32.const 104
    i32.const 128
    i64.load
    i64.store
    i32.const 112
    i32.const 136
    i64.load
    i64.store
    i32.const 0
    i32.const 96
    i64.load
    i64.store
    i32.const 8
    i32.const 104
    i64.load
    i64.store
    i32.const 16
    i32.const 112
    i64.load
    i64.store
    i32.const 144
    i64.const 1
    i64.store
    i32.const 144
    i64.load
    i64.const 0
    i64.lt_s
    if
      ;; primer: runtime-v1 code=array-index-out-of-bounds node=15 bytes=145..161
      i32.const 112
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 58
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 118
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 99
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 97
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 97
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 120
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 102
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 53
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 52
      call $write_error_byte
      i32.const 53
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 54
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 10
      call $write_error_byte
      unreachable
    end
    i32.const 144
    i64.load
    i64.const 3
    i64.ge_s
    if
      ;; primer: runtime-v1 code=array-index-out-of-bounds node=15 bytes=145..161
      i32.const 112
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 58
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 118
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 99
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 97
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 97
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 120
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 102
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 53
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 52
      call $write_error_byte
      i32.const 53
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 54
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 10
      call $write_error_byte
      unreachable
    end
    i32.const 24
    i32.const 144
    i64.load
    i32.wrap_i64
    i32.const 8
    i32.mul
    i32.add
    i64.load
    call $print_i64
    i32.const 152
    i64.const 2
    i64.store
    i32.const 152
    i64.load
    i64.const 0
    i64.lt_s
    if
      ;; primer: runtime-v1 code=array-index-out-of-bounds node=20 bytes=170..185
      i32.const 112
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 58
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 118
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 99
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 97
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 97
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 120
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 102
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 50
      call $write_error_byte
      i32.const 48
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 55
      call $write_error_byte
      i32.const 48
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 56
      call $write_error_byte
      i32.const 53
      call $write_error_byte
      i32.const 10
      call $write_error_byte
      unreachable
    end
    i32.const 152
    i64.load
    i64.const 3
    i64.ge_s
    if
      ;; primer: runtime-v1 code=array-index-out-of-bounds node=20 bytes=170..185
      i32.const 112
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 58
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 109
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 118
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 99
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 97
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 97
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 105
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 120
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 102
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 117
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 100
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 50
      call $write_error_byte
      i32.const 48
      call $write_error_byte
      i32.const 32
      call $write_error_byte
      i32.const 98
      call $write_error_byte
      i32.const 121
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 115
      call $write_error_byte
      i32.const 61
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 55
      call $write_error_byte
      i32.const 48
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 56
      call $write_error_byte
      i32.const 53
      call $write_error_byte
      i32.const 10
      call $write_error_byte
      unreachable
    end
    i32.const 0
    i32.const 152
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
