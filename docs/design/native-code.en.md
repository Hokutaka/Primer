# Tracing assembly through machine code

[日本語](native-code.ja.md)

## Language semantics and runtime targets

Direct Linux x86-64 assembly supports all eight integer kinds including u64, f32/f64, booleans, strings, arrays, products, functions, control flow, and exact conversions. It shares instruction representation and arithmetic with Windows in `codegen::x86_64`. Targets select calling conventions, output, sections, and stack allocation.

`emit-asm` accepts `--target x86_64-unknown-linux-gnu` and `--target x86_64-pc-windows-msvc`. Omission retains the fixed Windows default. The host OS never selects the output target.

SysV allocates integer and floating-point argument registers separately and passes the XMM argument count in AL for variadic printf calls. Windows uses positional registers, shadow space, and `__chkstk` when needed. Linux also touches each page during large stack allocations. Products and arrays retain independent value copies. Aggregate result storage is passed in RAX under Primer's internal convention; external C ABI compatibility is not guaranteed.

Strings remain immutable static length-prefixed data. Linux passes byte values to putchar; Windows enables binary stdout first. Japanese, NUL, CR, and LF survive without normalization.

## Observation stages

```text
Primer source → Primer IR → shared x86-64 instructions
                                       ↓
                           assembly with source origins
                                       ↓ Primer encoder or explicit external assembler
                           ELF/COFF object, bytes, relocations
                                       ↓ explicit external linker
                           executable machine code → comparison
```

Machine artifacts reuse the same assembly rather than reimplementing language semantics. Select the [Primer encoder](native-encoder.en.md) for encoding and object generation with `--encoder primer`, or keep the default `--encoder external`. Linking uses an explicitly selected external tool. The `emit-*` commands still return artifacts; only the explicitly invoked script starts external tools.

`--annotate-origins` adds `# primer-asm-origins v1`, `# primer-origin: #N bytes start..end`, and `primer_origin_nN_...` labels. Lowering retains NodeId and UTF-8 byte ranges. Constants, helpers, and startup are synthetic. Object symbols connect disassembly offsets to IR expressions. Removing annotations and origin labels restores ordinary assembly text. Observation exposes no memory mutation interface.

## Commands

On Linux with Rust, Node, cc, and objdump:

```sh
cargo build
node scripts/observe-native.cjs --source examples/native_values.prim --target x86_64-unknown-linux-gnu --primer target/debug/primer --cc cc --objdump objdump --output-dir target/native-demo-linux --run
```

On Windows with Rust, Node, Clang, MSVC CRT/linker, and llvm-objdump:

```powershell
cargo build
node scripts/observe-native.cjs --source examples/native_values.prim --target x86_64-pc-windows-msvc --primer target/debug/primer.exe --cc clang --objdump llvm-objdump --output-dir target/native-demo-windows --run
```

WSL users setting `CARGO_TARGET_DIR=target/unix` should select `--primer target/unix/debug/primer`. The output directory must be new and its parent must exist. Existing artifacts are never overwritten. Omit `--run` to generate and inspect without execution. Execution requires a matching host and explicit target. The script checks the assembler's object format too.

| Artifact | Contents |
| --- | --- |
| `source.prim` / `program.pir` | input, types, expressions, NodeIds |
| `program.s` | target-specific assembly with origins |
| `program.o` / `program.obj` | instruction bytes and unresolved relocations |
| `object.txt` / `text.txt` | sections, symbols, relocations, disassembly, and .text bytes |
| `program` / `program.exe`, `executable.txt` | linked executable and disassembly |
| `vm.stdout` / `native.stdout`, corresponding stderr | outputs captured by `--run` |
| `manifest.json` | schema, target, tool versions, arguments, stage outcomes, artifact SHA-256 hashes |

Object locations are section offsets; executable locations are link-time addresses, not runtime ASLR addresses. A linker may discard local origin symbols. Object-stage correspondence is retained even when names disappear from the executable. Whole-binary identity is not guaranteed across tool versions, formats, and linking environments.

## Success versus expected termination

Ordinary execution requires successful VM/native exits, empty stderr, and matching stdout. String output is byte-exact. Existing Windows numeric-only CRT output uses explicitly recorded CRLF-to-LF comparison.

`--run --expect-trap` requires matching VM/native `runtime-v1` records (reason, NodeId, byte range), matching prior stdout, and native SIGILL/Windows illegal-instruction termination. Successful execution, access violations, startup failures, and timeouts do not pass. Manifest outcomes distinguish `output-matched`, `expected-failure-confirmed`, `generated-not-executed`, and `failed`; `runtimeFailure` stores the matched record. See the [common diagnostic contract](runtime-diagnostics.en.md). Each tool has a 30-second timeout.

C u64 checks emit reasons to stderr, which tests compare against the corresponding VM error category. Windows abort and unrelated fast-fail conditions can share an exit code, so the code alone is insufficient. See Microsoft's [abort](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/abort) and [fast-fail](https://learn.microsoft.com/en-us/cpp/intrinsics/fastfail) specifications. QBE requires SIGABRT, LLVM illegal-instruction termination, and WAT unreachable. Extending common records to C, LLVM, QBE, and WAT remains future work.

`cargo test --test native_assembly` checks every example, string/u64 boundaries, four mixed arguments, large frames, copies, origins, and expected termination. Machine artifact tests require Node, a C driver, and objdump. Configure `PRIMER_TEST_NODE`, `PRIMER_TEST_CC` on Linux, `PRIMER_TEST_ASM_CLANG` on Windows, and `PRIMER_TEST_OBJDUMP`. CI requires execution on both operating systems.
