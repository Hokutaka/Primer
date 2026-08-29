# Primer language reference

[日本語](language.ja.md)

This document defines the syntax and semantics of Primer v0.1.

## Grammar

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

Arithmetic currently requires both operands to have the same type.

```text
i64 op i64 -> i64
f32 op f32 -> f32
f64 op f64 -> f64
```

Primer v0.1 performs no implicit numeric conversion.

For example, the following expression is a type error because its operands are `i64` and `f64`:

```primer
x: infer = 1 + 0.1;
```

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
