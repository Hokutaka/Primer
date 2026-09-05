[![CI](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml/badge.svg)](https://github.com/Hokutaka/Primer/actions/workflows/ci.yml)

# Primer

[日本語](README.md) | English

Primer is an experimental programming language designed to make compiler transformations observable.

Beyond the result, Primer makes it possible to inspect which types a computation uses and how it becomes generated code. A shared Primer IR (intermediate representation) holds the resolved meaning and types before lowering to each output target. Primer aims to combine sophisticated implementation with observability, while keeping inspection separate from mutation of compiler internals.

## Run Your First Example

You need a Rust development environment with rustup and Cargo. Clone the repository and install the CLI from its root:

```sh
git clone https://github.com/Hokutaka/Primer.git
cd Primer
cargo install --path .
```

[examples/floating_point.prim](examples/floating_point.prim) performs the same addition with different types:

```primer
a: f32 = 0.1 + 0.2;
b: f64 = 0.1 + 0.2;
c: infer = 0.1 + 0.2;

print(a);
print(b);
print(c);
```

```sh
primer run examples/floating_point.prim
```

Primer VM output:

```text
0.300000012
0.30000000000000004
0.30000000000000004
```

`f32` and `f64` represent numbers with different precision. `infer` explicitly requests type inference; `c` resolves to `f64` in this example.

During development, replace `primer` with `cargo run --quiet --` to run the updated code without reinstalling the CLI.

## Observe Computation and Transformation

The same source can be inspected as an intermediate representation or generated code, not just executed:

```sh
primer emit-ir examples/floating_point.prim
primer emit-c examples/floating_point.prim
```

`emit-ir` shows resolved types and operations; `emit-c` shows how they are represented in C. Backends consume the shared Primer IR instead of interpreting the source semantics again.

`emit-*` writes to standard output. To keep an artifact, use, for example, `primer emit-c examples/floating_point.prim -o floating_point.c`. To check syntax and types without running, use `primer check examples/floating_point.prim`.

The public observation points are Primer IR and emitted artifacts. Backend-specific Rust IR remains an internal lowering boundary. See the [compiler design](docs/design/architecture.en.md) and [observability contract](docs/design/observability.en.md) for details.

## Current Capabilities

- **Types and variables:** static typing; `bool`; `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`; `f32`, `f64`; `string`. Type declarations, `infer`, immutable bindings, and mutable bindings with `mut`.
- **Data structures:** named structs (product types), default field values and field access, and nestable fixed arrays. Value copies and array-element updates.
- **Functions and control flow:** typed functions, `void`, and explicit `return`. Top-level executable statements or `fn main() -> void`. `if` / `else`, `while`, `for`, and `break` / `continue`.
- **Operators:** arithmetic, integer remainder and bit operations, comparisons, `!`, and short-circuiting `&&` and `||`.
- **Explicit conversion:** equivalent spellings such as `f64(value)` and `convert<f64>(value)`. Conversion between implemented numeric types succeeds only when it preserves the value.
- **Output and execution:** `print(expr);`, Primer IR and backend artifact emission, and Primer VM execution.

Integer overflow, invalid integer division, out-of-bounds array access, and conversions that cannot preserve the value stop execution. There are no implicit numeric conversions. Ordinary floating-point arithmetic still rounds.

Strings are immutable UTF-8 values, supporting printing, equality, UTF-8 byte-length queries, and use in functions and data structures. They work through every output route. LLVM and QBE require an explicit runtime `--target`. [LLVM target selection](docs/reference/cli.en.md#llvm-target-selection) supports Windows x64 and Linux x86-64. QBE targets Linux x86-64, direct assembly targets Windows x64, and WAT uses a WebAssembly output host function. See [string design](docs/design/strings.en.md) for representation differences. Concatenation and string indexing are not implemented.

`u64`, dynamically sized arrays, recursion, failure recovery, and explicit rounding/truncation operations are not implemented. Current generated targets store even small integer types in 64-bit storage and check their value ranges.

### Output Targets

| Command | Artifact | Next step |
| --- | --- | --- |
| `emit-c` | C (`.c`) | Compile with GCC, Clang, or another C compiler |
| `emit-llvm` | LLVM IR (`.ll`) | Compile with LLVM / Clang |
| `emit-qbe` | QBE IR (`.ssa`) | Process with QBE |
| `emit-wat` | WebAssembly Text (`.wat`) | Run using WebAssembly tools and a host |
| `emit-asm` | Windows x86-64 assembly (`.s`) | Assemble and link |
| `emit-bytecode` | Primer bytecode (`.pbc`) | Inspect instructions; use `run` on source for VM execution |

Primer handles artifact generation. External tool selection, CPU targets, optimization settings, and measurement policy belong to the caller. See [output routes and targets](docs/design/targets.en.md) for details.

## Examples and Documentation

| Category | Examples |
| --- | --- |
| Basics | [Small-value output](examples/small_values.prim), [short-circuit evaluation](examples/short_circuit.prim) |
| Data structures | [Ring buffer](examples/ring_buffer.prim), [passing structs and arrays](examples/function_values.prim) |
| Numerical computation | [Sample mean and variance](examples/measurement_statistics.prim), [learning a line](examples/linear_regression.prim) |
| Algorithms | [Shortest paths](examples/shortest_paths.prim), [bitset subset sum](examples/subset_sum_bits.prim) |

Find more in the [examples index](examples/README.en.md). Run all examples from the repository root.

PowerShell:

```powershell
.\scripts\run-examples.ps1
```

WSL / Bash (WSL also needs its own Rust development environment):

```bash
bash scripts/run-examples.sh
bash scripts/test.sh
```

`run-examples` displays sample output. Select examples with `-Pattern "matrix*.prim"` in PowerShell or `--pattern 'matrix*.prim'` in Bash. `test.sh` runs fmt, Clippy, and all test targets, including expected-output checks. Use `cargo test --test examples` for sample tests alone.

The `.sh` scripts default to `target/unix`, keeping Linux build output separate from Windows artifacts. They respect an existing `CARGO_TARGET_DIR` setting.

- [Language reference](docs/reference/language.en.md): current syntax, types, operators, and conversion rules.
- [CLI reference](docs/reference/cli.en.md): commands and options.
- [Documentation index](docs/README.md): Japanese and English guides, with design decisions in `docs/design/` and current specifications in `docs/reference/`.

## Related Tools

- [Tint\*](https://github.com/Hokutaka/Tint-St.): a development and inspection environment that presents source and generated representations side by side.
- [Whitebase](https://github.com/Hokutaka/Whitebase): an experiment environment that runs, measures, and compares built-in Rust, C++, and Assembly operations. Integration with Primer artifacts is not implemented yet.

Primer owns language semantics and compilation. The intended Whitebase integration keeps experiments using emitted artifacts on the consumer side. See [Tool responsibilities](docs/design/architecture.en.md#tool-responsibilities) for the current implementation and integration boundary.

## License

Licensed under the [MIT License](LICENSE).
