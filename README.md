# Primer

[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

Primer is a small experimental programming language designed to keep the path from source code to generated code observable.

It starts deliberately small. Primer itself performs as little optimization as possible so that each transformation remains visible and comparable.

The same Primer source can currently be lowered to C, LLVM IR, WebAssembly Text, or QBE IR.

## v0.1 scope

Primer currently supports:

- static typing
- `i64`, `f32`, and `f64`
- explicit type declarations
- explicit type inference with `infer`
- immutable bindings
- `+`, `-`, `*`, `/`
- unary `-`
- parentheses
- `print(expr);`
- `//` line comments
- C code generation
- LLVM IR generation
- WebAssembly Text generation
- QBE IR generation

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

The type field is always required.

`infer` explicitly requests type inference rather than omitting the type:

```primer
x: infer = 1 + 2;
```

Unsuffixed floating-point literals are contextually typed when possible:

```primer
a: f32 = 0.1 + 0.2;
b: f64 = 0.1 + 0.2;
```

These can produce different generated representations while preserving the source-level type intent.

Primer currently performs no implicit numeric conversion between `i64`, `f32`, and `f64`.

## Install

From a checkout:

```sh
cargo install --path .
```

## Use

Validate a program:

```sh
primer check examples/hello.prim
```

### Emit C

To stdout:

```sh
primer emit-c examples/hello.prim
```

To a file:

```sh
primer emit-c examples/hello.prim -o hello.c
```

Example source:

```primer
a: f32 = 0.1 + 0.2;
```

can emit straightforward C such as:

```c
float primer_a = (0.1f + 0.2f);
```

The caller can then choose the C compiler and optimization level:

```sh
cc -O0 hello.c -o hello-o0
cc -O2 hello.c -o hello-o2
cc -O3 hello.c -o hello-o3
```

### Emit LLVM IR

To stdout:

```sh
primer emit-llvm examples/hello.prim
```

To a file:

```sh
primer emit-llvm examples/hello.prim -o hello.ll
```

The same Primer expression may become LLVM IR such as:

```llvm
%tmp0 = fadd float ...
```

The generated IR can then be inspected, optimized, or compiled with external LLVM tools.

For example:

```sh
clang hello.ll -o hello
```

### Emit WebAssembly Text

To stdout:

```sh
primer emit-wat examples/hello.prim
```

To a file:

```sh
primer emit-wat examples/hello.prim -o hello.wat
```

The generated WAT can be inspected directly or passed to an external WebAssembly toolchain.

This provides another lowering path while keeping the generated representation readable.

### Emit QBE IR

To stdout:

```sh
primer emit-qbe examples/hello.prim
```

To a file:

```sh
primer emit-qbe examples/hello.prim -o hello.ssa
```

Primer stops at QBE IR.

The external QBE compiler can then lower that IR to native assembly:

```sh
qbe -o hello.s hello.ssa
```

This keeps the two stages separate and observable:

```text
Primer source
    ↓
Primer QBE backend
    ↓
QBE IR
    ↓
QBE
    ↓
Assembly
```

Primer does not choose optimization levels, CPU targets, benchmark settings, external toolchain versions, or execution policy.

Those choices belong to the caller.

## Code generation

Primer currently has four code generation backends:

```text
                         ┌─ C backend ───────→ C
                         │
                         ├─ LLVM backend ────→ LLVM IR
Primer source ───────────┤
                         ├─ WAT backend ─────→ WAT
                         │
                         └─ QBE backend ─────→ QBE IR
```

All backends share the same front end:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
    ↓
Code generation
```

This makes it possible to compare how the same Primer program is represented through different lowering paths.

The generated representation and the external toolchain are intentionally separate:

```text
                 ┌─ C ─────────→ GCC / Clang ─→ Assembly
                 │
                 ├─ LLVM IR ───→ LLVM / Clang ─→ Assembly
Primer source ───┼─ QBE IR ────→ QBE ─────────→ Assembly
                 │
                 └─ WAT ───────→ WebAssembly toolchain
```

## Design direction

Primer should preserve observability over cleverness.

The language and compiler should keep important decisions visible:

- source-level types
- inferred types
- generated C
- generated LLVM IR
- generated WAT
- generated QBE IR
- external compiler transformations
- resulting assembly or executable code

Primer intentionally avoids hiding native compiler choices inside the language toolchain.

A typical C path is:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
    ↓
C backend
    ↓
Generated C
    ↓
GCC / Clang
    ↓
Assembly / native execution
```

A direct LLVM path is:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
    ↓
LLVM backend
    ↓
LLVM IR
    ↓
LLVM / Clang
    ↓
Assembly / native execution
```

A QBE path is:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
    ↓
QBE backend
    ↓
QBE IR
    ↓
QBE
    ↓
Assembly / native execution
```

A WebAssembly path is:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
    ↓
WAT backend
    ↓
WebAssembly Text
    ↓
WebAssembly toolchain / runtime
```

## Future experiments

Primer is intentionally small enough that adding another lowering path can itself be part of the experiment.

Possible future paths include:

```text
                         ┌─ C ─────────────→ native compiler
                         ├─ LLVM IR ───────→ LLVM
Primer source ───────────┼─ QBE IR ────────→ QBE
                         ├─ WAT ───────────→ WebAssembly
                         ├─ Direct Assembly
                         └─ Primer bytecode → Primer VM
```

A direct Assembly backend would make it possible to compare assembly produced through external compiler backends with assembly generated directly by Primer.

A small Primer VM would provide a different execution path entirely.

## Tint

[Tint](https://github.com/Hokutaka/Tint) is a small visual development environment for Primer.

It provides a lightweight source editor and a way to inspect Primer's generated representations without embedding the compiler implementation into the application.

```text
Primer source
     │
     ▼
   Tint*
     │
     ├─ C
     ├─ LLVM IR
     ├─ WAT
     └─ QBE IR
```

Primer remains the language toolchain.

Tint is a window into it.

## Whitebase

[Whitebase](https://github.com/Hokutaka/Whitebase) is expected to be one consumer of Primer, not Primer's host repository or runtime.

Primer exposes observable intermediate representations; Whitebase can choose how those representations are compiled, measured, compared, and visualized.

For example, the same Primer source can be compared through paths such as:

```text
Primer → C → GCC
Primer → C → Clang
Primer → LLVM IR → LLVM
Primer → QBE IR → QBE
Primer → WAT → WebAssembly
```

with compiler versions, optimization flags, generated assembly, execution results, and measurements recorded independently.

For more detailed language semantics and grammar, see `docs/design.md`.