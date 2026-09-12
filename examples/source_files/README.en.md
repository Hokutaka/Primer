# Observing file-local origins

[日本語](README.md)

This first step toward modules checks whether separated files still identify the correct expression. There is no import syntax yet. The Rust example parses each Primer file separately and builds common IR.

```sh
cargo run --example source_files
cargo run --example source_files -- failure
```

The second command intentionally exits with code 1 after division by zero. Check its reason, prior output, and definition-file location; it is not a successful program run. Passing these `.prim` files individually to `primer run` does not load definitions from other files.

| File | Types and expressions | What it checks |
| --- | --- | --- |
| [values.prim](values.prim) | `u64`, `string`, product, functions | Definition origins, default string values, division inside a function |
| [main.prim](main.prim) | `[Reading; 1]`, copies, short circuiting | Maximum u64, independent copies after reassignment, unevaluated right operand |
| [failure.prim](failure.prim) | Successful call followed by division by zero | Identifies `value / divisor` in `values.prim`, rather than the caller |

Normal stdout, with byte escapes:

```text
18446744073709551615\n2\n観測\0\r\n\nfalse\n計算\n5\n
```

Failure preserves `開始\n計算\n5\n計算\n`. Its structured record contains `code=division-by-zero` and `file=1`; human-readable output resolves to `values.prim:13:12`. NodeIds and byte ranges correspond to source text and may change with edits or line endings.

`cargo test --test source_files -- --nocapture` compares available execution routes for both separate-file ASTs and a single source. It also checks lexical, syntax, and semantic errors, and array bounds failures that must prevent right-hand evaluation. See the [design](../../docs/design/source-files.en.md).
