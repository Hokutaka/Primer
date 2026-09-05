# Primer CLI reference

[日本語](cli.ja.md)

This document defines the command-line interface of Primer v0.1.

## Commands

The current CLI provides the following commands:

```text
primer check <file>
primer emit-ir <file> [-o <output.pir>]
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [--target <triple>] [-o <output.ll>]
primer emit-wat <file> [-o <output.wat>]
primer emit-qbe <file> [--target <triple>] [-o <output.ssa>]
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

A successful `check` does not guarantee that every output route supports the program. Strings work through every route. Omitting the LLVM or QBE target diagnoses string-containing type definitions or expressions at their source location without producing an artifact. This diagnostic also leaves an existing file specified by `-o` unchanged.

## Primer IR emission

```text
primer emit-ir <file> [-o <output.pir>]
```

`primer emit-ir` emits the backend-independent Primer IR after semantic and type resolution.

## Output artifact emission

```text
primer emit-c <file> [-o <output.c>]
primer emit-llvm <file> [--target <triple>] [-o <output.ll>]
primer emit-qbe <file> [--target <triple>] [-o <output.ssa>]
primer emit-wat <file> [-o <output.wat>]
primer emit-asm <file> [-o <output.s>]
primer emit-bytecode <file> [-o <output.pbc>]
```

Each command emits the following artifact:

| Command | Output route | Current target | Artifact |
| --- | --- | --- | --- |
| `emit-c` | C | not selected by Primer | `.c` |
| `emit-llvm` | LLVM IR | unspecified, or explicit Windows x64 / Linux x86-64 | `.ll` |
| `emit-qbe` | QBE IR | unspecified, or explicit Linux x86-64 | `.ssa` |
| `emit-wat` | WebAssembly Text | WebAssembly | `.wat` |
| `emit-asm` | native assembly | x86-64, Windows, Windows x64 ABI | `.s` |
| `emit-bytecode` | Primer bytecode | Primer VM | `.pbc` |

Each `emit-*` command writes its observation to standard output by default. With `-o`, the caller chooses the output path.

The current `emit-asm` command has no target-selection option and emits assembly for x86-64 Windows.

### LLVM target selection

`--target` accepts `x86_64-unknown-linux-gnu` or `x86_64-pc-windows-msvc`. Programs using strings require it, including unused types and functions. Existing numeric-only invocations may omit it. The host OS never supplies a default. `--target` and `-o` (also `--output`) may appear in either order; duplicate options, missing values, and unsupported targets are errors.

On Linux x86-64:

```sh
primer emit-llvm examples/string_lookup.prim --target x86_64-unknown-linux-gnu -o target/string_lookup.ll
clang --target=x86_64-unknown-linux-gnu target/string_lookup.ll -o target/string_lookup
./target/string_lookup
```

On Windows x64 with the MSVC CRT and linker available:

```powershell
primer emit-llvm examples/string_lookup.prim --target x86_64-pc-windows-msvc -o target/string_lookup.ll
clang --target=x86_64-pc-windows-msvc target/string_lookup.ll -o target/string_lookup.exe
.\target\string_lookup.exe
```

Primer only generates LLVM; it does not launch Clang or the executable. Selection is recorded in `target triple`. Pass the same target to downstream tools. Windows programs containing strings initialize standard output in binary mode to preserve NUL, CR, and LF. See [string design](../design/strings.en.md#llvm-representation-and-targets).

Library callers can use `compile_to_llvm_with_target(source, Some(codegen::llvm::Target::X86_64UnknownLinuxGnu))`, or `X86_64PcWindowsMsvc`. Existing `compile_to_llvm(source)` remains the unspecified-target API.

### QBE target selection

QBE output containing strings requires `--target x86_64-unknown-linux-gnu`. Missing or unsupported targets and duplicate options produce diagnostics without changing existing output files. Existing numeric-only invocations may omit the target.

```sh
primer emit-qbe examples/string_lookup.prim --target x86_64-unknown-linux-gnu -o target/string_lookup.ssa
qbe -t amd64_sysv -o target/string_lookup.s target/string_lookup.ssa
cc target/string_lookup.s -o target/string_lookup
./target/string_lookup
```

The artifact records the target in a comment. Invoking QBE and the C linker belongs to the consumer. Library callers use `compile_to_qbe_with_target(source, Some(codegen::qbe::Target::X86_64UnknownLinuxGnu))`.

### WAT and direct assembly strings

`emit-wat` and `emit-asm` retain their existing fixed targets and need no additional target selection.

WAT using strings imports `primer.write_byte(i32) -> void`, passing each byte and a trailing LF without exposing memory. Alongside the existing numeric and Boolean host functions, the host implements the [string output contract](../design/strings.en.md#wat-output-and-the-external-boundary). `emit-wat` does not launch a host.

Generate Windows x64 direct assembly with `primer emit-asm examples/string_lookup.prim -o target/string_lookup.s` and build it with `clang --target=x86_64-pc-windows-msvc target/string_lookup.s -o target/string_lookup.exe`. Programs using strings switch standard output to binary mode before output.

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
