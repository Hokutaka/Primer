# Secret value design

[日本語](secrets.ja.md)

**Status: Draft**

This document organizes the purpose, guarantees, limits, and observation behavior of `Secret` values in Primer. `Secret<T>` is provisional notation for explaining the concept, not a final syntax or Rust implementation.

Primer aims to make many transformations observable. Broader observation must not accidentally display or persist passwords, tokens, personal data, model weights, or other secrets.

`Secret` is not a global switch that disables observation. It is a per-value safety boundary that prevents content from crossing into observations and ordinary external output.

The [observability contract](observability.en.md) defines general observation policy. The [compiler evolution plan](evolution-plan.en.md) defines the design process. [Use cases for design decisions](use-case-analysis.en.md) examines concrete future uses.

## Basic model

A normal value can be included in an observation explicitly requested by the user:

```text
value: i64 = 42
observation: 42
```

A secret value does not reveal its content even when broad observation is requested:

```text
value: Secret<i64> = ...
observation: <secret>
```

Primer aims to preserve the following rules for `Secret`:

1. Secret values are not included in observations, diagnostics, or debug output.
2. Values computed from secrets remain secret by default.
3. Secrets do not implicitly cross into normal output, storage, or external transmission.
4. Releasing a secret is explicit and auditable.
5. Lowering and optimization do not erase the secrecy marker.

## Values represented by `Secret`

`Secret` is not limited to passwords. Any value can be treated as secret:

```text
Secret<String>
Secret<Bytes>
Secret<Token>
Secret<Model>
Secret<Tensor>
Secret<UserDefinedType>
```

Domain-specific types such as `Password` can be user-defined or library types composed with `Secret`:

```text
Password
  value: Secret<Bytes>
```

The responsibilities are separate:

| Concept | Responsibility |
| --- | --- |
| `Password` | Password creation, checking, hashing, and other domain operations |
| `Secret<T>` | Prevent the content of `T` from crossing into observation or ordinary external output |

The ability to define a user type is separate from the ability to remove secrecy freely. Ordinary field access or type conversion cannot unwrap `Secret`.

## Visible and hidden information

The initial design candidate keeps the existence and safe transformation of a secret observable while hiding its content.

| Information | Default | Reason |
| --- | --- | --- |
| Actual value | Hidden | It is the secret itself |
| Text representation of the value | Hidden | It would disclose the value |
| Length, shape, and range | Hidden by default or decided separately | They can reveal properties of the value |
| The fact that a value is `Secret` | Visible | The safety boundary must be auditable |
| Inner type `T` | Visible by default | Type checking and transformation need explanation |
| Provenance and transformation path | Visible without value content | Secret flow must be auditable |
| The fact that a release occurred | Visible | Secret-boundary crossings must be auditable |
| Actual released value | Not included in observation records | Auditing release is separate from recording content |

Some uses also treat types, lengths, shapes, access locations, or timing as secret. Those uses need policies stronger than a value-hiding `Secret`. The initial design does not claim to hide existence and all metadata.

## Secret propagation

A result derived from a secret is secret by default:

```text
secret: Secret<i64>

secret + 1              -> Secret<i64>
secret == candidate     -> Secret<Bool>
hash(secret)             -> Secret<Hash>
make_record(secret)      -> Secret<Record> or Record with a secret field
```

This rule prevents derived values from becoming an observation bypass.

Propagation has two forms.

### Explicit data flow

A secret is used directly as an operand, argument, field, or return value. Types and IR can track this flow relatively directly.

The first implementation must track at least explicit data flow.

### Implicit control flow

A secret can leak through control flow without being printed directly:

```text
if secret_flag {
    print(1);
} else {
    print(0);
}
```

The printed `1` or `0` reveals `secret_flag`. Value redaction alone cannot prevent this leak.

Control-flow policy has several candidates:

| Policy | Benefit | Constraint |
| --- | --- | --- |
| Disallow `Secret<Bool>` as an ordinary branch condition | Simple and easier to keep safe | Makes secret-dependent computation difficult to express |
| Treat values and effects inside a secret condition as secret | Expresses more computations | Requires more complex type and effect tracking |
| Provide a separate constant-time secret selection operation | Can express selection with reduced timing leakage | Requires guarantees and validation in every backend |

The primary candidate disallows `Secret<Bool>` as an ordinary public branch condition until a safe control model is designed. Secret-dependent control is designed together with control flow, effects, and backend guarantees.

