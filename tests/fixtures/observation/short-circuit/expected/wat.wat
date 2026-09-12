(module
  (import "primer" "write_error_byte" (func $write_error_byte (param i32)))
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (memory 1)

  (func $primer_fn_report_0 (param $primer_value i32) (result i32)
    local.get $primer_value
    call $print_bool
    local.get $primer_value
    return
  )
  (func $main
    (local $primer_index i64)

    i32.const 16
    i64.const 4
    i64.store
    i32.const 24
    i64.const 9
    i64.store
    i32.const 0
    i32.const 16
    i64.load
    i64.store
    i32.const 8
    i32.const 24
    i64.load
    i64.store
    i64.const 2
    local.set $primer_index
    local.get $primer_index
    i64.const 2
    i64.lt_s
    if (result i32)
      i32.const 32
      local.get $primer_index
      i64.store
      i32.const 32
      i64.load
      i64.const 0
      i64.lt_s
      if
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
        i32.const 54
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
        i32.const 51
        call $write_error_byte
        i32.const 52
        call $write_error_byte
        i32.const 46
        call $write_error_byte
        i32.const 46
        call $write_error_byte
        i32.const 49
        call $write_error_byte
        i32.const 52
        call $write_error_byte
        i32.const 55
        call $write_error_byte
        i32.const 10
        call $write_error_byte
        unreachable
      end
      i32.const 32
      i64.load
      i64.const 2
      i64.ge_s
      if
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
        i32.const 54
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
        i32.const 51
        call $write_error_byte
        i32.const 52
        call $write_error_byte
        i32.const 46
        call $write_error_byte
        i32.const 46
        call $write_error_byte
        i32.const 49
        call $write_error_byte
        i32.const 52
        call $write_error_byte
        i32.const 55
        call $write_error_byte
        i32.const 10
        call $write_error_byte
        unreachable
      end
      i32.const 0
      i32.const 32
      i64.load
      i32.wrap_i64
      i32.const 8
      i32.mul
      i32.add
      i64.load
      i64.const 0
      i64.gt_s
    else
      i32.const 0
    end
    call $print_bool
    local.get $primer_index
    i64.const 2
    i64.eq
    if (result i32)
      i32.const 1
    else
      i32.const 0
      call $primer_fn_report_0
    end
    call $print_bool
    i32.const 0
    if (result i32)
      i32.const 1
    else
      i32.const 1
      call $primer_fn_report_0
      if (result i32)
        local.get $primer_index
        i64.const 0
        i64.gt_s
        if (result i32)
          i32.const 1
        else
          i32.const 0
          call $primer_fn_report_0
        end
      else
        i32.const 0
      end
    end
    call $print_bool
  )
  (export "main" (func $main))
)
