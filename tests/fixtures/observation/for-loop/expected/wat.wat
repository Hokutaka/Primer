(module
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $main
    (local $primer_sum i64)
    (local $primer_i i64)

    i64.const 0
    local.set $primer_sum
    i64.const 0
    local.set $primer_i
    block $for_end_0
      loop $for_condition_0
        local.get $primer_i
        i64.const 6
        i64.lt_s
        i32.eqz
        br_if $for_end_0
        block $for_continue_0
          local.get $primer_i
          i64.const 2
          i64.lt_s
          if
            br $for_continue_0
          end
          local.get $primer_sum
          local.get $primer_i
          i64.add
          local.set $primer_sum
        end
        local.get $primer_i
        i64.const 1
        i64.add
        local.set $primer_i
        br $for_condition_0
      end
    end
    local.get $primer_sum
    call $print_i64
  )
  (export "main" (func $main))
)
