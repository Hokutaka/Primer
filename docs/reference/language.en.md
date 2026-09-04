# Primer language reference

[日本語](language.ja.md)

This document defines the syntax and semantics of Primer v0.1.

## Grammar

```text
program     := item* EOF

item        := type_definition
             | function_definition
             | statement

type_definition :=
    "type" IDENT "{" field_definition ("," field_definition)* ","? "}"

field_definition := IDENT ":" type_ref ("=" expression)?

function_definition :=
    "fn" IDENT "(" parameters? ")" "->" return_type block

parameters  := parameter ("," parameter)*

parameter   := IDENT ":" type_ref

return_type := type_ref | "void"

statement   := binding
             | assignment
             | "print" "(" expression ")" ";"
             | IDENT "(" arguments? ")" ";"
             | "return" expression? ";"
             | if_statement
             | while_statement
             | for_statement
             | "break" ";"
             | "continue" ";"

if_statement := "if" expression block ("else" block)?

while_statement := "while" expression block

for_statement :=
    "for" "(" (binding_clause | assignment_clause) ";"
    expression ";" assignment_clause ")" block

block       := "{" statement* "}"

binding     := "mut"? IDENT ":" type_spec "=" expression ";"

binding_clause := "mut"? IDENT ":" type_spec "=" expression

assignment  := assignment_clause ";"

assignment_clause := IDENT "=" expression

type_spec   := "i64"
             | "f32"
             | "f64"
             | "bool"
             | fixed_array_type
             | IDENT
             | "infer"

type_ref    := "i64" | "f32" | "f64" | "bool" | fixed_array_type | IDENT

fixed_array_type := "[" type_ref ";" INTEGER "]"

expression  := equality

equality    := comparison (("==" | "!=") comparison)*

comparison  := additive (("<" | "<=" | ">" | ">=") additive)*

additive    := multiply (("+" | "-") multiply)*

multiply    := unary (("*" | "/") unary)*

unary       := ("-" | "!") unary
             | postfix

postfix     := primary (("." IDENT) | ("[" expression "]"))*

primary     := "true"
             | "false"
             | INTEGER
             | FLOAT
             | "[" expression ("," expression)* ","? "]"
             | IDENT
             | IDENT "(" arguments? ")"
             | IDENT "{" field_value ("," field_value)* ","? "}"
             | "(" expression ")"

arguments   := expression ("," expression)*

field_value := IDENT ":" expression
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

## Named product types

`type` defines a named product type that groups several fields into one value. Types have nominal identity, so two types with the same fields are still distinct.

```primer
type Point {
    x: f64 = 0.0,
    y: f64,
}

point: Point = Point {
    y: 2.0,
};

