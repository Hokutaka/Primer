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
| QBE | A 64-bit reference to read-only storage containing an eight-byte length and UTF-8 data |
| WAT | A 32-bit reference to length-prefixed data in private linear memory |
| Windows x64 direct assembly | A read-only data reference held in registers and stack slots |

Textual IR and bytecode escape line breaks and control characters. `print` writes the contents unchanged. Presentation does not alter the value.

Primer IR resolves shared semantics; each lowerer selects storage, copies, instructions, and calling conventions. Emitters turn backend IR into artifacts without reinterpreting Primer types or semantics. The priority is making each transformation and its reasons observable, rather than requiring a single representation. The observation boundaries remain `emit-ir` and generated artifacts.

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

## QBE, WAT, and direct assembly storage and passing

These routes statically place an eight-byte length followed by the original UTF-8 bytes for each literal, and retain a reference to its beginning as the string value. Function parameters and results, product fields, and array elements use that reference. Neither length nor contents change at runtime; no terminating NUL is added. Primer does not silently optimize by merging string literals.

Reassignment replaces the reference in a binding or array element. Shared storage is immutable, so previous copies remain unchanged. Product and array copies retain existing `blit`, memory load/store, and stack-slot copying. No dynamic allocation or reference counting is needed. These internal references do not add a Primer pointer type or an external ABI.

QBE puts data in read-only `.rodata`, using `loadl` for length and `loadub` for bytes. Strings require `--target x86_64-unknown-linux-gnu`; the artifact records the target and its `qbe -t amd64_sysv` mapping in a comment. This is the combination currently supported by Primer for QBE strings. Selecting another downstream QBE target does not translate Primer's runtime assumptions.

Direct assembly uses the existing Windows x64 target. Length and bytes reside in read-only storage, and references travel through `RAX` or eight-byte stack slots. Comparisons save the left operand before evaluating the right and passing both to a helper. The output helper follows Windows x64 shadow-space, stack-alignment, and register-preservation rules. `_setmode` runs before the first Primer operation and exits with code 1 on failure.

WAT puts data in private linear memory and uses 32-bit addresses. Its eight-byte length header is little-endian; current wasm32 operations read the low 32 bits. Lowering selects memory regions and page counts without allocating string data at runtime. Equality becomes `i32.load8_u` and branches.

### WAT output and the external boundary

WAT programs using strings import `primer.write_byte(i32) -> void`. Generated code reads each content byte, passes its value in 0–255, and finally passes LF (10). The host must preserve these bytes in order without character encoding or line-ending translation. Numbers and Booleans retain the existing `print_i64`, `print_f32`, `print_f64`, and `print_bool` contracts.

Memory is neither exported nor imported for output. Passing byte values instead of string storage references adds no output interface for modifying contents. This is a runtime output contract, not a compiler observation API or a way for observations to control compilation.

## Validation scope

Alongside C snapshots, generated C is compiled with and without optimization and compared byte-for-byte with VM output. Coverage includes Japanese text, empty strings, NUL, line breaks, non-normalizing equality, copies, returned values, nested arrays, evaluation order, and shadowed names. AddressSanitizer and UndefinedBehaviorSanitizer provide additional memory checks.

The LLVM snapshot explicitly selects Linux. On supported Windows/Linux hosts, `cargo test --test llvm_strings` compares generated LLVM, generated C, VM output, and known expectations at `-O0` and `-O2`. It checks raw CR/LF bytes, returns, products, nested arrays, copies, evaluation order, short-circuiting, and out-of-bounds access. Setting `PRIMER_TEST_LLVM_CLANG` and `PRIMER_TEST_CC` makes missing compilers a failure; CI requires comparisons on both operating systems. If unset and a default compiler is missing, execution comparisons print a reason and skip.

The `string-values` observation fixture fixes all eight artifacts. Shared inputs and known expectations in `tests/support/string_cases.rs` are checked against VM, C, LLVM, and, through `cargo test --test string_routes`, QBE, WAT, and direct assembly. Checks distinguish output bytes, evaluation order, short-circuiting, independent copies, and stopping on out-of-bounds access. Even when strings occur only in unused defaults, Windows C, LLVM, and assembly select the same output mode.

QBE runs on Linux x86-64 and assembly on Windows x64. WAT is validated and converted with WABT, then run in Node's WebAssembly engine; tests also require `main` to be its only export. The test host's floating-point output is limited to the shared fixture's exact value `1.5`, not a general numeric formatter. Unavailable execution routes print a reason and are distinguished from completed comparisons. Together, both CI jobs require execution of every route.

Development tools can be selected through `PRIMER_TEST_QBE`, `PRIMER_TEST_ASM_CLANG`, `PRIMER_TEST_NODE`, and `PRIMER_TEST_WAT2WASM_JS` (WABT's `bin/wat2wasm`). An unavailable configured tool fails the test. Install WABT with `npm install --prefix target/wasm-tools --no-audit --no-fund wabt@1.0.39`. Launching these tools belongs to development tests; Primer's emission commands do not launch them.
