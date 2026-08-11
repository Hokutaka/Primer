# Primer

Primer is a small experimental programming language designed to keep the path from source code to generated code observable.

It starts deliberately small. Primer itself performs as little optimization as possible so tools such as Whitebase can observe what the source expresses, what C is generated, and what the native compiler later transforms.

## v0.1 scope

Primer currently supports:

- signed 64-bit integer literals
- immutable `let` bindings
- `+`, `-`, `*`, `/`
- unary `-`
- parentheses
- `print(expr);`
- `//` line comments

Example:

```primer
let x = 1 + 2;
let y = x * 4;
print(y);
```

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

Emit C to stdout:

```sh
primer emit-c examples/hello.prim
```

Emit C to a file:

```sh
primer emit-c examples/hello.prim -o hello.c
```

Then compile it with the C compiler and optimization level you want to observe, for example:

```sh
cc -O0 hello.c -o hello-o0
cc -O2 hello.c -o hello-o2
cc -O3 hello.c -o hello-o3
```

Primer intentionally leaves those native compiler choices outside the language toolchain. The caller owns the compiler, flags, measurement environment, and assembly inspection.

## Design direction

Primer should make each layer easy to inspect:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Semantic checks
    ↓
Generated C
    ↓
External C compiler
    ↓
Assembly / native execution
```

Whitebase is expected to be one consumer of Primer, not Primer's host repository or runtime.
