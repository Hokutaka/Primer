# Primer compiler design

[日本語](architecture.ja.md)

Primer is a statically typed experimental language designed to make compiler transformations observable. Its compiler architecture and transformation boundaries are explicit.

The boundaries that Primer preserves for observability are defined in the [observability contract](observability.en.md). Terminology and conditions for generated output are defined in [output routes and targets](targets.en.md).

## Principles

Primer aims to combine sophisticated implementation with observability. As transformations become more advanced, their boundaries and results must remain observable.

In particular:

- type decisions should be visible and predictable;
- backend-independent meaning should be resolved before backend lowering;
- backend-specific decisions should happen behind an explicit lowering boundary;
- emitters should format backend IR rather than reinterpret Primer semantics;
- optimizations should be introduced as explicit, observable passes that can be examined as part of an experiment;
- generated observations should avoid incidental nondeterminism such as timestamps or random identifiers.

Primer v0.1 does not silently insert numeric conversions or hide transformations that are useful to observe.

## Compiler architecture

The compiler pipeline is:

```text
Primer Source
      ↓
Lexer / Parser
      ↓
AST
      ↓
Primer IR Builder
  - semantic validation
  - type resolution
  - contextual float resolution
      ↓
Primer IR
  - typed
  - backend independent
      │
      ├── Observation 1: emit-ir / .pir
      │
      ↓
Backend Lowering
      ↓
Backend-specific Rust IR
      ↓
Emitter
      ↓
Backend Artifact
      │
      └── Observation 2
```

The key architectural boundary is Primer IR.

The frontend decides what the Primer program means. Backends decide how that already-resolved meaning is represented for a target.

### Architectural invariants

The following rules are part of the compiler design:

1. Backend compilation starts from Primer IR, not directly from the AST.
2. Semantic validation and type resolution happen before backend lowering.
3. A backend lowerer may know both Primer IR and its own backend IR.
4. A backend emitter must not depend on Primer IR, the AST, or semantic-analysis state.
5. Backend-specific Rust IR is an internal implementation boundary.
6. Public observations are Primer IR text and emitted backend artifacts.
7. Optimization is not implicit. A future optimization stage must be an explicit, observable pass.
8. Primer IR gives each binding a deterministic compilation-local ID so references remain explicit across shadowing.
9. Structured `if`, `while`, `for`, `break`, and `continue` statements remain in Primer IR. A `for` keeps its initializer, condition, body, and update distinct; branches, merge points, update paths, back edges, and loop exits are introduced during lowering into Bytecode and each backend IR.
10. Every Primer IR statement and expression has a deterministic `NodeId` that is unique within one compilation. A `NodeId` identifies an element, while a `Span` locates source text; neither substitutes for the other.

Conceptually, every backend follows the same structure:

```text
Primer IR
    ↓
backend::lower()
    ↓
Backend-specific Rust IR
    ↓
backend::emit()
    ↓
Artifact
```

The physical Rust module layout may differ between backends, but the architectural boundary is the same.

## Primer IR

Primer IR is the typed, backend-independent representation produced after parsing, semantic validation, and type resolution.

Every Primer IR expression has a resolved concrete type:

```text
bool
i64
f32
f64
named product types
fixed arrays
```

`infer` is resolved before Primer IR is produced and therefore does not appear as a runtime or backend type.

Unsuffixed floating-point literals are also resolved before backend lowering. A backend does not need to repeat contextual type inference.

Integer literals retain their decimal digits in lexer tokens and the AST. Semantic analysis checks the range against the expected integer type, and construction of Primer IR converts the literal into a resolved value. This prevents the lexer's `i64` range from constraining future integer types.

Primer IR deliberately does not attempt to be a universal machine IR or prematurely impose SSA form. It represents Primer semantics closely enough to keep the frontend/backend boundary visible.

Primer IR statements and expressions share one sequence of `NodeId` values. `emit-ir` renders them as `#0`, `#1`, and so on. IDs are allocated deterministically, with a parent before its children and in textual IR order, so the same Primer version and input produce the same IDs.

A `NodeId` refers to an element within one compilation result. It is not stable across source edits or Primer versions. Multiple IR elements with the same `Span` can still have different `NodeId` values. This distinction provides a foundation for recording how one expression is later split into multiple backend instructions without relying on source locations as identity.

## Backend lowering

Each backend lowers Primer IR into a backend-specific Rust representation before emission.

An expression represented as one integer operation in Primer IR may become the operation plus an overflow check during backend lowering. The check is not left to accidental behavior in an external tool. Backend IR retains it as a checked integer operation or an explicit trap condition. The generated artifact exposes the target-appropriate result, such as a helper call, an overflow-flag branch, or `unreachable`.

The current output routes and implementation boundaries are:

| Output route | Backend-internal representation | Emitted artifact |
| --- | --- | --- |
| C | C IR | `.c` |
| LLVM | LLVM IR representation | `.ll` |
| QBE | QBE IR representation | `.ssa` |
| WebAssembly | WAT-oriented instruction IR | `.wat` |
| Direct x86-64 Windows assembly | assembly IR | `.s` |
| Primer bytecode | `BytecodeProgram` | `.pbc` |

Backend IR is allowed to encode decisions that do not belong in Primer IR.

Examples include:

