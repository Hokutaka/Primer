(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

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
          i64.add
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
          i64.add
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