## Explicit release

Converting a secret into a normal value is explicit declassification:

```text
secret_result: Secret<Bool>
public_result: Bool = declassify(secret_result)
```

`declassify` is not an ordinary cast. It crosses the secret safety boundary.

A release operation has at least the following properties:

- explicit in source;
- identifies the value being released;
- exposes its location through diagnostics and observation;
- can later associate a reason or purpose;
- cannot be invoked by an external observer without program authority;
- cannot be added or removed by an implicit optimization.

An observation record can include the release kind, location, and provenance of the target. It does not automatically persist the actual released value.

Whether release authority belongs to the program author, executor, or a policy remains undecided. Syntax and authority are designed together.

There are two ways to cross a secret boundary: `declassify`, which turns the original content into a normal value, and a trusted transformation that creates a different representation that may be public. They are not treated as the same release operation. The requirements for the latter are described under "Boundary with encryption."

## Behavior at the observation boundary

Secrets must be removed before observation snapshots are constructed, not only hidden during string rendering:

```text
compiler or VM state
      ↓
observation capture boundary
      ↓
redacted immutable snapshot
      ↓
renderer / file / external tool
```

Replacing content with `<secret>` only in a renderer leaves the secret inside observation data passed to that renderer. It can leak through storage, transmission, crash dumps, or another renderer.

Every output surface preserves the secret boundary:

- diagnostics;
- debug output;
- snapshots;
- transformation records;
- Emission Map side data;
- Primer VM traces;
- CLI output;
- future public observation schemas;
- data passed to Tint*, Whitebase, or agents.

Even when provenance or transformations are visible, they do not include content, length, or strings from which content can be inferred. Redaction uses a fixed representation.

## Ordinary output and external transmission

`Secret` applies to normal program output as well as observation.

The initial design candidate does not implicitly permit:

- `print(secret)`;
- serialization of a secret into an ordinary file;
- network transmission of a secret;
- returning a secret through a non-secret external boundary;
- storing a secret in a non-secret aggregate;
- interpolating a secret into formatted text.

Required operations use an API explicitly permitted to handle secrets or an explicit release.

An encrypted destination and `Secret` type rules are separate concepts. An encryption API that accepts a secret is itself defined as a secret-aware boundary.

## Boundary with encryption

`Secret` and encryption solve different problems:

- `Secret` lets the compiler track whether a value may be shown and prevents leakage into observation or ordinary output.
- Encryption transforms a value into another representation using a key. Cryptographic algorithms, key management, and protocols belong to libraries and runtimes.

A conceptual encryption API can be written as:

```text
plaintext: Secret<Bytes>
key: Secret<Key>
nonce: Bytes
ciphertext: Bytes = trusted_encrypt(plaintext, key, nonce)
```

The plaintext and key remain `Secret`. The ciphertext can be treated as a normal value only when `trusted_encrypt` is an approved secret-aware API whose contract explicitly classifies its output as public.

This differs from `declassify`:

- `declassify(secret)` releases the original content itself.
- `trusted_encrypt(secret, ...)` does not release the original content; it creates a different representation under an explicit contract.

An ordinary user function must not accept a secret and arbitrarily return a normal value. A trusted transformation requires at least:

- a signature or contract that declares secret inputs and the secrecy classification of its output;
- compiler or policy authority to use that contract;
- an audit record of the call location, transformation kind, and input provenance without actual values;
- rejection when the backend or runtime cannot implement the required boundary;
- no way for an ordinary function definition or cast to remove the secrecy marker.

Hashing is not automatically a safe public transformation. A deterministic hash or an input with a searchable set of candidates may still reveal the original secret. Redaction, aggregation, and anonymization likewise require their own contracts and policies before their outputs can be public.

Algorithm selection, authenticated encryption, nonce generation, key storage and rotation, and transport protocols are responsibilities above the Primer compiler core. Primer does not implement its own cryptography. It explains where an approved API was used and where its secret inputs came from.

FFI and external cryptographic libraries must also be declared trusted secret-aware boundaries. Passing a secret into an unsupported boundary or one whose contract cannot be verified is rejected.

Marking a value `Secret` does not automatically encrypt plaintext or keys in runtime memory. Secret memory, zeroization, hardware support, and similar properties are separate guarantees when needed.

## Compilation and lowering

The secrecy marker does not disappear after type checking:

