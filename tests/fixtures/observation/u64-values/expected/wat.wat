(module
  (import "primer" "print_u64" (func $print_u64 (param i64)))
  (import "primer" "print_bool" (func $print_bool (param i32)))
  (import "primer" "print_i64" (func $print_i64 (param i64)))
  (import "primer" "print_f32" (func $print_f32 (param f32)))
  (import "primer" "print_f64" (func $print_f64 (param f64)))

  (func $primer_convert_i64_u64 (param $value i64) (result i64)
    (local $result i64) (local $number f64)
    local.get $value
    i64.const 0
    i64.lt_s
    if
      unreachable
    end
    local.get $value
    local.set $result
    local.get $result
  )
  (func $primer_u64_div (param $left i64) (param $right i64) (result i64)
    local.get $right
    i64.eqz
    if
      unreachable
    end
    local.get $left
    local.get $right
    i64.div_u
  )

  (func $primer_u64_shr (param $left i64) (param $right i64) (result i64)
    local.get $right
    i64.const 64
    i64.ge_u
    if
      unreachable
    end
    local.get $left
    local.get $right
    i64.shr_u
  )

  (func $main
    (local $primer_maximum i64)

    i64.const -1
    local.set $primer_maximum
    local.get $primer_maximum
    call $print_u64
    local.get $primer_maximum
    i64.const 0
    i64.gt_u
    call $print_bool
    local.get $primer_maximum
    i64.const 2
    call $primer_u64_div
    call $print_u64
    local.get $primer_maximum
    i64.const 63
    call $primer_u64_shr
    call $print_u64
    i64.const 42
    call $primer_convert_i64_u64
    call $print_u64
  )
  (export "main" (func $main))
)
