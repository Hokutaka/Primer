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

## Validation scope

Alongside C snapshots, generated C is compiled with and without optimization and compared byte-for-byte with VM output. Coverage includes Japanese text, empty strings, NUL, line breaks, non-normalizing equality, copies, returned values, nested arrays, evaluation order, and shadowed names. AddressSanitizer and UndefinedBehaviorSanitizer provide additional memory checks.

String output through LLVM, QBE, WAT, and direct assembly remains unsupported and is diagnosed with a source location before lowering.