```text
Source type
  -> Primer IR
  -> Control-flow IR
  -> Backend IR
  -> runtime value or storage
```

Each stage verifies:

- secret provenance remains available;
- no value was copied into non-secret observation data;
- secret values do not reach public output;
- explicit releases remain explicit;
- optimization preserves the safety boundary;
- the backend supports required guarantees.

Even when an internal backend representation erases the type, side data retains information required for redaction and safety validation.

An unsupported backend must not silently lower `Secret` as a normal value. It returns an explicit unsupported-feature diagnostic.

## Relationship to user-defined types

User-defined types can describe values with secret domain meaning:

```text
Password
ApiToken
PrivateKey
PersonalRecord
ModelWeights
```

An ordinary user-defined type alone cannot guarantee that Primer observation will not inspect its content. `Secret` must be a compiler-understood safety boundary, whether represented as a type, qualifier, effect, or another mechanism.

The final representation remains undecided:

```text
Secret<Password>
secret Password
Password marked as secret
```

Under any syntax, ordinary type definitions cannot forge or remove the secrecy marker.

## What `Secret` does not guarantee

`Secret` is not encryption and does not defend against every attack.

The initial `Secret` alone cannot guarantee:

- encryption of values in memory;
- guaranteed zeroization after use;
- prevention of leakage through timing, caches, branches, or access patterns;
- secrecy from the OS, a debugger, another process, or an administrator;
- secrecy from a malicious backend or external toolchain;
- protection of a secret literal written directly in source from source readers;
- protection of a secret constant embedded in a binary from analysis;
- safe handling after a value enters an external library.

Constant-time processing, encryption, zeroization, secret management, and sandboxing are separate guarantees designed when required.

Wrapping a password or key literal written directly in source with `Secret` does not make it safe. Secrets need to enter through a secure runtime input boundary.

## Security levels

Different mechanisms are required depending on what must be hidden:

| Level | Hidden subject | Required mechanism |
| --- | --- | --- |
| Value secrecy | Values and directly derived content | `Secret`, redaction, release control |
| Metadata secrecy | Types, lengths, shapes, names, provenance | Metadata policy and stronger redaction |
| Behavioral secrecy | Branches, timing, access patterns, communication volume | Information-flow control, constant-time or oblivious processing |

The first `Secret` focuses on value secrecy. Uses that need metadata or behavioral secrecy explicitly require additional guarantees.

## Intended uses

- keep passwords, tokens, and private keys out of diagnostics and traces;
- exclude ML model weights, inputs, and gradients from observation;
- handle fields containing personal information safely;
- enable broad compiler observation while redacting only secret values;
- audit where secrets flow and where they are released;
- remove secrets before observation data is passed to an agent.

## Decisions required before implementation

1. Is `Secret<T>` represented as an ordinary generic type, a type qualifier, or an effect?
2. How much of the inner type, length, shape, and name remains visible?
3. At which IR stage is explicit data flow tracked?
4. How does control flow using `Secret<Bool>` behave?
5. What syntax, authority, and rationale represent release?
6. How are direct release and trusted transformations such as encryption distinguished?
7. Who approves trusted transformations, and how do they declare output secrecy?
8. How does secrecy propagate through parameters, returns, fields, and containers?
9. Under which conditions can secrets enter external functions or FFI?
10. What qualifies a standard API as secret-aware?
11. What minimum guarantees must a backend provide?
12. How much runtime-memory zeroization is guaranteed?
13. How are panics, crashes, and core dumps handled?
14. What fixed redaction appears in observation snapshots?

## Completion criteria

The first `Secret` implementation is complete only when at least the following hold:

- the type system distinguishes secret and normal values;
- results derived directly from secrets remain secret;
- diagnostics, debug output, snapshots, and traces contain no actual secret values;
- ordinary `print`, serialization, and external output reject secrets;
- only explicit release or an approved transformation crosses the secret boundary;
- ordinary functions and casts cannot remove the secrecy marker;
- trusted transformations explicitly declare the secrecy classification of their outputs;
- release can be audited without recording the value;
- unsupported backends return explicit diagnostics;
- optimization preserves the secret boundary;
- tests cover accepted operations, rejected operations, redaction, and unsupported backends;
- unsupported side channels and external boundaries are documented.

`Secret` is not an exception that weakens Primer observability. It makes the safely observable boundary explicit in the type system so that transformations can be explained without leaking secret content.