print(point.x);
```

Every field type is explicit and cannot use `infer`. A field without a default is required when constructing a value. Fields are named, so construction order may differ from definition order. Trailing commas are accepted.

Explicit field expressions are evaluated in source order. Defaults for omitted fields are then evaluated in definition order. Primer IR exposes this order and whether each value was explicit or came from a default.

`.` accesses a field and may be chained as in `segment.start.x`. Fields cannot be assigned directly. To make a change, construct a new value and reassign the whole `mut` binding.

```primer
mut point: Point = Point { x: 1.0, y: 2.0, };
point = Point { x: 3.0, y: point.y, };
```

Reassigning the original binding after copying a product value into another binding does not change the earlier value. This is the language-level value rule. Each backend's physical placement and copying remain observable in emitted artifacts.

The `{` immediately after an `if` or `while` condition starts its block. Parenthesize a construction expression when accessing one of its fields in a condition.

```primer
type Flags { enabled: bool, }

if (Flags { enabled: true, }).enabled {
    print(true);
}
```

Empty product types, empty construction expressions, infinitely sized recursion by value, product comparisons, and printing a whole product value are not currently supported.

See [Named product type design](../design/product-types.en.md) for the detailed design and backend representations.

## Fixed arrays

A fixed array is a value containing a known number of boxes of the same type. `[i64; 4]` means an array with four `i64` boxes.

```primer
values: [i64; 4] = [2, 4, 6, 8];
print(values[2]);
```

The length is part of the type, so `[i64; 3]` and `[i64; 4]` are different types. Primer reports an error when a literal has the wrong number of elements or its element types differ. Empty array literals are not currently available because they provide no element type to infer.

An index has type `i64`, and the first index is `0`. In the example above, `values[2]` reads the third value, `6`. A negative index or an index greater than or equal to the length stops execution in both the Primer VM and generated programs. Every backend leaves this bounds check visible in its artifact.

An array is copied as one value. Reassigning the original `mut` binding after a copy does not change the earlier copy.

```primer
mut first: [i64; 2] = [10, 20];
second: [i64; 2] = first;
first = [30, 40];
print(second[0]); // 10
```

An element type may be `bool`, `i64`, `f32`, `f64`, or a named product type. A fixed array may also be used as a field of a product type.

```primer
type Point {
    x: i64,
    y: i64,
}

type Path {
    points: [Point; 4],
}

path: Path = Path {
    points: [
        Point { x: 0, y: 0, },
        Point { x: 1, y: 1, },
        Point { x: 2, y: 4, },
        Point { x: 3, y: 9, },
    ],
};

print(path.points[2].y);
```

Directly nested fixed arrays such as `[[i64; 2]; 2]`, array parameters and results, and direct element assignment such as `values[0] = 1;` are not yet supported. Primer diagnoses these forms instead of silently assigning them another meaning.

See [Fixed array design](../design/fixed-arrays.en.md) for the detailed design and bounds-check representation in each backend.

## Functions and entrypoint

`fn` defines a named computation. Every parameter and the return type are explicit.

```primer
fn add(left: i64, right: i64) -> i64 {
    return left + right;
}

answer: i64 = add(20, 22);
```

A value-returning function uses an explicit `return expression;`. A trailing expression is not an implicit result. Primer reports an error when it cannot prove that every path returns a value.

A function without a value uses `-> void`. It may reach the end of its block or exit early with `return;`. A value-returning call is used as an expression, while a `void` call is a statement.

```primer
fn show(value: i64) -> void {
    print(value);
}

show(answer);
```

Function names are resolved across the whole file, so a call may precede its definition. Parameters and local bindings are not visible outside their function. A function also cannot read a top-level runtime binding.

Top-level executable statements receive a compiler-generated entrypoint. A program may instead define `fn main() -> void`, but an explicit `main` cannot be combined with top-level executable statements. `main` takes no parameters.

Current function signatures are limited to `bool`, `i64`, `f32`, and `f64`, with at most four parameters. Named product or fixed-array arguments and results, recursion, and command-line arguments are not yet supported. Unsupported forms produce diagnostics instead of silently changing meaning.

Primer IR and bytecode expose function IDs, parameter binding IDs, calls, and returns. Backend artifacts expose how those entities become function symbols, arguments, local storage, and ABI registers or memory. See [Function design](../design/functions.en.md) for details.

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

`for` groups a start statement, a `bool` continuation condition, an update statement, and a body. The start statement may declare a new binding or assign to an existing `mut` binding:

```primer
mut sum: i64 = 0;

for (mut i: i64 = 0; i < 6; i = i + 1) {
    sum = sum + i;
}
```

The start statement runs once. Before every iteration, the continuation condition is evaluated. After every completed iteration, the update statement runs and control returns to the condition. All three header parts are required in the current syntax.

A binding declared by the start statement is visible in the condition, update, and body, but not after the `for`. When the start statement assigns to an existing binding, that binding remains visible after the `for`. The body creates a nested block scope.

`break;` exits the innermost loop. In a `while`, `continue;` proceeds directly to its condition. In a `for`, it proceeds to the update and then the condition. Neither may be used outside a loop.

```primer
while value < 10 {
    value = value + 1;

    if value < 3 {
        continue;
    }

    if value > 5 {
        break;
    }
}
```

Primer currently has no labeled `break` or `continue` for naming an outer loop.

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

Primer IR assigns deterministic IDs to bindings so references remain unambiguous when names are reused. Structured `if`, `while`, `for`, `break`, and `continue` statements remain visible in Primer IR. A `for` keeps its initializer, condition, body, and update as distinct parts. During lowering into Bytecode and backend IRs, structured loops become condition, body, update when applicable, and exit paths. `break` and `continue` become jumps to the correct path of their target loop.

## Types

Primer v0.1 has one boolean type, three numeric types, fixed arrays, and user-defined named product types:

```text
bool
i64
f32
f64
fixed arrays
named product types
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

`print(expression);` accepts the current boolean and numeric types. Select a field of a named product or an element of a fixed array before printing it.

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
