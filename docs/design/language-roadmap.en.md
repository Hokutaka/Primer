# Language capabilities and next steps

[日本語](language-roadmap.ja.md)

As of 2026-09-12, this inventory includes merged PR #50 and runtime diagnostics across all routes. “Current” means implemented; candidates are proposals awaiting design and implementation. This document does not commit to candidate syntax or adoption. See the [language reference](../reference/language.en.md) for the current specification.

## Properties Primer should preserve

Primer prioritizes explaining a computation's meaning and its transformation into executable representations. New features should preserve these properties:

- Explicit types, evaluation order, short-circuiting, and failure conditions. No value loss through implicit numeric conversions.
- Independent values after copying. Updates through `mut` must not secretly change another value.
- Immutable UTF-8 strings preserving Japanese text, NUL, CR, and LF without Unicode normalization.
- Observable correspondence between source, Primer IR, and artifacts. Observation must not expose an interface for changing running state.
- Explicit targets, artifact-consuming tools, and supported capabilities. The host OS must not silently select language behavior.

## Current language capabilities

| Area | Implemented | Limits and boundaries | Runnable examples |
| --- | --- | --- | --- |
| Signed integers | `i8`, `i16`, `i32`, `i64`, arithmetic, comparisons, bit operations | Out-of-range arithmetic stops; small types currently also occupy 64-bit storage | [sensor_calibration](../../examples/sensor_calibration.prim), [integer_limits](../../examples/integer_limits.prim) |
| Unsigned integers | `u8`, `u16`, `u32`, `u64` | `u64` covers 0–18446744073709551615; no implicit signedness changes or wrapping | [u64_values](../../examples/u64_values.prim), [packet_counter](../../examples/packet_counter.prim) |
| Floating point | `f32`, `f64` | Arithmetic rounds at the selected precision; explicit conversions involving integers must preserve the value | [floating_point](../../examples/floating_point.prim) |
| Booleans | `bool`, comparisons, `!`, short-circuiting `&&` and `||` | No implicit numeric conversion | [short_circuit](../../examples/short_circuit.prim) |
| Strings | `string`, printing, `==`, `!=`, `byte_len` | No concatenation, indexing, character count, or numeric conversion | [string_byte_length](../../examples/string_byte_length.prim), [string_lookup](../../examples/string_lookup.prim) |
| Fixed arrays | `[T; N]`, nesting, value passing, updates through `mut` | Length belongs to the type; indices are `i64`; no dynamic lengths or slices | [fixed_arrays](../../examples/fixed_arrays.prim), [heat_diffusion](../../examples/heat_diffusion.prim) |
| Named product types | Fields, defaults, nesting, value passing | No direct field assignment; construct a new value and reassign the whole binding | [product-point](../../examples/product-point.prim), [packet_counter](../../examples/packet_counter.prim) |
| Functions and control flow | Typed parameters/results, `void`, `if`/`else`, `while`/`for`, `break`/`continue`/`return` | No fixed parameter-count limit; no recursion | [function_values](../../examples/function_values.prim), [loop_control](../../examples/loop_control.prim) |
| Bindings and conversions | Immutable by default, `mut`, explicit `infer`, `T(value)` and `convert<T>(value)` | `infer` is not a runtime type; conversions do not request truncation or saturation | [integer_conversions](../../examples/integer_conversions.prim) |
| Modules | Explicit imports, namespaces, function/type visibility | Cycles/private access diagnosed; no re-exports, module variables, or package distribution | [modules](../../examples/modules/README.en.md) |

The [example type tables](../../examples/README.en.md) list ranges and applications. Whole-array and whole-product printing and equality are not implemented.

## Separate language features from output routes

These language features are supported by the VM, generated C, LLVM, QBE, WAT, Windows/Linux direct assembly, and objects from Primer's own encoder. Windows/Linux distinguish targets; assembly/objects distinguish artifacts. Use the [route and target table](targets.en.md) when counting them.

The native encoder generates x86-64 instructions and COFF/ELF objects. It shares assembly lowering and currently reads an internal assembly representation to encode it. Linking uses external tools. A typed machine-instruction IR or an internal linker would be compiler implementation work, not new language features. See the [native encoder design](native-encoder.en.md).

Language support does not imply equal observation detail. Language check failures have comparable reasons, source locations, and prior output across all routes. LLVM/assembly origin annotations still do not promise debugging information across all routes and optimization stages.

## Proposed priorities

| Priority | Candidate | Purpose and first example |
| --- | --- | --- |
| 1 (implemented) | A common contract for failure reasons, source locations, and execution outcomes | Distinguish overflow, division by zero, conversion failures, and bounds failures; locate the failing expression. Compare identical failure examples across the VM and generated routes |
| 2 (foundation implemented) | Code organization | Modules, namespaces, and visibility are implemented. Consider explicit compile-time constants when concrete examples require them |
| 2 | Remove practical function/value limitations | The four-parameter limit has been lifted across all routes. Consider constructing a product with selected fields changed. Verify mixed arguments and independent copies |
| 3 | Alternatives and recoverable failures as values | Consider enumerations/sum types, exhaustive branching, success/failure and present/absent values. Express a missing lookup without a special sentinel string |
| 3 | Reusable array and numeric operations | Consider length queries, iteration, and generics when functions need to span types or lengths. Define rounding/truncation separately from today's exact conversions |
| 4 | Dynamic data and external I/O | Define ownership, lifetimes, allocation failure, and effects before bytes, slices, dynamic arrays, concatenation, and files. Add recursion only after solving per-call storage and resource limits |
| Experiment | GPU numeric computation | Define a narrow type, memory, synchronization, and diagnostic contract; compare independent element computations with the CPU |

