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
    "for" "(" (binding_clause | binding_assignment_clause) ";"
    expression ";" binding_assignment_clause ")" block

block       := "{" statement* "}"

binding     := "mut"? IDENT ":" type_spec "=" expression ";"

binding_clause := "mut"? IDENT ":" type_spec "=" expression

assignment  := assignment_target "=" expression ";"

assignment_target := IDENT ("[" expression "]")*

binding_assignment_clause := IDENT "=" expression

type_spec   := "i32"
             | "u32"
             | "i64"
             | "f32"
             | "f64"
             | "bool"
             | fixed_array_type
             | IDENT
             | "infer"

type_ref    := "i32" | "u32" | "i64" | "f32" | "f64" | "bool" | fixed_array_type | IDENT

fixed_array_type := "[" type_ref ";" INTEGER "]"

expression  := logical_or

logical_or  := logical_and ("||" logical_and)*

logical_and := equality ("&&" equality)*

equality    := comparison (("==" | "!=") comparison)*

comparison  := additive (("<" | "<=" | ">" | ">=") additive)*

additive    := multiply (("+" | "-") multiply)*

multiply    := unary (("*" | "/") unary)*

unary       := ("-" | "!") unary
             | postfix

postfix     := primary (("." IDENT) | ("[" expression "]"))*

primary     := "true"
             | ("i32" | "u32" | "i64") "(" expression ","? ")"
             | "convert" "<" type_ref ">" "(" expression ","? ")"
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

An element type may be `bool`, `i32`, `u32`, `i64`, `f32`, `f64`, a named product type, or another fixed array. A fixed array may also be used as a field of a product type.

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

Fixed arrays may be nested directly. In `matrix[row][column]`, the two indices are checked in sequence.

```primer
matrix: [[i64; 3]; 2] = [[1, 2, 3], [4, 5, 6]];
print(matrix[1][2]); // 6
```

Fixed arrays may be used as function parameters and results. They remain values and are copied across the function boundary.

An element of a `mut` array can be updated with `values[index] = value;`. Nested arrays support forms such as `matrix[row][column] = value;`. Indices are evaluated from left to right and each is bounds-checked immediately. The right-hand side is evaluated only after every check succeeds, followed by one write. If a check fails, the right-hand side is not evaluated and the array is unchanged.

Updating one copy of an array does not change another copy. The assigned value must have the declared element type. Updating through an immutable binding is an error, just like reassigning the complete array.

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

Function parameters and results may use `bool`, `i32`, `u32`, `i64`, `f32`, `f64`, named product types, and fixed arrays. Products and arrays are passed as values, so the received value and the caller's value do not share a mutable location. Functions accept at most four parameters. Recursion and command-line arguments are not yet supported. Unsupported forms produce diagnostics instead of silently changing meaning.

Primer IR and bytecode expose function IDs, parameter binding IDs, calls, and returns. Backend artifacts expose how those entities become function symbols, arguments, local storage, and ABI registers or memory. See [Function design](../design/functions.en.md) for details.

## Mutable bindings and reassignment

A binding that needs to change is declared with `mut` before its name:

```primer
mut count: i64 = 40;
count = count + 2;
print(count);
```

`mut` is not a type. It specifies that the name `count` may be reassigned. For arrays, it also permits element updates through that binding. Reassignment does not contain `: type_spec`; this distinction separates a new declaration from assignment to an existing binding.

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

Primer v0.1 has one boolean type, five numeric types, fixed arrays, and user-defined named product types:

```text
bool
i32
u32
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
i32       int64_t
u32       int64_t
i64       int64_t
f32       float
f64       double
```

Integer range and storage width are separate. `i32` represents `-2147483648` through `2147483647`, and `u32` represents `0` through `4294967295`, but generated targets currently store both in 64-bit locations. Arrays and products do not yet pack these values into 32 bits, so changing from `i64` does not currently reduce memory use.

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

## Combining conditions with logical operators

`&&` returns `true` when both operands are `true`; `||` returns `true` when at least one is `true`. Both operands must have type `bool`, and the result is `bool`. Numbers such as 0 and 1 are not treated as Booleans.

Evaluate the left operand once, then decide whether to evaluate the right operand. This is short-circuit evaluation:

| Operation | Evaluate the right operand when | Result when skipped |
| --- | --- | --- |
| `left && right` | The left operand is `true` | `false` |
| `left \|\| right` | The left operand is `false` | `true` |

```primer
count: i64 = 0;
print(count != 0 && 12 / count > 2); // false; the division is not executed
values: [i64; 2] = [4, 9];
index: i64 = 2;
print(index == 2 || values[index] == 9); // true; the out-of-bounds element is not read
```

When needed, the right operand is evaluated exactly once. Calls, effects such as `print`, and runtime checks in a skipped operand are not executed. Failure in the left operand or a required right operand still stops execution normally; errors are not suppressed.

Name resolution and type checking still apply to both operands. `false && missing` and `true || 1` are compile errors even though the right operand would not execute.

Precedence from strongest to weakest is unary operations, multiplication/division, addition/subtraction, ordering comparisons, equality comparisons, `&&`, then `||`. Thus `a < b && c == d || ready` means `((a < b) && (c == d)) || ready`. Repeated operators associate to the left; parentheses change grouping. `a < b < c` is not a range comparison.

