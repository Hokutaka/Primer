# Source locations with file identity

[日本語](source-files.ja.md)

## Use in modules

This foundation now supports [imports, namespaces, and visibility](modules.en.md), with cross-route CLI execution comparisons. `examples/source_files` remains a low-level API origin test combining AST items; `examples/modules` demonstrates real imports.

## Identity and contents

`SourceMap` stores caller-provided display names and UTF-8 text, assigning `SourceId` values starting at 1 in registration order. It performs no file I/O, path normalization, or Unicode normalization. Duplicate names do not overwrite existing files. ID 0 is reserved for the existing anonymous single-source APIs, including `compile(&str)` and CLI inputs without import/pub.

`Span` carries a file ID and a file-local byte range with an exclusive end. Japanese text, NUL, and CR/LF remain intact; files are not concatenated into virtual offsets. Resolution checks both UTF-8 boundaries and refuses to associate unknown files or invalid ranges with another source.

IDs are valid only within the same map and compilation. They are neither persistent identities nor content hashes. A future module loader must register files deterministically and retain the file inventory and content identity associated with generated artifacts.

## Parsing, diagnostics, and generation

- `lexer::lex_source` attaches file identity to tokens and lexical errors, including invalid string escapes.
- The parser preserves identity in all composed spans and rejects token streams mixing files.
- `compile_source` and `compile_source_to_ir` accept registered files. AST, common IR, bytecode, and all backends retain their ranges.
- `run_bytecode` returns structured origins and distinguishes VM internal errors from language failures.
- `render_compact_with_sources` resolves display names, lines, and columns through the map. It escapes control characters and falls back to numeric file/byte positions for unresolved spans.

Generated programs need no embedded paths or source text. Registered-file failure records add `file=N` between `node` and `bytes` in [runtime-v1](runtime-diagnostics.en.md); absence means anonymous source 0. Existing single-source records are unchanged. LLVM and assembly origin comments use the same optional field.

File IDs are immutable diagnostic data. There is no shared mutable current-file variable or external mutation API. Target selection remains explicit.

## Examples and verification

The [source_files example](../../examples/source_files/README.en.md) covers maximum `u64`, products, fixed arrays, independent copies, default string values, Japanese/NUL/CR/LF bytes, short circuiting, and division failure inside a function.

`tests/source_files.rs` compares known results and separate-file ASTs against a single source through VM, C, LLVM, QBE, WAT, direct assembly, and internal objects. C and LLVM run with and without optimization. It checks that callee failures refer to the definition file, and invalid array assignment targets prevent execution of the right-hand function. Unavailable tools are reported as skipped; explicitly configured tools are required. Tests explicitly select the Windows target on Windows and Linux on Linux; QBE execution is tested on Linux.

## Current module boundary

Explicit imports, function/type visibility, and diagnostics for cycles, name conflicts, and private access are implemented. Loading alone executes no initialization. Package distribution, dynamic loading, and mutable globals remain unsupported. See the [module rules](modules.en.md).
