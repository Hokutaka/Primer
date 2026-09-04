# Primer examples

[日本語](README.md)

This directory contains programs that can be read and executed with the current Primer language. Each example demonstrates a different piece of syntax or method of computation in a small program.

## Run all examples

From the repository root, run the following command to display each example's name, output, status, and a final summary:

```powershell
.\scripts\run-examples.ps1
```

Use `-Pattern "matrix*.prim"` to select examples. Use `-SkipBuild` to reuse an already built Primer executable.

## Basics

| Example | Demonstrates |
| --- | --- |
| [hello.prim](hello.prim) | a first example: name two integers, add them, and show the result with `print` |
| [floating_point.prim](floating_point.prim) | precision differences between `f32` and `f64`, and type inference with `infer` |
| [integer_limits.prim](integer_limits.prim) | minimum and maximum `i64` values and a check before addition overflows |
| [boolean_comparisons.prim](boolean_comparisons.prim) | booleans and comparisons |
| [conditional.prim](conditional.prim) | `if` / `else` and scope |
| [loop_control.prim](loop_control.prim) | `while`, `break`, and `continue` |
| [for_sum.prim](for_sum.prim) | `for` and assignment as its start statement |
| [product-point.prim](product-point.prim) | named product types, defaults, and field access |
| [functions.prim](functions.prim) | typed functions, parameters, results, and `void` functions |
| [function_values.prim](function_values.prim) | passing product types and nested fixed arrays through functions as values |
| [fixed_arrays.prim](fixed_arrays.prim) | fixed arrays, indexing, and array value copies |
| [bubble_sort.prim](bubble_sort.prim) | element updates in a `mut` fixed array |
| [product_arrays.prim](product_arrays.prim) | point arrays, indexed product values, and array value copies |
| [matrix_vector_product.prim](matrix_vector_product.prim) | nested fixed arrays and two-dimensional indexing |
| [matrix_composition.prim](matrix_composition.prim) | numerical computation passing a product type with nested arrays through functions |

## Numerical computation

| Example | Demonstrates |
| --- | --- |
| [square_root.prim](square_root.prim) | unrolled square-root approximation steps |
| [while_square_root.prim](while_square_root.prim) | repeated square-root approximation with `while` |
| [logistic_map.prim](logistic_map.prim) | result differences between `f32` and `f64` computation |
| [heat_diffusion.prim](heat_diffusion.prim) | four steps of heat diffusion along a rod, computing new temperatures from the previous array |
| [linear_regression.prim](linear_regression.prim) | learning a line from five points while observing slope, intercept, and loss |

## Algorithms

| Example | Demonstrates |
| --- | --- |
| [euclidean_gcd.prim](euclidean_gcd.prim) | greatest common divisor with the Euclidean algorithm |
| [fibonacci.prim](fibonacci.prim) | the Fibonacci sequence and update order for multiple values |
| [factorial.prim](factorial.prim) | factorial with `for` |
| [collatz.prim](collatz.prim) | Collatz steps and conditional state transitions |
| [prime_check.prim](prime_check.prim) | trial-division primality testing and early exit |
| [integer_square_root.prim](integer_square_root.prim) | integer square root with binary search |
| [exponentiation_by_squaring.prim](exponentiation_by_squaring.prim) | exponentiation by squaring |
| [pythagorean_triples.prim](pythagorean_triples.prim) | Pythagorean triples with nested `for` loops |
| [fixed_arrays.prim](fixed_arrays.prim) | array summation and linear search |
| [bubble_sort.prim](bubble_sort.prim) | in-place bubble sort by swapping elements |
| [product_arrays.prim](product_arrays.prim) | linear search for the nearest point in a point array |
| [matrix_vector_product.prim](matrix_vector_product.prim) | multiplication of a 3-by-3 matrix and a three-element vector |
| [matrix_composition.prim](matrix_composition.prim) | composition of 2-by-2 matrices followed by a vector transformation |
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

These examples are programs expressible with numbers, booleans, bindings, functions, conditionals, loops, named product types, and fixed arrays.

Elements of a `mut` array can be assigned directly, so in-place sorting and array-updating dynamic programming are expressible. Strings, recursion, and dynamically sized collections are not available yet.

`xor_neural_network.prim` demonstrates inference with predetermined weights. `linear_regression.prim` learns a line's slope and intercept from data using gradient descent. Training the XOR neural network itself is not included.
