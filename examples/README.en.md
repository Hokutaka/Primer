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
| [hello.prim](hello.prim) | bindings, arithmetic, and `print` |
| [floating_point.prim](floating_point.prim) | `i64`, `f32`, and `f64` |
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

## Current scope

These examples are programs expressible with numbers, booleans, bindings, functions, conditionals, loops, named product types, and fixed arrays.

Elements of a `mut` array can be assigned directly, so in-place sorting and array-updating dynamic programming are expressible. Strings, recursion, and dynamically sized collections are not available yet.

`xor_neural_network.prim` demonstrates inference with predetermined weights. It does not yet train those weights from examples.
