(module
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
        unreachable
      end
      i32.const 32
      i64.load
      i64.const 2
      i64.ge_s
      if
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
