# Primer examples

[日本語](README.md)

This directory contains programs that can be read and executed with the current Primer language. Each example demonstrates a different piece of syntax or method of computation in a small program.

## Run all examples

From the repository root, run the following command to display each example's name, output, status, and a final summary:

```powershell
.\scripts\run-examples.ps1
```

Use `-Pattern "matrix*.prim"` to select examples. Use `-SkipBuild` to reuse an already built Primer executable.

For WSL / Bash, use the following commands. WSL also needs its own Rust development environment.

```bash
bash scripts/run-examples.sh
bash scripts/run-examples.sh --pattern 'matrix*.prim' --skip-build
```

The `.sh` script defaults to `target/unix/debug/primer`, separate from Windows artifacts. It respects `CARGO_TARGET_DIR` when set. Use `--skip-build` only after building into the same output directory.

The runner checks each example's exit status. Use `cargo test --test examples` to compare expected output, or `bash scripts/test.sh` to run fmt, Clippy, and all test targets together.

## Basics

| Example | Demonstrates |
| --- | --- |
| [hello.prim](hello.prim) | a first example: name two integers, add them, and show the result with `print` |
| [string_values.prim](string_values.prim) | Japanese text, equality, line breaks, and preserved string copies after reassignment |
| [floating_point.prim](floating_point.prim) | precision differences between `f32` and `f64`, and type inference with `infer` |
| [small_values.prim](small_values.prim) | observing small numbers in scientific notation and distinguishing display from arithmetic rounding |
| [integer_limits.prim](integer_limits.prim) | minimum and maximum `i64` values and a check before addition overflows |
| [integer_conversions.prim](integer_conversions.prim) | widening `i32` to `i64` with two equivalent conversion spellings |
| [bit_flags.prim](bit_flags.prim) | setting, clearing, toggling, and testing eight independent bit switches |
| [boolean_comparisons.prim](boolean_comparisons.prim) | booleans and comparisons |
| [short_circuit.prim](short_circuit.prim) | combining conditions with `&&`/`\|\|` to skip unnecessary division, indexing, and function calls |
| [conditional.prim](conditional.prim) | `if` / `else` and scope |
| [loop_control.prim](loop_control.prim) | `while`, `break`, and `continue` |
| [for_sum.prim](for_sum.prim) | `for` and assignment as its start statement |
| [functions.prim](functions.prim) | typed functions, parameters, results, and `void` functions |

## Data structures

These examples show how to group, access, and pass multiple values. They use structs (named product types) and fixed arrays, which are currently supported.

| Example | Demonstrates |
| --- | --- |
| [ring_buffer.prim](ring_buffer.prim) | cycling a storage position with `%` to keep the latest four values and their average |
| [string_lookup.prim](string_lookup.prim) | linear search by a string key in an array of structs, returning display text or a default |
| [product-point.prim](product-point.prim) | grouping point coordinates in a struct, with field defaults and access |
| [fixed_arrays.prim](fixed_arrays.prim) | indexing, summation, and linear search in fixed arrays, and independent values after copying |
| [product_arrays.prim](product_arrays.prim) | arrays of structs, nearest-point search, and array value copies |
| [function_values.prim](function_values.prim) | passing and returning structs and nested fixed arrays as values |

## Numerical computation

| Example | Demonstrates |
| --- | --- |
| [measurement_statistics.prim](measurement_statistics.prim) | computing a fractional mean and variance from integer samples, then storing them exactly as `f32` |
| [normalized_histogram.prim](normalized_histogram.prim) | converting integer counts to probabilities and recovering the original counts |
| [square_root.prim](square_root.prim) | unrolled square-root approximation steps |
| [while_square_root.prim](while_square_root.prim) | repeated square-root approximation with `while` |
| [logistic_map.prim](logistic_map.prim) | result differences between `f32` and `f64` computation |
| [matrix_vector_product.prim](matrix_vector_product.prim) | multiplying a 3-by-3 matrix by a three-element vector using nested fixed arrays |
| [matrix_composition.prim](matrix_composition.prim) | passing structs and nested arrays through functions to compose 2-by-2 matrices and transform a vector |
| [population_statistics.prim](population_statistics.prim) | widening large `u32` values to `i64` for aggregation, returning average and maximum in a struct |
| [heat_diffusion.prim](heat_diffusion.prim) | four steps of heat diffusion along a rod, computing new temperatures from the previous array |
| [linear_regression.prim](linear_regression.prim) | learning a line from five points while observing slope, intercept, and loss |

## Algorithms

