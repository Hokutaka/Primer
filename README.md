# Primer

[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

Primer is a small experimental programming language designed to keep the path from source code to generated code observable.

It starts deliberately small. Primer itself performs as little optimization as possible so that each transformation remains visible and comparable.

The same Primer source can currently be lowered through several different paths, including C, LLVM IR, WebAssembly Text, QBE IR, direct x86-64 assembly, and Primer bytecode.

Primer bytecode can also be executed by Primer's own small virtual machine.

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
- direct Windows x86-64 assembly generation
- Primer bytecode generation
- Primer VM execution

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

When developing Primer itself, reinstall the CLI after changes:

```sh
cargo install --path . --force
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

### Emit direct assembly

Primer can also generate assembly directly without routing through C, LLVM, or QBE.

To stdout:

```sh
primer emit-asm examples/hello.prim
```

To a file:

```sh
primer emit-asm examples/hello.prim -o hello.s
```

The current direct assembly backend targets Windows x86-64.

This path is intentionally different from the external compiler paths:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
    ↓
Direct assembly backend
    ↓
x86-64 Assembly
```

The generated assembly can then be assembled and linked by an external toolchain.

For example:

```sh
clang hello.s -o hello
```

### Emit Primer bytecode

Primer has a small typed stack bytecode format.

To stdout:

```sh
primer emit-bytecode examples/hello.prim
```

To a file:

```sh
primer emit-bytecode examples/hello.prim -o hello.pbc
```

A program may produce bytecode such as:

```text
0000  push.f32 0.1
0001  push.f32 0.2
0002  add.f32
0003  store 0
0004  load 0
0005  print.f32
0006  halt
```

Unlike C, LLVM IR, QBE IR, or assembly, this representation is designed for Primer's own VM.

### Run with the Primer VM

Primer source can be compiled to Primer bytecode and executed directly by the Primer VM:

```sh
primer run examples/hello.prim
```

The execution path is:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
    ↓
Primer bytecode
    ↓
Primer VM
    ↓
Output
```

The VM is deliberately small and observable rather than optimized for performance.

## Code generation and execution paths

All paths share the same front end:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
```

From there, the same typed program can take several different routes:

```text
                         ┌─ C ────────────→ external C compiler
                         │
                         ├─ LLVM IR ──────→ LLVM / Clang
                         │
                         ├─ QBE IR ───────→ QBE
Primer source / AST ─────┼─ WAT ──────────→ WebAssembly toolchain
                         │
                         ├─ Direct ASM ────→ assembler / linker
                         │
                         └─ Bytecode ──────→ Primer VM
```

This makes it possible to compare how the same Primer program is represented through different lowering and execution paths.

The generated representation and the external toolchain are intentionally separate:

```text
Primer → C → GCC / Clang → Assembly
Primer → LLVM IR → LLVM / Clang → Assembly
Primer → QBE IR → QBE → Assembly
Primer → WAT → WebAssembly toolchain
Primer → Direct ASM → assembler / linker

Primer → Bytecode → Primer VM
```

Primer does not choose external compiler versions, optimization levels, benchmark settings, or execution environments for the external-toolchain paths.

Those choices belong to the caller.

## Design direction

Primer should preserve observability over cleverness.

The language and compiler should keep important decisions visible:

- source-level types
- inferred types
- generated C
- generated LLVM IR
- generated WAT
- generated QBE IR
- generated direct assembly
- generated Primer bytecode
- external compiler transformations
- Primer VM execution
- resulting assembly or executable behavior

Primer intentionally keeps source-level optimization minimal.

The goal is not to hide the path from source to execution, but to make that path easy to inspect.

The same source program can therefore be viewed through very different representations:

```text
Primer source
    │
    ├─ C
    ├─ LLVM IR
    ├─ WAT
    ├─ QBE IR
    ├─ Direct Assembly
    └─ Primer Bytecode
```

Each path exposes a different layer of the compilation process.

## Future experiments

Primer is intentionally small enough that adding another lowering or execution path can itself be part of the experiment.

Possible future work includes:

- lowering WAT to binary WebAssembly as an explicitly observable external step
- additional direct assembly targets
- further bytecode and VM experiments
- making more semantic information explicit between type checking and code generation
- improving source diagnostics and source locations

The important constraint remains the same: new machinery should make transformations easier to observe rather than hiding them.

## Tint\*

[Tint\*](https://github.com/Hokutaka/Tint-St.) is a small visual development environment for Primer.

It provides a lightweight source editor and a way to inspect Primer's generated representations and execution paths without embedding the compiler implementation into the application.

Tint\* can currently display:

```text
Primer source
     │
     ▼
   Tint*
     │
     ├─ C
     ├─ C ASM
     ├─ LLVM IR
     ├─ LLVM ASM
     ├─ WAT
     ├─ QBE IR
     ├─ QBE ASM
     ├─ Direct ASM
     ├─ Bytecode
     └─ VM Output
```

Some views are produced directly by Primer, while others intentionally show an external transformation.

For example:

```text
Primer → C → Clang → C ASM
Primer → LLVM IR → Clang → LLVM ASM
Primer → QBE IR → QBE → QBE ASM
Primer → Direct ASM
Primer → Bytecode → Primer VM → VM Output
```

Primer remains the language toolchain.

Tint* is a window into it.

## Whitebase

[Whitebase](https://github.com/Hokutaka/Whitebase) is expected to be one consumer of Primer, not Primer's host repository or runtime.

Primer exposes observable intermediate representations and execution paths; Whitebase can choose how those representations are compiled, measured, compared, and visualized.

For example, the same Primer source can be compared through paths such as:

```text
Primer → C → GCC
Primer → C → Clang
Primer → LLVM IR → LLVM
Primer → QBE IR → QBE
Primer → WAT → WebAssembly
Primer → Direct ASM
Primer → Bytecode → Primer VM
```

with compiler versions, optimization flags, generated assembly, execution results, and measurements recorded independently.

For more detailed language semantics and grammar, see `docs/design.md`.