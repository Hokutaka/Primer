# Primer examples

[日本語](README.md)

This directory contains programs that can be read and executed with the current Primer language. Each example demonstrates a different piece of syntax or method of computation in a small program.

## Basics

| Example | Demonstrates |
| --- | --- |
| [hello.prim](hello.prim) | bindings, arithmetic, and `print` |
| [floating_point.prim](floating_point.prim) | `i64`, `f32`, and `f64` |
| [boolean_comparisons.prim](boolean_comparisons.prim) | booleans and comparisons |
| [conditional.prim](conditional.prim) | `if` / `else` and scope |
| [loop_control.prim](loop_control.prim) | `while`, `break`, and `continue` |
| [for_sum.prim](for_sum.prim) | `for` and assignment as its start statement |

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

## Current scope

These examples use algorithms expressible with numbers, booleans, bindings, conditionals, and loops.

Primer does not yet have arrays, strings, functions, recursion, or user-defined types. Sorting, array search, tree and graph traversal, and general dynamic programming therefore cannot yet be expressed naturally. Corresponding examples can be added as those language features arrive.