| Example | Demonstrates |
| --- | --- |
| [color_blending.prim](color_blending.prim) | widening `u8` color channels to `u16` before adding and averaging them |
| [sensor_calibration.prim](sensor_calibration.prim) | correcting `i16` readings with an `i8` offset, then aggregating in `i32` |
| [maximum_subarray.prim](maximum_subarray.prim) | maximum sum of a contiguous `i32` subarray, converting a `u32` position to an index |
| [subset_sum_bits.prim](subset_sum_bits.prim) | finding reachable sums together using shifts and bitwise OR |
| [euclidean_gcd.prim](euclidean_gcd.prim) | greatest common divisor with the Euclidean algorithm |
| [fibonacci.prim](fibonacci.prim) | the Fibonacci sequence and update order for multiple values |
| [factorial.prim](factorial.prim) | factorial with `for` |
| [collatz.prim](collatz.prim) | Collatz steps and conditional state transitions |
| [prime_check.prim](prime_check.prim) | trial-division primality testing and early exit |
| [integer_square_root.prim](integer_square_root.prim) | integer square root with binary search |
| [exponentiation_by_squaring.prim](exponentiation_by_squaring.prim) | exponentiation by squaring |
| [pythagorean_triples.prim](pythagorean_triples.prim) | Pythagorean triples with nested `for` loops |
| [bubble_sort.prim](bubble_sort.prim) | in-place bubble sort by swapping elements in a `mut` fixed array |
| [xor_neural_network.prim](xor_neural_network.prim) | XOR inference with a tiny neural network and fixed-array weights |
| [coin_change.prim](coin_change.prim) | dynamic programming for minimum coin counts, followed by reconstruction of the chosen coins |
| [shortest_paths.prim](shortest_paths.prim) | all-pairs shortest paths by gradually allowing more intermediate towns |

## Reading intermediate results

The new examples print intermediate values as well as final answers. Japanese comments in each file describe the output order.

- `coin_change.prim`: minimum counts for amounts 1 through 6, then the selected coins, 3 and 3.
- `shortest_paths.prim`: changes in the distance from town 0 to town 3, then the 4-by-4 distance table in row order. `-1` means unreachable.
- `heat_diffusion.prim`: five temperatures per step for four steps, then the saved initial center temperature.
- `linear_regression.prim`: initial loss; epoch, slope, intercept, and loss every ten epochs; then a prediction for a new input of 3.

Try changing `rate` (the size of each learning step) or the iteration count in the regression example and compare the loss. To inspect the representations at different stages, run:

```powershell
cargo run --quiet -- run examples/linear_regression.prim
cargo run --quiet -- emit-ir examples/linear_regression.prim
cargo run --quiet -- emit-bytecode examples/linear_regression.prim
cargo run --quiet -- emit-c examples/linear_regression.prim
```

`integer_limits.prim` succeeds by default. Uncomment an expression at the end to observe an overflow stop and its diagnostic location.

## Current scope

These examples are programs expressible with numbers, booleans, strings, bindings, functions, conditionals, loops, named product types, and fixed arrays.

Elements of a `mut` array can be assigned directly, so in-place sorting and array-updating dynamic programming are expressible. Recursion and dynamically sized collections are not available yet.

The two string examples support every output route. LLVM and QBE require explicit targets: QBE is validated on Linux x86-64, direct assembly on Windows x64, and WAT in a WebAssembly environment providing the output host functions. `emit-ir` and `emit-bytecode` also expose type and content transformations.

Run QBE, WAT, and direct assembly comparisons with `cargo test --test string_routes`. [String design](../docs/design/strings.en.md#validation-scope) documents tool selection and validation scope.

For example, emit and compile the string-key lookup:

```sh
cargo run --quiet -- emit-c examples/string_lookup.prim -o target/string_lookup.c
clang -std=c11 target/string_lookup.c -o target/string_lookup
```

Run the executable with `./target/string_lookup` in Bash or `.\target\string_lookup.exe` on Windows. An external C compiler is required.

For LLVM, use the Windows/Linux command examples in the [CLI reference](../docs/reference/cli.en.md#llvm-target-selection). `cargo test --test llvm_strings` compares VM, generated C, and generated LLVM output byte-for-byte. Setting `PRIMER_TEST_LLVM_CLANG` and `PRIMER_TEST_CC` makes unavailable selected compilers a test failure.

`cargo test --test c_strings` runs generated C with and without optimization and compares it with the VM. Execution comparisons skip when the default C compiler is unavailable; setting `PRIMER_TEST_CC` makes the selected compiler mandatory. CI requires Clang and also checks with AddressSanitizer and UndefinedBehaviorSanitizer.

`xor_neural_network.prim` demonstrates inference with predetermined weights. `linear_regression.prim` learns a line's slope and intercept from data using gradient descent. Training the XOR neural network itself is not included.

### Following string origins

`string_origins.prim` demonstrates calls, string content equality, and short-circuit evaluation. Escaped output is `日本語\0\ntrue\nfalse\n`; `skipped` is never printed. Compare `emit-ir` with `emit-llvm --annotate-origins` to follow equality node #7 and short-circuit node #14 to calls and branches. See the [CLI walkthrough](../docs/reference/cli.en.md#following-llvm-origins).

### Inspecting string byte lengths

Run `cargo run -- run examples/string_byte_length.prim` and inspect `emit-ir` or `emit-llvm --target x86_64-unknown-linux-gnu --annotate-origins` for the same input.

Output lines are `0, 9, 3, 2, 3, 4, 7, 3, 9, left, right, 9, false, false, 6, 10`, each followed by LF. The example exercises UTF-8 lengths, saved copies, calls, arrays, defaults, and evaluation order. Each of `left` and `right` is printed once; `skipped` is never printed. C, LLVM, QBE, WAT, and direct assembly are also executed against known expected bytes. See the [small observation fixture](../tests/fixtures/observation/string-byte-length/) for representations in every route.
