(module
  (import "primer" "write_error_byte" (func $write_error_byte (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $primer_i64_add_n23_b177_188 (param $left i64) (param $right i64) (result i64)
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
      i32.const 105
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 103
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 118
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 102
      call $write_error_byte
      i32.const 108
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 119
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
      i32.const 51
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
      i32.const 55
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 49
      call $write_error_byte
      i32.const 56
      call $write_error_byte
      i32.const 56
      call $write_error_byte
      i32.const 10
      call $write_error_byte
      unreachable
    end
    local.get $result
  )

  (func $primer_i64_add_n9_b70_79 (param $left i64) (param $right i64) (result i64)
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
      i32.const 105
      call $write_error_byte
      i32.const 110
      call $write_error_byte
      i32.const 116
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 103
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 45
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 118
      call $write_error_byte
      i32.const 101
      call $write_error_byte
      i32.const 114
      call $write_error_byte
      i32.const 102
      call $write_error_byte
      i32.const 108
      call $write_error_byte
      i32.const 111
      call $write_error_byte
      i32.const 119
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
      i32.const 57
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
      i32.const 55
      call $write_error_byte
      i32.const 48
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 46
      call $write_error_byte
      i32.const 55
      call $write_error_byte
      i32.const 57
      call $write_error_byte
      i32.const 10
      call $write_error_byte
      unreachable
    end
    local.get $result
  )

  (func $main
    (local $primer_value i64)
    (local $primer_sum i64)

    i64.const 0
    local.set $primer_value
    i64.const 0
    local.set $primer_sum
    block $while_end_0
      loop $while_condition_0
        local.get $primer_value
        i64.const 10
        i64.lt_s
        i32.eqz
        br_if $while_end_0
        block $while_continue_0
          local.get $primer_value
          i64.const 1
          call $primer_i64_add_n9_b70_79
          local.set $primer_value
          local.get $primer_value
          i64.const 3
          i64.lt_s
          if
            br $while_continue_0
          end
          local.get $primer_value
          i64.const 5
          i64.gt_s
          if
            br $while_end_0
          end
          local.get $primer_sum
          local.get $primer_value
          call $primer_i64_add_n23_b177_188
          local.set $primer_sum
        end
        br $while_condition_0
      end
    end
    local.get $primer_sum
    call $print_i64
    local.get $primer_value
    call $print_i64
  )
  (export "main" (func $main))
)