Logical operators work anywhere a `bool` expression is accepted, including bindings, function arguments/results, array elements, and product fields, not only conditions. The [short-circuit example](../../examples/short_circuit.prim) includes array traversal.

Primer IR retains `and.short_circuit.bool` and `or.short_circuit.bool`. Lowering uses conditional jumps in bytecode, `&&`/`||` in C, branches in LLVM/QBE/Windows x86-64, and a Boolean-producing `if` in WAT. It never evaluates the right operand eagerly before selecting a result.

## Numeric literals

Integer literals use an explicit suffix if present, otherwise the expected integer type from context, and default to `i64` only when no type information is available.

```primer
x: i64 = 42;
```

An integer literal remains a sequence of decimal digits until its type is known. Once its type is resolved, that type's range is checked and an out-of-range value is a compilation error. The sign is parsed as unary `-`, but `-9223372036854775808` is accepted as the minimum `i64` value.

Integer suffixes are `i32`, `u32`, and `i64`. Unsuffixed numbers receive expected types from declarations, assignments, arguments, returns, fields, and array elements. Without an outer expected type, already typed values in the same arithmetic expression supply the type.

```primer
count: i32 = 4;
first: infer = count + 1;
second: infer = (1 + 2) + count;
explicit: infer = 3000000000u32;
default: infer = 1 + 2; // no type information, so i64
```

`first` and `second` have type `i32`, independent of operand order. Already typed variables and suffixed literals are not reinterpreted; operations between different types fail type checking. Integer literals are not reinterpreted as floating-point values. Comparison results are `bool`, separately from their integer operand types.

An array declared with `infer` still derives its element type from the first element. `[1i32, 2]` is `[i32; 2]`, but `[1, 2i32]` is an error because the first element defaults to `i64`. An explicitly typed `[i32; 2]` accepts `[1, 2]`. Array indices remain `i64`; use `values[i64(index)]` for a `u32` position.

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
i32 op i32 -> i32
u32 op u32 -> u32
i64 op i64 -> i64
f32 op f32 -> f32
f64 op f64 -> f64
```

Integer `+`, `-`, `*`, and signed integer unary `-` stop execution when their result is outside that integer type's range. They do not silently wrap from one end of the range to the other. Integer division by zero and division of the minimum signed integer value by `-1` also stop execution. Unary minus is rejected for `u32`, including `-0u32`. Integer division rounds toward zero.

The Primer VM diagnoses the failing operation kind, type, bytecode instruction index, and source location. Generated C, LLVM IR, QBE IR, WebAssembly Text, and Windows x86-64 assembly retain corresponding checks or traps, making the enforcement point observable.

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

## Explicit integer conversions

Integer conversion has two equivalent spellings:

```primer
value: i64 = 42;
compact: infer = i64(value);
explicit: infer = convert<i64>(value);
```

All pairs among `i32`, `u32`, and `i64` support conversion. Conversion succeeds only if the same numerical value fits the destination; otherwise execution stops. It does not truncate or wrap. Conversions involving floating-point values, `bool`, arrays, or product types are not supported.

```primer
count: u32 = 3000000000;
wide: i64 = i64(count);
back: u32 = convert<u32>(wide);
```

Every `i32` fits in `i64`, but `u32` to `i32` can fail despite equal bit widths. VM conversion failures retain both source and destination types.
`value: i32 = 2147483648;` is a compile-time literal error; `i32(2147483648)` evaluates an `i64` value and then fails conversion at runtime.

Both spellings evaluate the expression inside parentheses exactly once. The destination type is not passed into the input expression to change its arithmetic. A noninteger input is a compile-time error. Exactly one argument is required; a trailing comma is allowed. Conversion produces a value and cannot be used as a standalone statement.

If input evaluation fails, the diagnostic points to that operation. For example, `i64(1 / 0)` stops at division before conversion is reached.

Primer IR retains the source and destination integer types, input, original spelling, and source location. Both spellings use the same operation kind; spelling is origin information. Bytecode emits both types, for example `convert.checked i32 -> u32`, with the corresponding Primer IR `NodeId` and `Span` as the instruction origin. C, LLVM, QBE, WAT, and Windows x86-64 retain the input in 64-bit storage and generate destination range checks for conversions to `i32` and `u32`. Conversion to `i64` needs no additional execution operation, while the explicit conversion remains in Primer IR and bytecode.

Functions and types cannot be defined with the built-in type names `bool`, `i32`, `u32`, `i64`, `f32`, or `f64`; these are diagnosed at the definition. `convert` is not a keyword: ordinary calls such as `convert(value)` and comparisons such as `convert < limit` remain available. The `convert<type>(expression)` form is a built-in conversion whose meaning does not change when a user function named `convert` exists. This form does not introduce user-defined generic functions.

## Output

`print(expression);` accepts the current boolean and numeric types. Select a field of a named product or an element of a fixed array before printing it.

Primer keeps floating-point output precise enough to expose the behavior being observed.

The current formatting policy is:

```text
bool   `true` or `false`
i32 / u32 / i64    integer output
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