This is not a commitment to implement every item together. Priority 1 and foundational modules are implemented; continue evaluating additions through small designs and executable examples. A limited GPU experiment need not wait for complete modules, generics, or dynamic allocation.

The first priority-1 implementation adds [common runtime failure records](runtime-diagnostics.en.md) to the VM and Windows/Linux assembly and internal objects, including retained output before failure. C, LLVM, QBE, and WAT now implement the same contract and are compared against those reasons, locations, and prior output.

Following [file-aware locations](source-files.en.md), [modules](modules.en.md) now implement explicit imports, namespaces, and function/type visibility, with cross-route CLI comparisons of modular and single-file programs. Compile-time constants, re-exports, and package distribution remain unsupported. The four-parameter limit has also been lifted; [the mixed-argument example](../../examples/function_arguments.prim) checks evaluation order and copies. Next, consider constructing products with selected fields changed through concrete examples.

Inheritance, implicit shared mutable references, automatic GPU dispatch, general asynchronous execution, and a large package system are not early priorities because current examples have not established their need.

## GPU direction

GPU execution is worth including as a future target. Observing how sequential CPU computations become parallel element computations fits Primer's purpose. GPU generation and execution are currently unimplemented. This proposal concerns computation, not a graphics API.

### Decide types and execution contracts first

Selecting an LLVM GPU backend does not make existing programs run unchanged. NVPTX distinguishes host-launchable kernels from device functions and defines GPU address spaces. CPU output helpers also cannot simply be assumed. See the [LLVM NVPTX guide](https://llvm.org/docs/NVPTXUsage.html).

The API and hardware have not been selected:

| Candidate | Considerations |
| --- | --- |
| WebGPU / WGSL | A possible starting point for a limited 32-bit numeric experiment. WGSL runtime scalar types do not include `u64` or `f64`, so this would not support all current Primer types |
| Vulkan / SPIR-V | Query features such as `shaderInt64` and `shaderFloat64`; do not assume 64-bit support on every device |
| LLVM NVPTX / CUDA | An NVIDIA-targeted option requiring kernel calling conventions, memory rules, and host launch/result handling |

These constraints follow from [WGSL scalar types](https://www.w3.org/TR/WGSL/#scalar-types) and [Vulkan features](https://docs.vulkan.org/spec/latest/chapters/features.html). Unsupported `u64` values must never silently become `f32`. Reject unsupported capabilities explicitly, or separately verify an implementation that preserves their meaning.

At minimum, GPU support should expose:

- Input/output element types, lengths, layouts, host/device ownership boundaries, transfers, and readback.
- The relationship between logical elements and work, dispatch bounds, and synchronization. Observation APIs must not enable external mutation during execution.
- Integer range and index checks, with a contract for recovering failure reason, source location, and logical element index.
- Floating-point rounding, operation grouping, and comparison policy. Declare any tolerance in advance without silently weakening existing CPU semantics.
- Separate compilation, transfer, kernel execution, and readback timings. Do not assume small examples become faster.

Do not equate global GPU execution order with CPU `print` order. Initially prohibit output inside kernels and explicitly print results in element order on the host after readback. See the [Vulkan compute tutorial](https://docs.vulkan.org/tutorial/latest/11_Compute_Shader.html) for synchronization requirements.

### First experiment and completion criteria

1. Choose elementwise addition/transformation with read-only input arrays and disjoint output writes. Initially require a restricted form whose ranges and indices can be checked for safety.
2. Generate for explicit types, hardware, and tools, and compare against known CPU answers. Reject unsupported types or computations whose safety is not established.
3. Before supporting general computation, implement failure reporting. Define an execution-order-independent rule for multiple failures, such as selecting the smallest logical element index.
4. Separately verify element counts not divisible by the workgroup size, bounds failures, overflow, missing capabilities, and transfer failures.
5. Then expand to matrix operations and heat diffusion. [matrix_vector_product](../../examples/matrix_vector_product.prim) and [heat_diffusion](../../examples/heat_diffusion.prim) are existing CPU comparison examples, not implemented GPU examples. Replacing the latter's `f64` with `f32` for a limited route would be a separately typed experiment.

The initial experiment excludes conflicting shared writes, atomics, parallel reductions, graphics, and an internal GPU machine-code encoder.

## Completion criteria for language additions

Update Japanese design rationale, matching English documentation, a small example categorized by type, known expected results, and successful/failing execution comparisons together. Distinguish normal completion, expected diagnostics/stops, unexpected failures, and unexecuted checks. A nonzero exit alone is not evidence of success.

For common language additions, preserve semantics across existing routes and verify evaluation order, short-circuiting, independent copies, string bytes, and origins. Limited experiments such as GPU support must declare their scope; do not claim full route parity while features remain unsupported.
