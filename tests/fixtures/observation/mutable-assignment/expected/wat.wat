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

  (func $main
    (local $primer_count i64)
    (local $primer_ratio f32)

    i64.const 40
    local.set $primer_count
    local.get $primer_count
    i64.const 2
    call $primer_i64_add
    local.set $primer_count
    f32.const 0.25
    local.set $primer_ratio
    local.get $primer_ratio
    f32.const 2.0
    f32.mul
    local.set $primer_ratio
    local.get $primer_count
    call $print_i64
    local.get $primer_ratio
    call $print_f32
  )
  (export "main" (func $main))
)
