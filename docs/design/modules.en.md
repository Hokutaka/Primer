# Modules, namespaces, and visibility

[日本語](modules.ja.md)

## Purpose

Moving types and functions into another file must preserve values, evaluation order, independent copies, and failure reasons, while retaining the definition file's origin. Modules resolve names at compile time; they introduce no shared runtime state or external mutation interface.

```primer
import "values.prim" as values;
item: values::Reading = values::reading(18446744073709551615);
print(item.amount);
```

```primer
// values.prim
pub type Reading { amount: u64, }
pub fn reading(amount: u64) -> Reading {
    return Reading { amount: amount };
}
```

## Names and visibility

- Write `import "relative/path.prim" as alias;` before definitions and statements. An alias is required. `import`, `as`, and `pub` are keywords.
- Access external types and functions with `alias::name`. Local definitions use unqualified names. Definitions in separate files may have the same name.
- Definitions are private to their file by default. Use `pub fn` or `pub type` to expose each definition independently.
- A public type exposes all its fields and defaults; field visibility and opaque types are unsupported. Public parameter/result types and public fields cannot refer to private types, including inside arrays.
- Public function bodies may call private helpers in the same file. Importers cannot call those helpers directly.
- Duplicate aliases and collisions between aliases and definitions, bindings, or parameters are diagnosed. Built-in type names, `infer`, `convert`, and `byte_len` cannot be aliases.
- Multiple aliases may refer to one file. Diamond imports register that physical file once and preserve nominal type identity.

Aliases belong only to their declaring file. Re-exports, wildcard imports, multi-level `a::b::name` paths, and imported variable values are unsupported. Every user of a dependency declares its own import.

## Loading and execution

Relative paths resolve against the importing file's directory, with no fallback to the process working directory or environment search paths. Import paths use forward slashes and end in `.prim`. Parent-directory `..` references are allowed; absolute paths, drive prefixes, backslashes, and control characters are rejected. The entry CLI argument accepts ordinary OS paths.

Canonical physical paths identify duplicate files and cycles. Imports through symbolic links resolve relative to the target directory. Hard links with distinct canonical paths are separate modules. File IDs follow entry-first, depth-first traversal in import declaration order. Cycles are diagnosed and import nesting is limited to 128 levels. Files are read as UTF-8 without Unicode or CR/LF normalization.

**Imported files may contain only imports, types, and functions.** Top-level bindings and executable statements are diagnosed even in unused dependencies. Put initialization in a function and call it explicitly from the entry. Imported functions named `main` are ordinary functions and never run automatically. The entry retains the existing choice of top-level statements or a parameterless `void` main, not both.

Unused definitions are still checked and generated. Modules do not bypass existing type, parameter-count, recursion, or target restrictions. Numeric and string semantics remain shared by every route.

## Compiler pipeline and observation

`modules::load(entry)` parses files and resolves imports and visibility into a common AST with distinct names. Existing semantic analysis, common IR, and lowering remain shared. Backends do not reinterpret modules. Internal names are `module_<file ID>_<definition name>`, except entry function `main`. [File-aware spans](source-files.en.md) retain original spelling and byte ranges.

Existing string-based APIs such as `compile(&str)` perform no file I/O and request the file API for import/pub declarations. CLI inputs without import/pub preserve existing artifacts and diagnostics. Module inputs assign numeric file IDs, including the entry, and failure records include `file=N`.

```sh
primer emit-sources examples/modules/main.prim -o sources.json
```

This explicitly outputs JSON with schema `primer-sources-v1` and a registration-ordered `files` array containing `id`, `name`, and `text`. It exposes exact source contents, not running memory or a mutation interface. Interpret `file`, `node`, and `bytes` with the matching contents and compiler version; IDs are not persistent across edits.

`observe-native.cjs` compiles from the original entry and saves/hashes `sources.json`. It rejects changes to the inventory or contents detected between observation's start and end. It does not lock the filesystem across CLI calls; keep inputs unchanged during observation.

## Verification and remaining scope

The [module examples](../../examples/modules/README.en.md) contain modular and single-file normal programs and a callee-failure example. `tests/source_files.rs` executes CLI-generated C, LLVM, QBE, WAT, assembly, and internal objects against VM values, prior output, reasons, and definition-file spans. C/LLVM run with and without optimization on the supported Windows/Linux targets.

`tests/modules.rs` covers diamond imports, same-named private helpers, nominal identity, private types leaking through public signatures, cycle/syntax/type/path errors, preserving output files after compilation failures, and exact source contents. Existing single-source observation fixtures remain unchanged.

Package distribution, version resolution, re-exports, compile-time constants, separate compilation, dynamic loading, and mutable module globals are outside this increment. Further function-parameter and value-operation improvements should follow concrete executable examples.
