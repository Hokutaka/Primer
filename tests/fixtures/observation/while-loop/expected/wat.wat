(module
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $main
    (local $primer_count i64)
    (local $primer_sum i64)
    (local $primer_marker i32)

    i64.const 0
    local.set $primer_count
    i64.const 0
    local.set $primer_sum
    block
      loop
        local.get $primer_count
        i64.const 4
        i64.lt_s
        i32.eqz
        br_if 1
        local.get $primer_sum
        local.get $primer_count
        i64.add
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
        i64.add
        local.set $primer_count
        br 0
      end
    end
    local.get $primer_sum
    call $print_i64
  )
  (export "main" (func $main))
)
