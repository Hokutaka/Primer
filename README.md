# Primer

[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

Primer is a small experimental programming language designed to keep the path from source code to generated code observable.

It starts deliberately small. Primer itself performs as little optimization as possible so that each transformation remains visible and comparable.

Primer can currently emit either straightforward C or LLVM IR from the same source program.

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

The generated IR can then be inspected, assembled, optimized, or compiled with external LLVM tools.

For example:

```sh
llvm-as hello.ll -o hello.bc
clang hello.ll -o hello
```

Primer does not choose optimization levels, CPU targets, benchmark settings, or execution policy.

Those choices belong to the caller.

## Code generation

Primer currently has two code generation backends:

```text
                     ┌─ C backend ─────→ C
                     │
Primer source ───────┤
                     │
                     └─ LLVM backend ──→ LLVM IR
```

Both backends share the same front end:

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

## Design direction

Primer should preserve observability over cleverness.

The language and compiler should keep important decisions visible:

- source-level types
- inferred types
- generated C
- generated LLVM IR
- external compiler optimization
- resulting machine code

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

## Whitebase

[Whitebase](https://github.com/Hokutaka/Whitebase) is expected to be one consumer of Primer, not Primer's host repository or runtime.

Primer exposes observable intermediate representations; Whitebase can choose how those representations are compiled, measured, compared, and visualized.

For example, the same Primer source can eventually be compared through paths such as:

```text
Primer → C → GCC
Primer → C → Clang
Primer → LLVM IR → LLVM
```

with compiler versions, optimization flags, generated assembly, execution results, and measurements recorded independently.

For more detailed language semantics and grammar, see `docs/design.md`.