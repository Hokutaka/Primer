(module
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $main
    (local $primer_value i64)
    (local $primer_value_1 i32)

    i64.const 1
    local.set $primer_value
    local.get $primer_value
    i64.const 2
    i64.lt_s
    if
      i64.const 42
      local.set $primer_value
      i32.const 1
      local.set $primer_value_1
      local.get $primer_value_1
      call $print_bool
    else
      i64.const 0
      i64.const 1
      i64.sub
      local.set $primer_value
    end
    local.get $primer_value
    call $print_i64
  )
  (export "main" (func $main))
)
