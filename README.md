# Primer

[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

Primer is a small experimental programming language designed to make compiler transformations observable.

The language is deliberately small, and the compiler keeps its transformation boundaries explicit. The same resolved Primer program can be lowered through C, LLVM IR, QBE IR, WebAssembly Text, direct Windows x86-64 assembly, or Primer bytecode.

Primer bytecode can also be executed by Primer's own small virtual machine.

## Compiler architecture

All backends share the same frontend and the same typed, backend-independent Primer IR:

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
      │
      ├── emit-ir / .pir
      │
      ↓
Backend Lowering
      ↓
Backend-specific Rust IR
      ↓
Emitter
      ↓
Backend Artifact
```

The important boundary is Primer IR.

The frontend decides what the source program means. Each backend then lowers that resolved meaning into its own internal Rust representation before emitting text or bytecode.

Primer's backend emitters do not reinterpret the AST or repeat semantic/type resolution.

For the detailed architecture and invariants, see [docs/design.md](docs/design.md).

## Observation points

Primer exposes two primary observation boundaries.

**Observation 1: resolved Primer meaning**

```sh
primer emit-ir examples/hello.prim
primer emit-ir examples/hello.prim -o hello.pir
```

Primer IR is produced after semantic and type resolution and before backend-specific lowering.

**Observation 2: backend artifact**

```text
emit-c         → .c
emit-llvm      → .ll
emit-qbe       → .ssa
emit-wat       → .wat
emit-asm       → .s
emit-bytecode  → .pbc
```

The existing `emit-*` commands are the observation surface. Backend-specific Rust IR remains an internal lowering boundary.

## v0.1 scope

Primer currently supports:

- static typing;
- `i64`, `f32`, and `f64`;
- explicit type declarations;
- explicit type inference with `infer`;
- immutable bindings;
- `+`, `-`, `*`, `/`;
- unary `-`;
- parentheses;
- `print(expr);`;
- `//` line comments;
- Primer IR emission;
- C code generation;
- LLVM IR generation;
- QBE IR generation;
- WebAssembly Text generation;
- direct Windows x86-64 assembly generation;
- Primer bytecode generation;
- Primer VM execution.

Example:

```primer
integer: i64 = 1 + 2;

single: f32 = 0.1 + 0.2;
double: f64 = 0.1 + 0.2;

inferred: infer = single + single;

print(integer);
print(single);
print(double);
print(inferred);
```

The type field is always required. `infer` explicitly requests inference rather than omitting the type:

```primer
x: infer = 1 + 2;
```

Unsuffixed floating-point literals are contextually typed when possible:

```primer
a: f32 = 0.1 + 0.2;
b: f64 = 0.1 + 0.2;
```

That decision is resolved in Primer IR before backend lowering.

Primer currently performs no implicit numeric conversion between `i64`, `f32`, and `f64`.

## Install

From a checkout:

```sh
cargo install --path .
```

When developing Primer itself, reinstall the CLI after changes:

```sh
cargo install --path . --force
```

## CLI

Validate a source file:

```sh
primer check examples/hello.prim
```

Emit resolved Primer IR:

```sh
primer emit-ir examples/hello.prim
primer emit-ir examples/hello.prim -o hello.pir
```

Emit backend artifacts:

```sh
primer emit-c examples/hello.prim -o hello.c
primer emit-llvm examples/hello.prim -o hello.ll
primer emit-qbe examples/hello.prim -o hello.ssa
primer emit-wat examples/hello.prim -o hello.wat
primer emit-asm examples/hello.prim -o hello.s
primer emit-bytecode examples/hello.prim -o hello.pbc
```

Run through Primer bytecode and the Primer VM:

```sh
primer run examples/hello.prim
```

Without `-o`, emit commands write to standard output.

## Backend paths

Each backend follows the same architectural shape:

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

The current routes are:

| Backend | Artifact | Typical next step |
| --- | --- | --- |
| C | `.c` | GCC / Clang |
| LLVM | `.ll` | LLVM / Clang |
| QBE | `.ssa` | QBE |
| WebAssembly | `.wat` | WebAssembly toolchain |
| Direct x86-64 Windows assembly | `.s` | assembler / linker |
| Primer bytecode | `.pbc` | Primer VM |

For example:

```text
Primer Source
      ↓
Primer IR
      ├──→ C IR        → .c   → GCC / Clang
      ├──→ LLVM IR     → .ll  → LLVM / Clang
      ├──→ QBE IR      → .ssa → QBE
      ├──→ WAT IR      → .wat → WebAssembly toolchain
      ├──→ ASM IR      → .s   → assembler / linker
      └──→ Bytecode IR → .pbc → Primer VM
```

Primer owns the transformation up to the emitted artifact. External compiler versions, optimization levels, CPU targets, benchmark settings, and measurement policy belong to the caller.

## Design direction

Primer should preserve observability over cleverness.

In practice, that means:

- frontend type decisions are resolved before backend lowering;
- backend decisions stay behind explicit lowering boundaries;
- emitters format backend IR instead of reinterpreting Primer semantics;
- source-level optimization stays minimal unless an explicit optimization pass is introduced;
- generated observations should avoid incidental nondeterminism where practical.

The goal is not merely to generate code. It is to keep the route from source meaning to target representation inspectable.

## Tint\*

[Tint\*](https://github.com/Hokutaka/Tint-St.) is a visual development and inspection environment for Primer.

Tint* consumes Primer's public CLI output and presents source and generated representations side by side. It may also invoke external tools to show downstream views such as C-generated assembly, LLVM-generated assembly, or QBE-generated assembly.

Primer remains the language toolchain. Tint* is a window into it.

## Whitebase

[Whitebase](https://github.com/Hokutaka/Whitebase) is an external consumer of Primer artifacts.

Primer is responsible for:

```text
Transform → Lower → Emit → Observe
```

Whitebase can take those artifacts and:

```text
Route → Build → Run → Measure → Compare
```

This keeps compiler semantics inside Primer while leaving toolchain selection, benchmarking, and comparison policy outside it.

## Documentation

For detailed language semantics, compiler architecture, observation boundaries, and future design constraints, see [docs/design.md](docs/design.md).
