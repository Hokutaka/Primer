# String design

[日本語](strings.ja.md)

## Value semantics

`string` is an immutable UTF-8 value. Reassignment to a `mut` binding replaces the stored value; it does not modify that string's contents. Previously copied values and values passed to functions remain unchanged.

Comparison uses the entire contents without Unicode normalization or case conversion. Embedded NUL is part of the value. Syntax and operators are specified in the [language reference](../reference/language.en.md#strings).

## Observation and representation

| Stage | Current representation |
| --- | --- |
| Primer IR | The `string` type and decoded contents; Span identifies the original quoted spelling |
| Bytecode | `push.string`, typed storage, comparison, and output instructions; source-derived instructions retain NodeId and Span |
| VM | Owned Rust `String`; copying a value clones its contents |
| C | A struct containing a read-only data pointer and a UTF-8 byte count |
| LLVM | `%primer.string = type { ptr, i64 }` and static module byte arrays |

Textual IR and bytecode escape line breaks and control characters. `print` writes the contents unchanged. Presentation does not alter the value.

## C storage lifetime

```c
typedef struct primer_string {
    const unsigned char *data;
    size_t length;
} primer_string;
```

All current strings originate from source literals. Their data is emitted as C string literals with static storage duration, retained until process exit. Returning strings or arrays containing strings from functions does not invalidate the data.

Assignment copies the pointer and byte count. Sharing the data preserves immutable value semantics because Primer exposes no operation to change those contents. Equality compares contents, not sharing or addresses. No string-specific `malloc`, `free`, or reference counting is needed.

This representation applies to the current feature set, which does not create new string contents at runtime. It does not cover memory management for concatenation or external input. The generated C struct is not a stable external-integration ABI.

## Bytes and evaluation order

Each literal byte is emitted as a fixed three-digit octal escape. Japanese text does not depend on the C compiler's execution encoding, and quotes or other contents cannot become C syntax.

Equality checks byte counts before using `memcmp`. Printing uses `fwrite` and appends LF, never `strlen` or `strcmp`. On Windows, programs using strings switch standard output to binary mode to preserve CR, LF, and NUL. Existing numeric-only programs retain text mode.

Where C does not guarantee evaluation order and multiple operands can produce effects or fail, expressions are saved to function-local temporaries in source order. Evaluation stays inside short-circuit operands and loop conditions or updates. Binding names also include Primer IR IDs to avoid shadowing and runtime-helper collisions.

## LLVM representation and targets

LLVM string data is constant and lasts until process exit. Each literal is stored in lowering order as `private unnamed_addr constant [N x i8]`, using fixed two-digit hexadecimal escapes for every UTF-8 byte. No terminator is added; the original byte count is retained as `i64`. Empty strings never dereference their data.

Strings, products containing strings, and nested arrays are copied as LLVM aggregate values, including function parameters and results. Contents are never written and require no dynamic allocation. Equality checks length and every byte regardless of sharing or constant merging. This internal representation does not guarantee external C ABI compatibility.

Printing calls `putchar` for each byte and appends LF. Bytes are zero-extended to `i32`, preserving the high bits of UTF-8. This uses the same standard output as numeric `printf` and Boolean `puts`, preserving mixed output order. Comparison and output loops remain readable in generated LLVM. Optimizing large string output is future work.

LLVM programs using strings require an explicit `--target`. Two targets are supported:

| Target | Standard output initialization |
| --- | --- |
| `x86_64-unknown-linux-gnu` | No mode change |
| `x86_64-pc-windows-msvc` | Call `_setmode(1, 32768)` on CRT standard output (descriptor 1) to select binary mode |

Windows initialization runs before any Primer operation, including a call to an explicit `main`. Failure exits with code 1. Programs using strings also terminate numeric and Boolean output with LF. Existing programs without strings retain unspecified-target generation and their previous output mode.

The selection is recorded in the artifact's `target triple`; the compiler never selects it from its host OS or environment variables. Strings in unused definitions also require a target, with a source-located diagnostic when omitted. Pass the same target to downstream Clang. Overriding it with a different target does not translate already generated OS-specific operations.

LLVM lowering emits expressions in source order and retains the required branches inside short-circuit expressions and loops. Target selection is generation data, not authorization to launch external programs or interfere with the compiler.

References: [LLVM constants](https://www.llvm.org/docs/LangRef.html#constants), [target triple](https://www.llvm.org/docs/LangRef.html#target-triple), and [Microsoft CRT _setmode](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/setmode?view=msvc-170).

## Validation scope

Alongside C snapshots, generated C is compiled with and without optimization and compared byte-for-byte with VM output. Coverage includes Japanese text, empty strings, NUL, line breaks, non-normalizing equality, copies, returned values, nested arrays, evaluation order, and shadowed names. AddressSanitizer and UndefinedBehaviorSanitizer provide additional memory checks.

The LLVM snapshot explicitly selects Linux. On supported Windows/Linux hosts, `cargo test --test llvm_strings` compares generated LLVM, generated C, VM output, and known expectations at `-O0` and `-O2`. It checks raw CR/LF bytes, returns, products, nested arrays, copies, evaluation order, short-circuiting, and out-of-bounds access. Setting `PRIMER_TEST_LLVM_CLANG` and `PRIMER_TEST_CC` makes missing compilers a failure; CI requires comparisons on both operating systems. If unset and a default compiler is missing, execution comparisons print a reason and skip.

String output through QBE, WAT, and direct assembly remains unsupported and is diagnosed with a source location before lowering.
