# Primer design notes

Primer is a small, statically typed experimental language designed to make compiler transformations observable.

The language is intentionally small. The compiler architecture is intentionally explicit.

## Principles

Primer should preserve observability over cleverness.

In particular:

- type decisions should be visible and predictable;
- backend-independent meaning should be resolved before backend lowering;
- backend-specific decisions should happen behind an explicit lowering boundary;
- emitters should format backend IR rather than reinterpret Primer semantics;
- source-level optimization should not happen unless an explicit optimization pass is added for an experiment;
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
i64
f32
f64
```

`infer` is resolved before Primer IR is produced and therefore does not appear as a runtime or backend type.

Unsuffixed floating-point literals are also resolved before backend lowering. A backend does not need to repeat contextual type inference.

Primer IR deliberately does not attempt to be a universal machine IR or prematurely impose SSA form. It represents Primer semantics closely enough to keep the frontend/backend boundary visible.

## Backend lowering

Each backend lowers Primer IR into a backend-specific Rust representation before emission.

Current backend boundaries are:

| Backend | Internal representation | Emitted artifact |
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

### Observation 2: backend artifact

Backend emit commands expose the result after backend lowering and emission:

```text
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [-o <output.ll>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-wat <file> [-o <output.wat>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
```

These observations are intended to answer:

> How did this backend represent the resolved Primer program?

The existing `emit-*` commands are the Observation API. Primer does not currently need a second `observe` command that duplicates them.

### Internal backend IR is not an observation contract

Backend-specific Rust IR sits between Observation 1 and Observation 2, but it remains internal.

Making backend IR public would turn implementation details into compatibility requirements. That may be useful for a future experiment, but it is not part of the v0.1 observation contract.

A future explicit backend-IR observation point may be added only if there is a concrete need for it.

## v0.1 grammar

```text
program     := statement* EOF

statement   := binding
             | "print" "(" expression ")" ";"

binding     := IDENT ":" type_spec "=" expression ";"

type_spec   := "i64"
             | "f32"
             | "f64"
             | "infer"

expression  := additive

additive    := multiply (("+" | "-") multiply)*

multiply    := unary (("*" | "/") unary)*

unary       := "-" unary
             | primary

primary     := INTEGER
             | FLOAT
             | IDENT
             | "(" expression ")"
```

Bindings are immutable and may only refer to bindings declared earlier in the file.

A type specifier is always required.

```primer
count: i64 = 42;
single: f32 = 0.1 + 0.2;
double: f64 = 0.1 + 0.2;
value: infer = count * 2;
```

`infer` explicitly requests type inference. It is not itself a runtime type.

## Types

Primer v0.1 has three concrete numeric types:

```text
i64
f32
f64
```

Backends map these types to their own representations during lowering.

For example, the C backend maps them as follows:

```text
Primer    C
i64       int64_t
f32       float
f64       double
```

## Numeric literals

Integer literals have type `i64`.

```primer
x: i64 = 42;
```

Floating-point literals without a suffix are contextually typed when an explicit floating-point type is available.

```primer
a: f32 = 0.1 + 0.2;
b: f64 = 0.1 + 0.2;
```

That distinction is resolved in Primer IR before backend lowering.

For example, the C backend may emit:

```c
float primer_a = (0.1f + 0.2f);
double primer_b = (0.1 + 0.2);
```

When no expected floating-point type is available, an unsuffixed floating-point literal defaults to `f64`.

```primer
x: infer = 0.1 + 0.2;
```

Here `x` is inferred as `f64`.

A literal suffix can explicitly select its type:

```primer
a: infer = 0.1f32 + 0.2f32;
b: infer = 0.1f64 + 0.2f64;
```

Scientific notation is also accepted for floating-point literals:

```primer
x: f64 = 1.5e-3;
```

## Type checking

Arithmetic initially requires both operands to have the same type.

```text
i64 op i64 -> i64
f32 op f32 -> f32
f64 op f64 -> f64
```

Primer v0.1 performs no implicit numeric conversion.

For example:

```primer
x: infer = 1 + 0.1;
```

is a type error because the operands are `i64` and `f64`.

Explicit binding types are checked against the resolved expression type:

```primer
x: f32 = 0.1 + 0.2;
```

The `f32` binding supplies the expected type to unsuffixed floating-point literals, so the expression is evaluated as `f32`.

This decision is recorded in Primer IR and is not recomputed by individual backends.

## Output

`print(expression);` accepts all current numeric types.

Primer keeps floating-point output precise enough to expose the behavior being observed.

The current formatting policy is:

```text
i64    integer output
f32    9 significant digits
f64    17 significant digits
```

For example, an `f32` calculation such as:

```primer
x: f32 = 0.1 + 0.2;
print(x);
```

may visibly produce the floating-point approximation rather than a shortened decimal representation.

Backends are responsible for preserving the observable print behavior while implementing it according to their own target conventions.

## Code generation

Primer deliberately avoids implicit source-level optimization.

Backend lowering may perform the mechanical transformations required by its target representation, but it should not erase useful structure merely to be clever.

Examples of legitimate backend lowering include:

- selecting typed LLVM or QBE instructions;
- converting a Primer expression into WebAssembly stack instructions;
- mapping Primer values to C types and expressions;
- allocating Direct ASM stack slots and materializing constants;
- lowering Primer operations into bytecode instructions.

If optimization is introduced later, it should appear as a named pass with an explicit boundary rather than being hidden inside emission.

## CLI contract

The current CLI surface is:

```text
primer check <file>
primer emit-ir <file> [-o <output.pir>]
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [-o <output.ll>]
primer emit-wat <file> [-o <output.wat>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
primer run <file>
primer --version
```

`primer check` performs parsing and semantic/type validation.

Each `emit-*` command writes its observation to standard output by default. With `-o`, the caller chooses the output path.

`primer run` lowers through Primer bytecode and executes the resulting `BytecodeProgram` in the Primer VM. Runtime output is useful for validation and experiments, but it is distinct from the two compiler observation boundaries defined above.

Primer does not choose external experiment policy such as:

- GCC versus Clang;
- optimization levels for external compilers;
- CPU targets for external toolchains;
- benchmark settings;
- measurement policy;
- comparison policy.

Those choices belong to the caller.

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

It should consume Primer's public CLI observations rather than duplicate compiler semantics.

Its role is to make source and generated representations easy to inspect and compare interactively.

### Whitebase

Whitebase consumes emitted artifacts as experiment inputs.

Its role is to route, build, run, measure, and compare them while recording the external choices that affect the experiment.

For example:

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

For the same Primer version, source input, backend, and explicit options, Primer should produce deterministic textual observations whenever practical.

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

> Primer should make transformations easier to observe, not harder to explain.
