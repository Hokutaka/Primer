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
- functions cannot read top-level runtime bindings;
- function names are registered file-wide, allowing forward calls.

An explicit main function must be `fn main() -> void`. A program with top-level executable statements receives a compiler-generated entrypoint, so the two forms cannot be combined.

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

- C emits typed functions and prototypes.
- LLVM IR exposes typed parameters and stores them in observable local slots.
- QBE IR exposes function parameters and stack allocation.
- WebAssembly Text emits typed parameters/results and `call` instructions.
- Windows x86-64 lowers argument positions into general-purpose or XMM registers under the Windows x64 ABI.
- Primer bytecode and the VM create an independent frame for each call.

Primer IR does not choose ABI registers or stack offsets. Those are backend-lowering decisions visible in emitted artifacts.

## Current limits

Function signatures currently use scalar types and at most four parameters. This keeps the first ABI implementation consistent across every output route. A function body may still use named product types and fixed arrays as local values.

Both direct and indirect recursion produce a diagnostic. The current WebAssembly backend does not yet separate product temporary memory per invocation, so enabling recursion could corrupt values in only some routes.

Before recursion is enabled, all of the following must hold:

- every backend has independent call frames;
- product locals and temporaries are invocation-local;
- observations distinguish call-stack frames and provenance;
- stack or memory consumption can have explicit limits.

## Security boundary

`FunctionId`, `BindingId`, and instruction numbers identify relationships within one compilation result. They are not handles for replacing functions, references for modifying live frames, or authority to write back into compiler state.

Future foreign functions or plugins must keep observation APIs separate from execution and mutation authority. Enabling observation alone must never alter call targets or generated artifacts.
