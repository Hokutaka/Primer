# Primer language reference

[日本語](language.ja.md)

This document defines the syntax and semantics of Primer v0.1.

## Grammar

```text
program     := statement* EOF

statement   := binding
             | assignment
             | "print" "(" expression ")" ";"
             | if_statement
             | while_statement

if_statement := "if" expression block ("else" block)?

while_statement := "while" expression block

block       := "{" statement* "}"

binding     := "mut"? IDENT ":" type_spec "=" expression ";"

assignment  := IDENT "=" expression ";"

type_spec   := "i64"
             | "f32"
             | "f64"
             | "bool"
             | "infer"

expression  := equality

equality    := comparison (("==" | "!=") comparison)*

comparison  := additive (("<" | "<=" | ">" | ">=") additive)*

additive    := multiply (("+" | "-") multiply)*

multiply    := unary (("*" | "/") unary)*

unary       := ("-" | "!") unary
             | primary

primary     := "true"
             | "false"
             | INTEGER
             | FLOAT
             | IDENT
             | "(" expression ")"
```

Bindings are immutable by default. Only a binding declared with `mut` can be reassigned. Expressions may only refer to bindings declared earlier in the file.

A type specifier is always required.

```primer
count: i64 = 42;
single: f32 = 0.1 + 0.2;
double: f64 = 0.1 + 0.2;
value: infer = count * 2;
```

`infer` explicitly requests type inference. It is not itself a runtime type.

## Mutable bindings and reassignment

A binding that needs to change is declared with `mut` before its name:

```primer
mut count: i64 = 40;
count = count + 2;
print(count);
```

`mut` is not a type. It specifies that the name `count` may be reassigned. Reassignment does not contain `: type_spec`; this distinction separates a new declaration from assignment to an existing binding.

Reassigning a binding without `mut` is a type-checking error:

```primer
count: i64 = 40;
count = 42; // error
```

The assigned value must have the type resolved when the binding was declared. With `infer`, inference happens only at the declaration:

```primer
mut value: infer = 1; // resolved as i64
value = 2;            // OK
value = 0.5;          // error
```

Primer IR preserves initialization and reassignment as different statements. Bytecode likewise distinguishes initialization `store` from reassignment `assign`.

## Conditionals, loops, and block scope

`if` executes statements according to a `bool` condition. The `else` block is optional.

```primer
if value < 10 {
    print(value);
} else {
    print(10);
}
```

A condition that is not `bool` is a type-checking error. An `if` is currently a statement and does not produce a value.

`while` repeats its body while its condition is `true`. The condition is evaluated before the body on every iteration, so a condition that starts as `false` executes the body zero times.

```primer
mut count: i64 = 0;

while count < 3 {
    print(count);
    count = count + 1;
}
```

The condition of a `while` must also be `bool`. A `while` is a statement and does not produce a value.

Each braced block creates a new scope. Bindings declared inside a block are not visible outside it. An inner block can read an outer binding and can reassign it when it is `mut`.

An inner block may declare a distinct binding with the same name as an outer binding.

```primer
mut value: i64 = 1;

if true {
    value = 2;          // updates the outer value
    value: bool = true; // a distinct value local to this block
    print(value);       // the bool value
}

print(value);           // the i64 value
```

Primer IR assigns deterministic IDs to bindings so references remain unambiguous when names are reused. Structured `if` and `while` statements remain visible in Primer IR. During lowering into Bytecode and backend IRs, an `if` becomes branches and a merge point, while a `while` becomes a conditional branch and a path from its body back to its condition.

## Types

Primer v0.1 has one boolean type and three numeric types:

```text
bool
i64
f32
f64
```

Backends map these types to their own representations during lowering.

For example, the C backend maps them as follows:

```text
Primer    C
bool      bool
i64       int64_t
f32       float
f64       double
```

## Booleans and comparisons

`bool` has two values, `true` and `false`. The `!` operator negates a boolean value.

```primer
enabled: bool = true;
disabled: bool = !enabled;
```

`==` and `!=` compare values of the same type. Numeric types additionally support `<`, `<=`, `>`, and `>=`. A comparison always produces `bool`.

```primer
same: bool = enabled == true;
small: bool = 1 + 2 < 4;
different: bool = 0.1f32 != 0.2f32;
```

Boolean ordering and arithmetic are not supported. Comparisons do not perform implicit numeric conversion.

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

Comparison operands must also have the same type. Primer IR exposes the operand type separately from the resulting `bool` type.

## Output

`print(expression);` accepts every current concrete type.

Primer keeps floating-point output precise enough to expose the behavior being observed.

The current formatting policy is:

```text
bool   `true` or `false`
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