- LLVM/QBE temporary values and backend instructions;
- WAT stack-machine instructions and locals;
- C-specific types and print formats;
- x86-64 stack slots, frame size, constants, registers, and ABI operations;
- bytecode slots and VM instructions.

These representations are not currently public serialization formats and are not part of the stable CLI contract.

### Source locations and bytecode instruction provenance

Primer IR statements and expressions retain their corresponding UTF-8 byte ranges in the source. A range includes its start and excludes its end. Line and column numbers are derived from this range when displayed.

Each bytecode instruction stores one of the following origins separately from the instruction itself:

- `Source { node_id, span }`: the instruction was lowered from a Primer IR statement or expression;
- `Synthetic`: the compiler generated the instruction without a directly corresponding source range.

`Synthetic` does not mean that provenance was lost. It explicitly identifies compiler-generated instructions.

The `node_id` identifies the Primer IR element that produced an instruction. When one IR element lowers into several instructions, those instructions may share the same `node_id`. The `span` is the focused source range used for diagnostics and does not have to cover the whole IR element. For example, bounds checks produced by a nested array-element assignment share the assignment statement's `node_id` while retaining a different index `span` for each check.

The VM reports an execution error using its bytecode instruction index. `run_vm` resolves that instruction's origin and associates the Primer IR `NodeId` and, when available, a source location with the execution error. This provenance is currently an internal representation and is not included in the `emit-bytecode` text format.

## Observation boundaries

Primer exposes two primary observation boundaries.

### Observation 1: resolved Primer meaning

```text
primer emit-ir <file> [-o <output.pir>]
```

The `.pir` observation is produced after frontend semantic and type resolution but before backend lowering.

It is intended to answer:

> What does Primer consider this source program to mean?

An `emit-ir` result guarantees that:

- parsing succeeded;
- semantic validation succeeded;
- expression types are resolved;
- contextual floating-point types are resolved;
- the result is backend independent.

Backend allocation, ABI, stack-machine, or target-instruction decisions do not belong in this observation.

### Observation 2: output artifact

Emit commands for each output route expose the result after backend lowering and emission:

```text
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [-o <output.ll>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-wat <file> [-o <output.wat>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
```

These observations are intended to answer:

> How did the selected output route and target represent the resolved Primer program?

The existing `emit-*` commands are the observation API. Primer does not currently need a second `observe` command that duplicates them.

### Internal backend IR is not an observation contract

Backend-specific Rust IR sits between Observation 1 and Observation 2, but it remains internal.

Making backend IR public would turn implementation details into compatibility requirements. That may be useful for a future experiment, but it is not part of the v0.1 observation contract.

A future explicit backend-IR observation point may be added only if there is a concrete need for it.

## Code generation

Primer avoids unobservable, implicit source-level optimization rather than optimization itself.

Backend lowering may perform advanced transformations as well as the mechanical transformations required by its target representation. If a transformation removes structure that is useful to observe, it must be treated as an explicit, observable pass.

Examples of legitimate backend lowering include:

- selecting typed LLVM or QBE instructions;
- converting a Primer expression into WebAssembly stack instructions;
- mapping Primer values to C types and expressions;
- allocating Direct ASM stack slots and materializing constants;
- lowering Primer operations into bytecode instructions.

If optimization is introduced later, it should appear as a named pass with an explicit boundary rather than being hidden inside emission.

## Tool responsibilities

Primer, Tint*, and Whitebase have different responsibilities.

### Primer

Primer owns compiler transformation and emission:

```text
Parse
  ↓
Resolve
  ↓
Primer IR
  ↓
Lower
  ↓
Backend IR
  ↓
Emit
```

Primer defines and produces the observable compiler artifacts.

### Tint*

Tint* is a visual development and inspection environment for Primer.

It should consume Primer's public CLI observations rather than duplicate compiler semantics. Its role is to make source and generated representations easy to inspect and compare interactively.

### Whitebase

Whitebase consumes emitted artifacts as experiment inputs.

Its role is to route, build, run, measure, and compare them while recording the external choices that affect the experiment.

```text
Primer source
      ↓
Primer
      ↓
Observation artifact
      ↓
Whitebase
  - select build route
  - invoke external tools
  - run
  - measure
  - compare
```

Whitebase should treat Primer as an external tool boundary rather than depending on Primer's internal Rust IR.

## Reproducibility

Observation artifacts are most useful when they can be compared directly.

For the same Primer version, source input, output route, target, target features, and explicit options, Primer should produce deterministic textual observations whenever practical.

Primer-generated observations should therefore avoid incidental values such as timestamps, random identifiers, or environment-dependent metadata unless such data is itself the subject of an experiment.

External toolchain output is outside this guarantee and should be recorded by the consumer, such as Whitebase.

## Non-goals and future work

The current design intentionally does not require:

- a public serialization format for backend-specific Rust IR;
- a generic `observe` command duplicating the existing `emit-*` commands;
- a universal SSA representation shared by all backends;
- implicit optimization;
- build orchestration inside Primer;
- benchmarking or performance measurement inside Primer.

Possible future work includes:

- an explicit optimization pipeline with additional observation boundaries;
- optional backend-IR inspection when a concrete use case requires it;
- an Observation Bundle that collects source, Primer IR, emitted artifacts, and metadata together;
- consuming serialized Primer IR as an explicit compiler input if experiments require replaying the backend half of the pipeline.

Those features should be added only when they preserve the central rule:

> Primer should make transformations easier to observe and easier to explain.
