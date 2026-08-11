# Primer design notes

Primer is intentionally a small, statically typed source-to-C language.

## Principle

Primer should preserve observability over cleverness.

The compiler should avoid source-level optimization unless a future experiment explicitly adds an optimization pass. This keeps the transformation boundary visible:

1. Primer source expresses the program.
2. Primer lexes and parses it into an AST.
3. Primer resolves and validates types.
4. Primer emits straightforward C.
5. An external C compiler owns optimization and machine-code generation.
6. Consumers such as Whitebase can compare each stage independently.

Primer should make type decisions visible and predictable. It should not silently insert numeric conversions or hide transformations that are useful to observe.

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

They map directly to C types:

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

The generated C preserves that distinction:

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

## Output

`print(expression);` accepts all current numeric types.

Generated C uses enough significant digits to make floating-point behavior observable:

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

## C generation

Primer emits straightforward C and deliberately avoids source-level optimization.

For example:

```primer
a: f32 = 0.1 + 0.2;
b: f64 = 0.1 + 0.2;
```

may emit:

```c
float primer_a = (0.1f + 0.2f);
double primer_b = (0.1 + 0.2);
```

Primer preserves expression structure where practical so that later transformations performed by the native compiler remain observable.

## CLI contract

```text
primer check <file>
primer emit-c <file> [-o <output.c>]
primer --version
```

`primer check` runs the source through parsing and semantic/type validation.

`primer emit-c` writes generated C to standard output by default. With `-o`, the caller chooses the output path.

`emit-c` is the important integration boundary.

Primer does not select:

- GCC or Clang;
- optimization levels;
- CPU targets;
- output directories;
- benchmark settings;
- execution policy.

Those choices belong to the caller.

## Whitebase integration

Whitebase should treat Primer as an external installed tool, not as a source dependency.

A future adapter can:

1. detect `primer` on `PATH`;
2. record `primer --version`;
3. invoke `primer emit-c` for a benchmark source;
4. store the generated C in the experiment workspace;
5. compile it with explicitly recorded compiler flags;
6. measure the resulting executable;
7. inspect and compare the generated machine code.

This keeps the boundary explicit:

```text
Primer source
    ↓
Lexer
    ↓
Parser / AST
    ↓
Type checking
    ↓
Generated C
    ↓
External C compiler
    ↓
Machine code
    ↓
Whitebase observation
```