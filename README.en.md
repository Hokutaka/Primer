[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

# Primer

[日本語](README.md) | English

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
  - contextual numeric literal resolution
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

- **Types:** `bool`; `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`; `f32`, `f64`. Static typing and explicit type inference with `infer`.
- **Variables:** immutable bindings, mutable bindings with `mut`, and type-preserving reassignment.
- **Arithmetic:** `+`, `-`, `*`, `/`, unary `-`, and integer `%`. Integer overflow and invalid division stop execution.
- **Comparison and logic:** `==`, `!=`, `<`, `<=`, `>`, `>=`, `!`, and short-circuiting `&&` and `||`.
- **Bit operations:** integer `&`, `|`, `^`, `~`, `<<`, and `>>`, with checked shift counts and left-shift overflow detection.
- **Explicit conversion:** equivalent spellings such as `f64(value)` and `convert<f64>(value)`. Conversion between implemented numeric types succeeds only when it preserves the value.
- **Data structures:** named structs (product types), default field values and field access, and nestable fixed arrays. Value copies, array-element updates, and runtime bounds checks.
- **Functions and control flow:** typed functions passing numbers, booleans, structs, and fixed arrays, with `void` and explicit `return`. `if` / `else`, `while`, `for`, and `break` / `continue`. Top-level executable statements or an explicit `fn main() -> void`.
- **Output and execution:** `print(expr);`, Primer IR and backend artifact emission, and Primer VM execution. Line comments use `//`.

`u64`, strings, dynamically sized arrays, recursion, failure recovery, and explicit rounding/truncation operations are not implemented. Current generated targets store even small integer types in 64-bit storage and check their value ranges.

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

Programs demonstrating basics, data structures, numerical computation, and algorithms are collected in the [examples index](examples/README.en.md). The [sample mean and variance](examples/measurement_statistics.prim) and [count-to-probability](examples/normalized_histogram.prim) examples use explicit integer/floating-point conversion.

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

Primer performs no implicit numeric conversion. Changing a type requires an explicit conversion:

```primer
count: u32 = 3;
ratio: f64 = f64(count) / convert<f64>(2);
print(ratio); // 1.5
// print(i32(ratio)); // fails because the fractional part would be lost
```

Conversion stops execution if the destination cannot preserve the value; it does not silently round it. Ordinary floating-point arithmetic still rounds. See the [language reference](docs/reference/language.en.md) for detailed rules, including infinity, NaN, and negative zero.

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
