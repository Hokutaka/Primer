# Function design

[日本語](functions.ja.md)

Status: Implemented

This document records design decisions and observation guarantees for Primer functions, calls, returns, and entrypoints. Current syntax is defined by the [language reference](../reference/language.en.md).

## Purpose

A function is not only a reuse mechanism. In Primer, it is also a unit whose transformation can be followed from source through Primer IR, backend IR, and emitted artifacts.

The design preserves three properties:

- function names, parameters, calls, and returns remain traceable after transformation;
- backend ABI decisions do not become part of Primer semantics;
- observation identifiers do not grant authority to modify the compiler or live values.

## Current semantics

```primer
fn add(left: i64, right: i64) -> i64 {
    return left + right;
}
```

- `fn` explicitly introduces a function definition;
- parameters and results use explicit concrete types;
- a function without a result explicitly uses `void`;
- returning a value requires `return expression;`;
- parameters are immutable local bindings;
- named product types and fixed arrays may also be used as parameters and results;
- functions cannot read top-level runtime bindings;
- function names are registered file-wide, allowing forward calls.

An explicit main function must be `fn main() -> void`. A program with top-level executable statements receives a compiler-generated entrypoint, so the two forms cannot be combined.

## Values across function boundaries

Scalars, named product types, and fixed arrays all cross function boundaries as values.

```primer
type Point { x: i64, y: i64, }

fn move_x(point: Point, amount: i64) -> Point {
    return Point { x: point.x + amount, y: point.y, };
}

original: Point = Point { x: 2, y: 3, };
moved: Point = move_x(original, 5);
print(original.x); // 2
print(moved.x);    // 7
```

The `point` received by `move_x` is a value separate from the caller's `original`. A returned product or array also becomes a new value in the caller. The function and its caller never acquire an implicit shared mutable location.

## Observable information

| Stage | Preserved information |
| --- | --- |
| AST | Source names, type names, blocks, and spans |
| Primer IR | `FunctionId`, `BindingId`, resolved types, structured calls and returns |
| Bytecode | Function number, parameter slots, per-function instruction numbers, call argument count, return presence |
| Backend IR | Backend locals, temporaries, control flow, and call representation |
| Artifact | Function symbols, argument representation, stack or memory placement, concrete calls and returns |

A VM error retains both the function number and its instruction number. Instruction provenance maps the failure back to a source span without confusing it with an entrypoint instruction.

## Backend boundary

In this section, "aggregate value" means either a product value or a fixed array.

- C emits typed functions and prototypes and passes aggregates as C values.
- LLVM IR emits typed parameters and results and stores received values in observable local slots.
- QBE IR receives aggregate argument addresses and copies them into callee stack storage at function entry. An aggregate result uses a caller-provided destination as a hidden first argument.
- WebAssembly Text represents the same scheme with addresses in linear memory.
- Windows x86-64 lowers scalar arguments to the general-purpose or XMM registers selected by the Windows x64 ABI. Aggregate argument addresses use the positional general-purpose registers, while the internal Primer convention passes an aggregate result destination in `RAX`.
- Primer bytecode and the VM create an independent frame for every call and clone aggregate values into it.

Primer IR does not choose ABI registers, stack offsets, or hidden result destinations. Those are backend-lowering decisions visible in emitted artifacts. Addresses used internally by QBE, WebAssembly, and Windows x86-64 only implement copying; they do not define Primer reference types or an external ABI.

## Current limits

Functions currently accept at most four parameters. Scalars, named product types, and fixed arrays may be used as parameters and results.

Both direct and indirect recursion produce a diagnostic. The current WebAssembly backend does not yet separate product temporary memory per invocation, so enabling recursion could corrupt values in only some routes.

Before recursion is enabled, all of the following must hold:

- every backend has independent call frames;
- product locals and temporaries are invocation-local;
- observations distinguish call-stack frames and provenance;
- stack or memory consumption can have explicit limits.

## Security boundary

`FunctionId`, `BindingId`, and instruction numbers identify relationships within one compilation result. They are not handles for replacing functions, references for modifying live frames, or authority to write back into compiler state.

Aggregate addresses visible in emitted artifacts have the same boundary. A Primer program cannot extract or retain one as a value. A callee immediately copies an aggregate argument into its own storage, so the address does not become a path for modifying the caller's value.

Future foreign functions or plugins must keep observation APIs separate from execution and mutation authority. Enabling observation alone must never alter call targets or generated artifacts.
