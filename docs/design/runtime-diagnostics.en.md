# Common runtime failure records

[日本語](runtime-diagnostics.ja.md)

## Purpose and current scope

A nonzero exit alone cannot distinguish an intended language stop from an access violation or startup failure. Primer compares failure reasons and source locations across routes as part of observability, without exposing state mutation or execution intervention.

| Route | Current runtime diagnostics |
| --- | --- |
| VM | Default human-readable diagnostics and explicit `runtime-v1` format |
| Windows/Linux direct assembly | Language checks write `runtime-v1` to stderr, then terminate with an illegal instruction |
| Internal COFF/ELF encoder | Encodes the same assembly lowering and diagnostics |
| C | Existing diagnostics, including reasons for some u64 checks; common records and source ranges are unimplemented |
| LLVM / QBE / WAT | Existing trap / abort / unreachable; common records are unimplemented |

The first implementation makes the recently added native encoder comparable with the VM. C, LLVM, QBE, and WAT are the next extensions of this contract. This is not complete diagnostic parity across all routes.

## Record

```text
primer: runtime-v1 code=division-by-zero node=1 bytes=6..11
```

This identifies division in `print(1 / 0);`. Each ASCII line contains:

- `code`: A failure reason from the table below.
- `node`: The corresponding Primer IR NodeId. A failure inside a callee identifies the failing expression, not the caller's expression.
- `bytes`: The original UTF-8 source range, zero-based with an exclusive end. These are not line numbers or displayed character counts.

Records exclude paths, source text, and runtime values. Interpret them with the same source and compiler version. NodeIds are not persistent across edits or versions. Native observation manifests also record the source SHA-256 and tools used.

| Code | Meaning |
| --- | --- |
| `integer-overflow` | Addition, subtraction, multiplication, negation, or left shift exceeds the type's range |
| `division-by-zero` | Integer division by zero |
| `division-overflow` | Integer division result exceeds the type's range |
| `remainder-by-zero` | Integer remainder with a zero divisor |
| `invalid-shift-count` | Shift count is negative or at least the type's bit width |
| `integer-conversion-out-of-range` | Explicit integer-to-integer conversion exceeds the range |
| `conversion-out-of-range` | Conversion involving floating point exceeds the range |
| `conversion-inexact` | Conversion changes the value |
| `conversion-not-finite` | NaN or infinity cannot convert to an integer |
| `conversion-nan` | Changing float types cannot preserve NaN |
| `conversion-negative-zero` | Integer conversion cannot preserve negative zero's sign |
| `array-index-out-of-bounds` | Array index is outside its bounds |

For array assignments, the NodeId identifies the assignment and the range identifies the failing `[...]`. Nested indices are checked left to right; failure prevents right-hand-side evaluation. An integer-to-float conversion that rounds beyond a range still reports `conversion-inexact`: it cannot preserve the original integer value.

This format does not classify compilation diagnostics, invalid-bytecode internals, OS resource failures, or output I/O failures as language check failures. The VM uses existing diagnostics for those cases; unexpected generated-code crashes must not receive fabricated common records.

## Usage and retained output

```sh
primer run examples/runtime_failures/function_division.prim --diagnostic-format runtime-v1
```

Ordinary `run` keeps the existing source/bytecode-position diagnostics. `--diagnostic-format runtime-v1` selects common records for language check failures. The VM exits with code 1. `ExecutionError::runtime_failure()` exposes structured reasons and ranges.

Previously executed `print` operations are not rolled back. The VM retains their output in `VmError::output()`, and the CLI writes it to stdout. Native failures flush existing stdout buffers, write the stderr record, and stop with `ud2`. This does not guarantee the display order of merged stdout/stderr. Windows programs containing strings retain their existing binary stdout behavior.

Writing uses the explicitly selected target's Linux `write` or Windows `_write`. Diagnostic comparisons normalize CRLF/LF. If stderr is closed or a write fails partway through, code still attempts the original illegal-instruction stop, but complete diagnostic delivery is not guaranteed. Observation tools reject incomplete records.

## Implementation decisions

Each source-derived failure site has a read-only record. This increases artifact size, but reporting runs only on failure branches, without introducing mutable “current location” state on successful paths. Records require neither dynamic allocation nor externally mutable diagnostic variables. Checks needed to distinguish numeric conversion reasons are always generated as part of preserving operation semantics.

`--annotate-origins` still only controls observational comments and symbols. It does not disable runtime diagnostics. Tests compare instruction/data contents and behavior with and without annotations. Failure ABIs and output functions depend on the explicit compilation target, not the host OS.

## Verification

[Three expected-failure examples](../../examples/runtime_failures/README.en.md) trace overflow, nested array assignment, and division inside a function. They are separate from normal examples and list their intended failure reasons.

`observe-native.cjs --run --expect-trap` runs the VM with common records and compares reason, NodeId, byte range, and prior output with native execution. It also requires SIGILL / Windows illegal-instruction termination. Matching diagnostics with successful termination, unreported crashes, additional errors, and timeouts do not pass. The manifest's `runtimeFailure` stores the matched record.

Tests compare 49 failure cases with known reasons, matching Windows/Linux assembly and internal objects against the VM. They include Unicode, CRLF, short-circuiting, function calls, default-field expressions, conversion boundaries, and retained output. The 50 normal examples and existing test suite also remain part of validation.
