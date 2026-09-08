# Expected runtime failures

[日本語](README.md)

These three examples must stop for the specified reason. They are excluded from the root directory's successful-example batch.

| Example | Output before failure | Reason and source location |
| --- | --- | --- |
| [overflow.prim](overflow.prim) | `カウンタを更新` | `integer-overflow`, `counter + 1` |
| [array_update.prim](array_update.prim) | `4` | `array-index-out-of-bounds`, assignment's `[2]`; the right-hand side does not execute |
| [function_division.prim](function_division.prim) | `false`, `除算を開始` | `division-by-zero`, `value / divisor` inside the function; the first call is short-circuited |

```sh
cargo run -- run examples/runtime_failures/array_update.prim --diagnostic-format runtime-v1
```

The VM exits with code 1 and emits one stderr line: `primer: runtime-v1 code=... node=... bytes=.....`. The byte range addresses the original UTF-8 source, with an exclusive end. Use the NodeId and range to locate the operation in `emit-ir`. Changing CRLF/LF changes byte offsets.

Compare with the internal encoder on Windows, using a new output directory:

```powershell
node scripts/observe-native.cjs --source examples/runtime_failures/array_update.prim --target x86_64-pc-windows-msvc --primer target/debug/primer.exe --cc clang --objdump llvm-objdump --output-dir target/observe-array-failure --encoder primer --run --expect-trap
```

On Linux explicitly select `x86_64-unknown-linux-gnu`, `target/unix/debug/primer`, `cc`, and `objdump`. Select `--encoder external` to use an external assembler. Verification requires matching reasons, NodeIds, byte ranges, prior output, and the intended native illegal-instruction termination. See the [runtime diagnostic contract](../../docs/design/runtime-diagnostics.en.md).
