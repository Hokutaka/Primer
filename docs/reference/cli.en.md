# Primer CLI reference

[日本語](cli.ja.md)

This document defines the command-line interface of Primer v0.1.

## Commands

The current CLI provides the following commands:

```text
primer check <file>
primer emit-ir <file> [-o <output.pir>]
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [-o <output.ll>]
primer emit-wat <file> [-o <output.wat>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
primer run <file>
primer --version
```

## Validation

```text
primer check <file>
```

`primer check` parses the input source file and performs semantic validation and type checking.

A successful `check` does not guarantee that every output route supports the program. Strings currently work with `check`, `emit-ir`, `emit-bytecode`, and `run`. C, LLVM, QBE, WAT, and assembly emission diagnose string-containing type definitions or expressions at their source location without producing an artifact. This diagnostic also leaves an existing file specified by `-o` unchanged.

## Primer IR emission

```text
primer emit-ir <file> [-o <output.pir>]
```

`primer emit-ir` emits the backend-independent Primer IR after semantic and type resolution.

## Output artifact emission

```text
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [-o <output.ll>]
primer emit-qbe <file> [-o <output.ssa>]
primer emit-wat <file> [-o <output.wat>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
```

Each command emits the following artifact:

| Command | Output route | Current target | Artifact |
| --- | --- | --- | --- |
| `emit-c` | C | not selected by Primer | `.c` |
| `emit-llvm` | LLVM IR | not selected by Primer | `.ll` |
| `emit-qbe` | QBE IR | not selected by Primer | `.ssa` |
| `emit-wat` | WebAssembly Text | WebAssembly | `.wat` |
| `emit-asm` | native assembly | x86-64, Windows, Windows x64 ABI | `.s` |
| `emit-bytecode` | Primer bytecode | Primer VM | `.pbc` |

Each `emit-*` command writes its observation to standard output by default. With `-o`, the caller chooses the output path.

The current `emit-asm` command has no target-selection option and emits assembly for x86-64 Windows.

## Execution

```text
primer run <file>
```

`primer run` lowers the program to Primer bytecode and executes the resulting `BytecodeProgram` in the Primer VM.

Runtime output is useful for validation and experiments, but it is distinct from the two compiler observation boundaries defined in the [compiler design](../design/architecture.en.md).

When a runtime error occurs in a bytecode instruction derived from source, the diagnostic includes both the source location and the bytecode instruction index:

```text
primer: cannot divide an integer by zero at 1:7 (bytecode instruction 0002)
```

The bytecode instruction index is still displayed when no source location is available. Compact diagnostics do not include source text or the input file path.

## Version

```text
primer --version
```

`primer --version` prints the Primer version.

## External settings not controlled by Primer

Primer does not choose external experiment policy such as:

- GCC versus Clang;
- optimization levels for external compilers;
- CPU targets for external toolchains;
- benchmark settings;
- measurement policy;
- comparison policy.

Those choices belong to the caller and should be recorded when necessary.
