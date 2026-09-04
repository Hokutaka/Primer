# Primer

[日本語](README.md) | English

[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

Primer is an experimental programming language designed to make compiler transformations observable.

The compiler keeps its transformation boundaries explicit. The same resolved Primer program can be lowered through C, LLVM IR, QBE IR, WebAssembly Text, direct Windows x86-64 assembly, or Primer bytecode.

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

For the detailed architecture and invariants, see the [compiler design](docs/design/architecture.en.md).

## Observation points

Primer exposes two primary observation boundaries.

**Observation 1: resolved Primer meaning**

```sh
primer emit-ir examples/hello.prim
primer emit-ir examples/hello.prim -o hello.pir
```

Primer IR is produced after semantic and type resolution and before backend-specific lowering.

**Observation 2: output artifact**

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
- `bool`, `i64`, `f32`, and `f64`;
- named product types, default field values, and field access;
- nestable fixed arrays of fixed-size values that may also be product-type fields, indexing, element updates, value copies, and runtime bounds checks;
- typed functions accepting scalars, product types, and fixed arrays, plus `void` and explicit `return`;
- top-level executable statements or an explicit `fn main() -> void`;
- explicit type declarations;
- explicit type inference with `infer`;
- immutable bindings;
- `+`, `-`, `*`, `/`;
- unary `-`;
- `==`, `!=`, `<`, `<=`, `>`, and `>=`;
- unary `!`;
- `if` / `else` and block scope;
- `while` with `break` / `continue` targeting the innermost loop;
- `for` with explicit initialization, condition, and update;
- parentheses;
- `print(expr);`;
- `//` line comments;
- mutable bindings with `mut`;
- type-preserving reassignment and array-element updates;
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

Programs demonstrating basics, numerical computation, and algorithms supported by the current language are collected in the [examples index](examples/README.en.md).

Run every example and display its output from the repository root with:

```powershell
.\scripts\run-examples.ps1
```

Use `-Pattern "matrix*.prim"` to select matching examples. Use `cargo test --test examples` when the expected output should also be verified automatically.

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

Emit artifacts for each output route:

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

## Output routes

The backend that implements each output route follows the same architectural shape:

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

| Output route | Current target | Artifact | Typical next step |
| --- | --- | --- | --- |
| C | not selected by Primer | `.c` | GCC / Clang |
| LLVM IR | not selected by Primer | `.ll` | LLVM / Clang |
| QBE IR | not selected by Primer | `.ssa` | QBE |
| WebAssembly Text | WebAssembly | `.wat` | WebAssembly toolchain |
| Native assembly | x86-64, Windows, Windows x64 ABI | `.s` | assembler / linker |
| Primer bytecode | Primer VM | `.pbc` | Primer VM |

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

The distinction between output routes, targets, artifacts, and backends is defined in [output routes and targets](docs/design/targets.en.md).

## Design direction

Primer aims to combine sophisticated implementation with observability. As transformations become more advanced, their boundaries and results must remain observable.

In practice, that means:

- frontend type decisions are resolved before backend lowering;
- backend decisions stay behind explicit lowering boundaries;
- emitters format backend IR instead of reinterpreting Primer semantics;
- optimizations are introduced as explicit, observable passes;
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

Primer documentation is organized by purpose:

- [compiler design](docs/design/architecture.en.md);
- [observability contract](docs/design/observability.en.md);
- [output routes and targets](docs/design/targets.en.md);
- [named product type design](docs/design/product-types.en.md);
- [function design](docs/design/functions.en.md);
- [fixed array design](docs/design/fixed-arrays.en.md);
- [language reference](docs/reference/language.en.md);
- [CLI reference](docs/reference/cli.en.md).

Japanese versions and the complete documentation index are available in [docs/README.md](docs/README.md).

## License

Licensed under the [MIT License](LICENSE).
