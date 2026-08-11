# Primer design notes

Primer is intentionally a small source-to-C language.

## Principle

Primer should preserve observability over cleverness.

The compiler should avoid source-level optimization unless a future experiment explicitly adds an optimization pass. This keeps the transformation boundary visible:

1. Primer source expresses the program.
2. Primer parses and validates it.
3. Primer emits straightforward C.
4. An external C compiler owns optimization and machine-code generation.
5. Consumers such as Whitebase can compare each stage independently.

## v0.1 grammar

```text
program     := statement* EOF
statement   := "let" IDENT "=" expression ";"
             | "print" "(" expression ")" ";"
expression  := additive
additive    := multiply (("+" | "-") multiply)*
multiply    := unary (("*" | "/") unary)*
unary       := "-" unary | primary
primary     := INTEGER | IDENT | "(" expression ")"
```

All values are signed 64-bit integers in v0.1.

Bindings are immutable and may only refer to bindings declared earlier in the file.

## CLI contract

```text
primer check <file>
primer emit-c <file> [-o <output.c>]
primer --version
```

`emit-c` is the important integration boundary. Primer does not select GCC/Clang, optimization levels, CPU targets, benchmark settings, or execution policy.

## Whitebase integration

Whitebase should treat Primer as an external installed tool, not as a source dependency.

A future adapter can:

1. detect `primer` on PATH;
2. record `primer --version`;
3. invoke `primer emit-c` for a benchmark source;
4. compile the generated C with explicitly recorded compiler flags;
5. measure and inspect the resulting native code.
