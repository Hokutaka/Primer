# Fixed array design

[日本語](fixed-arrays.ja.md)

Status: Implemented

This document records design decisions and observable information for Primer fixed arrays. Current syntax is defined by the [language reference](../reference/language.en.md).

## What is available

```primer
values: [i64; 4] = [2, 4, 6, 8];
print(values[2]);
```

`[i64; 4]` is an array containing four `i64` values. `values[2]` reads the third value because indexing starts at zero.

The initial array feature supports:

- grouping a known number of values of one scalar type;
- reading one element with an `i64` index;
- copying a complete array into another binding;
- reassigning a complete array to a compatible `mut` binding;
- aggregation and linear search with loops.

Direct element assignment, dynamic lengths, nested arrays, and arrays crossing function boundaries are outside the current scope.

## Design decisions

### Length is part of the type

`[i64; 3]` and `[i64; 4]` are different types. Storage size is therefore known during compilation, and assigning arrays with different lengths is rejected before execution.

### Arrays are values

Putting an array into another binding copies the whole value. Two bindings do not silently share one hidden mutable region.

This is the same language-level rule as named product types. Physical instructions and memory copies differ by backend without changing Primer semantics.

### Every index is checked

Valid indices range from `0` through `length - 1`. Negative indices and indices greater than or equal to `length` are out of bounds.

The Primer VM and every generated route perform the check: C, LLVM IR, QBE IR, WebAssembly Text, and Windows x86-64 assembly. Silently removing the check as an optimization would change current language semantics.

## Observable information

| Stage | Preserved information |
| --- | --- |
| AST | Element type name, length, elements, index expression, and spans |
| Primer IR | Resolved `[element; length]`, `array[...]`, and `index(...)` |
| Bytecode | `array.new element length`, `array.get element length`, and instruction origin |
| Primer VM | Array value, element type, length, failing index, and instruction position |
| Backend IR | Placement, copies, bounds checks, element-address calculation, and load |
| Artifact | Backend-specific array representation and executable bounds checks |

Length remains explicit type information instead of being inferred from hidden runtime metadata. Every stage can therefore answer how many elements it is handling.

## Backend representation

| Backend | Array | Bounds check |
| --- | --- | --- |
| C | A dedicated `struct` containing a C array | One `primer_array_get_*` helper per element type and length |
| LLVM IR | `[N x element]` | An internal helper per type and length; failure calls `llvm.trap` |
| QBE IR | Stack storage with an 8-byte stride | Comparisons and branches; failure calls `abort` |
| WebAssembly Text | Linear memory with an 8-byte stride | `i64.lt_s` / `i64.ge_s`; failure executes `unreachable` |
| Windows x86-64 | Stack slots with an 8-byte stride | Negative and upper-bound comparisons; failure executes `ud2` |
| Primer bytecode | A typed array value | The VM checks `array.get` |

QBE, WebAssembly, and Windows x86-64 currently use an 8-byte stride even for 4-byte scalar values. This is a simple and observable initial layout. It is a backend-lowering choice, not part of the Primer type meaning.

## Security boundary

Observable array types, lengths, binding IDs, instruction numbers, and memory addresses do not grant authority to modify a running array. Observation remains separate from external interference.

Bounds checks stop execution before an invalid memory access. Arrays do not, however, hide secrets. Elements may be visible in artifacts and observations. Rules for secret values belong to the separate `Secret` design.

## Current limits

- Element types are limited to `bool`, `i64`, `f32`, and `f64`.
- Length is a positive integer.
- Empty array literals are unavailable.
- Direct element assignment is unavailable.
- Nested arrays and array fields in product types are unavailable.
- Array parameters and results are unavailable.
- Whole-array comparison and `print` are unavailable.

Unsupported forms are diagnosed in the frontend instead of acquiring different behavior in different backends.
