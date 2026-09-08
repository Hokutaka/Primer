# Primer's x86-64 encoder

[日本語](native-encoder.ja.md)

## Scope

`emit-obj` encodes x86-64 instructions and writes Linux ELF64 or Windows COFF objects entirely within Primer. It does not require an external assembler or launch a JIT or executable. Linking remains the responsibility of an explicitly selected external linker.

```text
Primer IR → shared x86-64 instructions → internally generated ASM
                                          ├→ emit-asm → external assembler
                                          └→ restricted ASM reader → encoder → ELF/COFF
```

Language semantics and instruction selection are shared with the existing path. This first boundary is a small assembler for internally generated ASM, rather than a public general-purpose assembler; there is no arbitrary-ASM input command. It does not repeat type inference, failure checks, evaluation order, or value-copy rules. A future typed machine-instruction IR can replace this internal text boundary.

The encoder covers the forms needed by all current language features: eight integer kinds, f32/f64, booleans, strings, arrays, products, functions, and control flow. General x86-64 assembly, AVX, short-branch optimization, and an internal linker are outside its scope. Newly generated unsupported instruction forms produce diagnostics; there is no automatic external-assembler fallback.

## Encoding and relocations

- Instructions contain the required legacy prefixes, REX, ModR/M, SIB, displacements, and immediates. Register classes and widths, address scales, and immediate ranges are checked.
- Local branches use fixed 32-bit relative displacements. References within `.text` are resolved; displacements outside their representable range are rejected.
- Constants and strings occupy a read-only section. ELF uses PC32 for RIP-relative data and PLT32 for external calls; COFF uses REL32. Addends account for immediates following a displacement.
- ELF includes an empty `.note.GNU-stack` and does not request executable stack memory. COFF timestamps are fixed at zero. Symbol and section ordering is deterministic.
- COFF supports up to 65,535 relocations. Positions, values, and displacements are not silently truncated into narrower fields.

Primer may use longer immediate/branch forms and single-byte NOP alignment. Instruction bytes and addresses can differ from external assemblers. Comparisons establish language behavior and execution results. Identical input, target, and options produce identical Primer objects.

`--annotate-origins` retains source-origin labels as symbols. It does not change `.text`, constant/string data, or relocation meaning. The symbol table itself changes. Generated objects are observations, not interfaces for external mutation of compiler state.

## Commands

Both `--target` and `-o` are required so binary objects are never implicitly printed to a terminal. Output is a relocatable object, not an executable.

```powershell
cargo build
target/debug/primer.exe emit-obj examples/packet_counter.prim --target x86_64-pc-windows-msvc --annotate-origins -o target/packet.obj
clang target/packet.obj -o target/packet.exe
target/packet.exe
```

```sh
cargo build
target/debug/primer emit-obj examples/packet_counter.prim --target x86_64-unknown-linux-gnu --annotate-origins -o target/packet.o
cc target/packet.o -o target/packet
target/packet
```

With `CARGO_TARGET_DIR=target/unix` in WSL, use `target/unix/debug/primer`. Encoding can target either OS from either host. Execution and linked libraries must match the chosen target.

Select `--encoder primer` in the observation script; the existing `external` default remains available for comparison.

```powershell
node scripts/observe-native.cjs --source examples/packet_counter.prim --target x86_64-pc-windows-msvc --primer target/debug/primer.exe --cc clang --objdump llvm-objdump --output-dir target/packet-own-windows --encoder primer --run
```

`manifest.json` records `encoder: primer` and an `encode-object` stage. The C driver performs linking only. IR, corresponding ASM, the Primer object, disassembly, and VM output comparison are saved through the [native observation workflow](native-code.en.md).

## Validation

All examples, u64/string boundaries, and large aggregate arguments and returns are linked and executed on Windows/Linux against the VM and external-assembler path. Expected illegal-instruction traps are checked for both encoders. Lower-level tests cover REX/SIB, displacement boundaries, SSE2, local branches, and RIP-relative addends. Instruction-byte expectations were cross-checked with the independent LLVM assembler. CLI tests clear PATH to verify object generation requires no external tools.

References: [Intel instruction manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html), [Microsoft PE/COFF specification](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format), [ELF64 format](https://uclibc.org/docs/elf-64-gen.pdf), and [x86-64 psABI](https://gitlab.com/x86-psABIs/x86-64-ABI).
