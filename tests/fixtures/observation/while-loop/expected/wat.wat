(module
  (import "primer" "print_bool" (func $print_bool (param i32)))
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

  (func $main
    (local $primer_count i64)
    (local $primer_sum i64)
    (local $primer_marker i32)

    i64.const 0
    local.set $primer_count
    i64.const 0
    local.set $primer_sum
    block $while_end_0
      loop $while_condition_0
        local.get $primer_count
        i64.const 4
        i64.lt_s
        i32.eqz
        br_if $while_end_0
        block $while_continue_0
          local.get $primer_sum
          local.get $primer_count
          call $primer_i64_add
          local.set $primer_sum
          local.get $primer_count
          i64.const 2
          i64.eq
          if
            i32.const 1
            local.set $primer_marker
            local.get $primer_marker
            call $print_bool
          end
          local.get $primer_count
          i64.const 1
          call $primer_i64_add
          local.set $primer_count
        end
        br $while_condition_0
      end
    end
    local.get $primer_sum
    call $print_i64
  )
  (export "main" (func $main))
)
