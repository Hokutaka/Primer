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

The current fixed-array feature supports:

- grouping a known number of values of one type;
- using a fixed array as a product-type field;
- nesting fixed arrays directly;
- reading one element with an `i64` index;
- copying a complete array into another binding;
- reassigning a complete array to a compatible `mut` binding;
- updating one element or a nested element through a `mut` binding;
- using arrays as function parameters and results;
- aggregation and linear search with loops.

Dynamic lengths are outside the current scope.

## Design decisions

### Length is part of the type

`[i64; 3]` and `[i64; 4]` are different types. Storage size is therefore known during compilation, and assigning arrays with different lengths is rejected before execution.

### Arrays are values

Putting an array into another binding copies the whole value. Two bindings do not silently share one hidden mutable region.

This is the same language-level rule as named product types. Physical instructions and memory copies differ by backend without changing Primer semantics.

Updating an element of a `mut` array changes only the value held by that binding. An array copied to another binding before the update remains unchanged. Element assignment does not introduce shared mutable storage.

### Fixed-size values compose

A named product type may be an array element, and a fixed array may be a field of a product type.

```primer
type Point {
    x: i64,
    y: i64,
}

type Path {
    points: [Point; 4],
}
```

Arrays and product types still copy as independent values when combined. The frontend rejects a type such as `type Node { children: [Node; 1], }` because its size would be infinite even though the cycle passes through an array.

Fixed arrays may also be nested directly.

```primer
matrix: [[i64; 3]; 2] = [[1, 2, 3], [4, 5, 6]];
print(matrix[1][2]);
```

An inner array is also one value. Copying or reassigning the complete array copies every level. In `matrix[row][column]`, the outer and inner indices are checked separately.

### Every index is checked

Valid indices range from `0` through `length - 1`. Negative indices and indices greater than or equal to `length` are out of bounds.

The Primer VM and every generated route perform the check: C, LLVM IR, QBE IR, WebAssembly Text, and Windows x86-64 assembly. Silently removing the check as an optimization would change current language semantics.

For element assignment, indices are evaluated one at a time from left to right and each level is bounds-checked immediately. The right-hand side is evaluated only after every index is valid, followed by one write. A failed check leaves the array unchanged and does not evaluate the right-hand side.

## Observable information

| Stage | Preserved information |
| --- | --- |
| AST | Element type syntax, length, elements, index expressions, assignment root and index path, and spans |
| Primer IR | Resolved `[element; length]`, `array[...]`, `index(...)`, and typed assignment targets |
| Bytecode | `array.new`, `array.get`, `array.check`, `array.assign`, and instruction origin |
| Primer VM | Array value, element type, length, failing index, and instruction position |
| Backend IR | Recursive types, placement, copies, each bounds check, element-address calculation, and load, store, or aggregate copy |
| Artifact | Backend-specific array representation and executable bounds checks |

Length remains explicit type information instead of being inferred from hidden runtime metadata. Every stage can therefore answer how many elements it is handling.

## Backend representation

| Backend | Array | Bounds check |
| --- | --- | --- |
| C | A dedicated `struct` containing a C array | `primer_array_get_*` / `primer_array_at_*` helpers per used type and length |
| LLVM IR | `[N x element]` | Internal get/set helpers per used type and length; failure calls `llvm.trap` |
| QBE IR | Stack storage with 8-byte scalar units and product stride derived from fields | Comparisons and branches; failure calls `abort` |
| WebAssembly Text | Linear memory with 8-byte scalar units and product stride derived from fields | `i64.lt_s` / `i64.ge_s`; failure executes `unreachable` |
| Windows x86-64 | One stack slot per scalar and multiple field-derived slots per product value | Negative and upper-bound comparisons; failure executes `ud2` |
| Primer bytecode | A typed array value | The VM checks `array.get` and `array.check` |

QBE, WebAssembly, and Windows x86-64 currently reserve 8-byte units even for 4-byte scalar values. A product or array element uses the storage required by the complete value as its stride. This simple, observable layout is a backend-lowering choice, not part of the Primer type meaning.

## Security boundary

Observable array types, lengths, binding IDs, instruction numbers, and memory addresses do not grant authority to modify a running array. Observation remains separate from external interference.

Bounds checks stop execution before an invalid memory access. Arrays do not, however, hide secrets. Elements may be visible in artifacts and observations. The current language specification provides no mechanism for hiding secret values.

## Current limits

- Element types are `bool`, `i64`, `f32`, `f64`, named product types, or fixed arrays.
- Length is a positive integer.
- Empty array literals are unavailable.
- Whole-array comparison and `print` are unavailable.

Unsupported forms are diagnosed in the frontend instead of acquiring different behavior in different backends.
