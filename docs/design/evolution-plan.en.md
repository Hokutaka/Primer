# Compiler evolution plan

[日本語](evolution-plan.ja.md)

**Status: Draft**

This document organizes the problems, required capabilities, design candidates, decisions, and implementation order for extending Primer. It is not a decision record that changes the current language specification or compatibility contract. Agreed decisions will be incorporated into the [compiler architecture](architecture.en.md), [observability contract](observability.en.md), and [language reference](../reference/language.en.md).

The [use cases for design decisions](use-case-analysis.en.md) evaluate the possibilities and limits of design candidates against concrete future uses.

## How to read this draft

The first review is intended to follow these sections in order:

1. Confirm Primer's decision criteria in "Purpose."
2. Find missing questions in "Questions observation should answer" and "Primer-specific design questions."
3. Compare the benefits and constraints of alternatives in "How design choices change possibilities."
4. Review immediate decisions in "Decisions for Issue #2."

"Capabilities, limits, and use cases" is reference material for individual observation capabilities. The sections from "Implementation problems" onward translate design decisions into issues and implementation order.

The current primary questions are whether an explanation includes transformations and rationales in addition to provenance, whether Structured Primer IR and Control-flow IR are separate stages, and how backend support is rolled out.

## Purpose

Primer is an experimental programming language for making compiler transformations observable. Keeping the language or implementation small is not an end in itself. Advanced analysis, lowering, and optimization remain valid as long as their boundaries, inputs, results, and reasons can be traced.

Primer is a general-purpose programming language and does not restrict programs to a particular domain. Use cases do not select which domains Primer permits. They test whether a design unnecessarily closes options when presented with different kinds of computation.

As a requirement for generality, Primer will let users give meaningful names and structure to values through user-defined types. Rather than making every domain type a built-in, Primer aims to let users construct required representations from general types, functions, and data structures.

To use broad observability safely, Primer will design `Secret` values that do not cross into observations or ordinary external output. `Secret` is not limited to passwords. It is a safety boundary applicable to values of any type, including user-defined types. The [Secret value design](secrets.en.md) develops this direction.

Primer does not avoid pursuing performance. It aims to explain which transformations contribute to performance so that performance can be pursued using observable evidence.

The ability to observe compilation is distinct from the ability to interfere with it. Observation data is read-only and does not grant authority to modify compiler state.

## Current position

Primer currently has the following foundations:

- typed, backend-independent Primer IR;
- explicit lowering from Primer IR into each backend IR;
- two public observation boundaries: Primer IR and emitted artifacts;
- UTF-8 byte-range `Span` values used by the AST, Primer IR, and diagnostics;
- `Source(span)` and `Synthetic` origins on bytecode instructions;
- VM execution diagnostics that associate bytecode instruction indexes with source locations;
- snapshot tests for deterministic textual output;
- design boundaries for observation and control, source disclosure, and reproducibility.

The current pipeline primarily handles straight-line statements and expressions:

```text
Source
  -> Tokens
  -> AST
  -> typed Primer IR
  -> Backend IR
  -> Artifact
```

Functions, control flow, aggregate types, and optimization introduce relationships that are not one-to-one. Passing only the current `Span` through those transformations will not preserve the explainability required by Primer.

## Questions observation should answer

The future observation model should answer at least the following questions:

| Direction | Question |
| --- | --- |
| Forward from source | What did this source element become at each stage? |
| Backward from an artifact | Which source elements produced this IR element or emitted instruction? |
| Between stages | Which pass or lowering generated, changed, merged, or split this element? |
| Compiler-generated elements | Why was an element with no direct source counterpart required? |
| Removal | Why is an element from an earlier stage absent from a later stage? |
| Diagnostics | At which stage and element did a problem occur, and where does it map to source? |
| Comparison | How do routes, targets, or options differ for the same input? |

These questions do not require a particular file format or CLI command. Primer should first retain enough internal information to answer them, then decide how that information should be exposed.

## Primer-specific design questions

General compiler design primarily focuses on implementing type checking and code generation correctly. Primer must additionally evaluate every design change using the following questions:

- What becomes newly observable because of this change?
- What remains unobservable under this design?
- Which investigations, comparisons, and explanations become possible?
- How much implementation cost and data volume are required to retain the information?
- Can observation affect artifacts, security, or reproducibility?

### Separate the information that forms an explanation

Primer needs to distinguish five kinds of information when explaining a transformation:

| Information | Question answered | Example |
| --- | --- | --- |
| Provenance | Where did it come from? | This instruction came from a source expression and an IR entity |
| Transformation | What happened? | One expression was split into three instructions |
| Rationale | Why was it done? | The target ABI requires argument extension |
| Observation representation | How is it shown? | Snapshots before and after a pass, plus a mapping |
| Measurement | What consequence was observed? | Execution time, code size, or instruction count changed |

These kinds of information cannot substitute for each other.

- A `Span` alone does not describe the transformation or its rationale.
- A pass name alone does not show which entities changed or how.
- Recording a rationale does not prove that a transformation improved performance.
- A measurement alone does not explain which transformation contributed to the difference.

Primer owns provenance, transformations, and rationales. Consumers such as Whitebase measure performance in external toolchains and combine those measurements with Primer observations.

### Treat information-loss points as boundaries

New IR stages and observation points are not chosen only to match implementation modules. A transformation is a candidate for an explicit boundary when source-level meaning cannot be reconstructed from its result.

Examples include:

- lowering a structured `if` into basic blocks and branches;
- splitting an aggregate into address calculations and individual memory operations;
- lowering an abstract call into argument operations for a specific ABI;
- merging or removing expressions, branches, or loads during optimization.

When the result alone cannot explain the transformation, Primer retains representations before and after the transformation or records the transformation itself.

### Decide how much to record about transformations that did not happen

Experiments may need to answer not only why a transformation happened, but also why a candidate transformation was not applied.

Always recording every candidate and rejection reason would substantially increase observation volume and implementation cost. Primer can choose among the following levels:

1. Record only transformations that were applied.
2. Record rejection reasons for explicitly selected passes.
3. In a detailed investigation mode, record a bounded set of considered candidates.

The primary candidate is to begin with level 1 and add level 2 or 3 to individual passes when a concrete experiment requires it.

### Distinguish deterministic presentation from observed facts

Sorting observation results for stable comparison can hide the order in which processing actually occurred. Primer does not confuse the following concepts:

```text
canonical order  deterministic presentation for comparison and storage
execution order  the order in which processing or selection actually occurred
```

When actual order is meaningful, it is retained as observation data. Stabilizing presentation must not rewrite an order that was itself being observed.

### Describe correspondence across multiple backends

The same Primer IR entity can lower into different numbers and kinds of entities in different output routes. Primer should support comparing how routes diverge from common meaning, rather than only inspecting each artifact in isolation.

This comparison requires distinguishing:

- meaning shared by Primer;
- representation selected by an output route;
- transformations required by a target or ABI;
- transformations performed outside Primer by external toolchains.

Primer explains only the boundaries it owns. It does not record an external compiler decision as a Primer decision.

## Capabilities, limits, and use cases

Observation capabilities do not need to arrive all at once. For each capability, Primer makes clear what becomes possible and what remains impossible.

### Source ranges

**Enables**

- map diagnostics to line and column locations;
- map AST, Primer IR, and bytecode instructions back to source ranges;
- highlight corresponding source in an editor.

**Does not enable or is limited by**

- cannot distinguish several entities generated from the same range;
- cannot represent merging, removal, or selection rationale;
- a `Span` alone cannot identify a file in a multi-source compilation.

**Use cases**

- diagnostic rendering;
- highlighting source and a single IR entity;
- reverse lookup for current bytecode execution errors.

### Compilation-local entity identifiers

**Enables**

- distinguish entities sharing the same `Span`;
- reference exact relationships within and across stages;
- enumerate several entities generated from one entity.

**Does not enable or is limited by**

- an identifier alone does not provide a source location, transformation, or rationale;
- direct comparison with old results requires a separately defined cross-compilation stability policy;
- assigning identifiers does not preserve why removed entities disappeared.

**Use cases**

- entity selection and tracing in a UI;
- provenance graphs and mappings;
- references inside structured observation data.

### One-to-many and many-to-one provenance

**Enables**

- trace the splitting of an expression into several instructions;
- trace the merging of values or branches into one result;
- navigate between source, Structured IR, Control-flow IR, and Backend IR.

**Does not enable or is limited by**

- relationships alone do not describe transformation rules or rationales;
- an entity removed without a later counterpart requires a separate record;
- retaining relationships without bounds can produce large observation data.

**Use cases**

- forward tracing from source to emitted instructions;
- reverse lookup from artifacts to source;
- comparing lowering results across backends.

### Immutable snapshots for each stage

**Enables**

- compare representations before and after a transformation;
- observe without exposing mutable compiler state;
- fix transformation results in regression tests;
- provide agents and external tools with inputs that have explicit boundaries.

**Does not enable or is limited by**

- snapshots alone do not explain why a difference occurred;
- individual operations cannot be replayed unless intermediate activity is retained;
- storing every stage for a large program consumes time and space.

**Use cases**

- diffs before and after a pass;
- compiler regression investigation;
- stage-by-stage display in Tint*;
- structured analysis by an agent.

### Transformation records

**Enables**

- explain generation, replacement, merging, splitting, and removal as operations;
- identify the pass that caused a difference between snapshots;
- record removed entities separately from later results.

**Does not enable or is limited by**

- a recorded operation does not automatically prove semantic preservation;
- recording every low-level operation can produce mechanical, unreadable explanations;
- records must remain consistent with pass implementations.

**Use cases**

- optimization explanations;
- finding the cause of unexpected code-generation differences;
- compiler education and transformation visualization.

### Rationales

**Enables**

- explain the rule that selected an instruction, ABI operation, or optimization;
- explain why the same meaning becomes a different representation for each target;
- detect regressions caused by changes to selection conditions.

**Does not enable or is limited by**

- a rationale does not prove an actual performance improvement;
- free-form text is difficult to compare or process mechanically;
- overly detailed rationale categories can accidentally freeze internal implementation as a public contract.

**Use cases**

- instruction selection and ABI-lowering explanations;
- optimization-decision investigation;
- relating Primer observations to Whitebase measurements.

The primary candidate is to separate a stable rationale kind from optional explanatory text.

### Rejection reasons

**Enables**

- investigate why an expected optimization did not occur;
- compare optimization conditions or target features;
- identify possible pass improvements.

**Does not enable or is limited by**

- recording every candidate can rapidly increase observation volume;
- implementations that do not enumerate candidates cannot produce complete rejection reasons;
- the information is not required for every normal compilation.

**Use cases**

- optimization development;
- investigating target-specific performance differences;
- detailed experiments on a selected pass.

### Emission Map

**Enables**

- map ranges of emitted text back to Backend IR and source;
- highlight corresponding locations without changing the artifact;
- compare how the same Primer IR entity appears in each backend.

**Does not enable or is limited by**

- cannot trace transformations performed outside Primer by an external compiler;
- emitters must record output ranges accurately;
- formatting by another tool after emission invalidates range mappings.

**Use cases**

- synchronized source and artifact display in Tint*;
- generated-code investigation;
- backend-difference comparison.

### Backend capability information

**Enables**

- introduce a language feature incrementally through supported output routes;
- diagnose unsupported features before execution;
- let tools select usable routes mechanically.

**Does not enable or is limited by**

- declaring support does not prove semantic equivalence between backends;
- overly granular capabilities make combination management complex;
- the simple contract that every route supports every feature is lost.

**Use cases**

- incremental implementation of language features;
- experimental backend development;
- validation by the CLI, Tint*, and Whitebase before execution.

### Versioned public observation schema

**Enables**

- Tint*, Whitebase, and agents to consume structured observations reliably;
- identify results produced by different Primer versions;
- inspect entities and relationships without parsing human-readable text.

**Does not enable or is limited by**

- public release creates compatibility and migration obligations;
- exposing internal IR directly limits freedom to experiment with implementation;
- a schema cannot explain information that the compiler did not retain internally.

**Use cases**

- long-lived experiment results;
- IDEs and visualization tools;
- read-only agent analysis;
- comparison across Primer versions.

## How design choices change possibilities

| Design choice | What becomes possible | What becomes harder or is lost |
| --- | --- | --- |
| Separate Structured IR and Control-flow IR | Observe the boundary where structure becomes a CFG and share control-flow lowering across backends | Adds another IR stage, validator, and provenance relationship |
| Lower directly from Structured IR into each backend | Keep the pipeline and number of implementation stages smaller | Distributes control-flow conversion across backends and makes common comparison harder |
| Implement every backend together | Keep language semantics aligned across all output routes | Makes each feature large and prevents early validation of meaning and IR alone |
| Introduce support incrementally with explicit capabilities | Support small vertical implementations and experiments | Requires unsupported diagnostics, a capability table, and branching in consumers |
| Embed observation data into artifacts | Inspect correspondence using one file | Changes artifacts and may affect downstream tools or performance |
| Store an Emission Map as side data | Support reverse lookup without changing artifacts | Requires managing the artifact and mapping as a pair |
| Retain snapshots only | Keep implementation and comparison relatively simple | Cannot directly explain rationales or removal processes |
| Retain transformation records as well | Explain generation, changes, and removal | Adds record volume, implementation work, and presentation design |
| Publish the internal model early | Allow external tools to be built sooner | Turns internal design changes into compatibility problems |
| Validate internal use cases before publication | Narrow the schema using real use cases | Delays availability to external tools |

## Intended use cases

### Interactive compiler inspection

A user selects a source expression and sees corresponding entities in Structured IR, Control-flow IR, Backend IR, and the artifact. The user can also navigate backward from an emitted instruction to source and its transformation rationale.

Required capabilities include source ranges, entity identifiers, provenance across stages, and an Emission Map.

### Compiler regression investigation

Snapshots from two Primer versions for the same input are compared to find the first pass and entity where behavior diverged.

Required capabilities include deterministic snapshots, pass identity, and transformation records.

### Optimization experiments

Primer shows applied transformations and their rationales. Whitebase measures execution time and code size. Explanations and measurements remain separate but can be associated with the same experiment.

Required capabilities include transformation records, rationales, explicit compilation conditions, and experiment identity shared with external measurements.

### Backend comparison

A user compares how the same Primer IR entity is represented in C, LLVM IR, QBE IR, WAT, assembly, and bytecode.

Required capabilities include shared semantic entities, provenance from each lowering, Emission Maps, and target conditions.

### Agent analysis

An agent receives a size-limited structured snapshot containing only required stages. The agent can read relationships but cannot use identifiers to mutate compiler state.

Required capabilities include a versioned read-only schema, size limits, and explicit source disclosure.

### Diagnostic explanations

A diagnostic can show not only the error location, but also the stage and rule that detected the problem and its relationship to earlier entities.

Required capabilities include structured diagnostics, stage identity, provenance, and safe rendering.

## What observations explain and what they do not prove

Primer does not overstate what its observations establish.

- Provenance does not prove that a transformation preserved semantics.
- A rationale does not prove that performance improved.
- A deterministic artifact does not prove identical runtime behavior across targets.
- Capability information does not prove that a backend implementation is correct.
- An Emission Map does not trace transformations inside external toolchains.
- Snapshots alone cannot fully explain why an entity was removed.

Correctness requires validation and tests. Performance requires measurement. External transformations require records from the external tools that perform them.

## Decision template for design issues

Future design issues record more than a feature name and implementation proposal. They include:

1. Problem to solve
2. Questions observation should answer
3. Information currently retained
4. Additional information and transformation boundaries required
5. Candidate designs
6. What each candidate enables
7. What each candidate prevents or makes difficult
8. Intended use cases
9. Security and external-interference impact
10. Reproducibility and determinism impact
11. Public compatibility impact
12. Verification method and completion criteria

This structure evaluates not only whether Primer can implement a feature, but also whether the resulting design can explain it.

## Implementation problems

### A `Span` is a location, not an element identity

A `Span` identifies a source location but cannot distinguish multiple elements generated from the same range. It also cannot fully describe one element derived from several ranges or an element with no direct source counterpart.

Source locations and identities for entities within a compilation need separate representations.

### Transformations are not always one-to-one

Future transformations include:

- splitting one expression into several instructions;
- merging values or branches into one value;
- lowering an aggregate into loads, stores, and address calculations;
- merging or removing elements during optimization;
- generating helper instructions for an ABI or target.

A single `Span` on each result cannot explain these relationships.

### Structured meaning and control flow have different roles

The current Primer IR represents resolved Primer meaning in a structure close to the source. The implemented `if` remains a structured statement and is lowered into branches and merge points in each backend IR. Functions, loops, and early returns may eventually justify a shared control-flow stage with explicit basic blocks, branches, and terminators.

Turning Primer IR directly into a CFG would discard source-level structure early. Keeping only structured IR would instead require every backend to reinterpret control flow independently.

### Artifacts are not mapped back to source

Primer can currently map generated bytecode instructions back to source locations. It does not retain mappings from ranges in emitted C, LLVM IR, QBE IR, WAT, or assembly text to internal or source entities.

Embedding comments or identifiers into artifacts would change the artifacts themselves. Mapping information should be retained separately.

### Backend rollout policy is undefined

For each new language feature, Primer must decide whether every output route is implemented at once or whether support can be introduced incrementally with explicit capability reporting.

Incremental rollout requires unsupported features to produce structured diagnostics rather than panics or invalid output.

## Required capabilities

### Before later language extensions

- decide whether to introduce a separate control-flow stage;
- define a conceptual provenance model supporting one-to-many, many-to-one, and compiler-generated relationships;
- define the backend rollout policy for new language features;
- define invariants preserved by each transformation stage.

### Introduced with the first control-flow feature

- blocks and lexical scope;
- compilation-local binding identifiers;
- structured `if` statements in Primer IR;
- branches and deterministic merge points in Bytecode and each backend IR;
- diagnostics for condition types and scope violations.

A shared control-flow IR, cross-transformation provenance, and common rules for merging SSA values after branches have not been introduced. The current QBE lowering places bindings in explicit stack slots so mutable values keep their meaning across branches.

### Before adding public observation features

- immutable snapshots detached from compiler state;
- schema versioning;
- deterministic identifiers and ordering;
- explicit source-text and file-path disclosure settings;
- capture-time redaction that prevents secret values from entering observation snapshots;
- limits on output size, depth, and element count;
- a compatibility policy for unknown fields and new passes.

## Design proposal

### Separate IR stages by responsibility

The primary candidate architecture is:

```text
Primer Source
      ↓
AST
      ↓
Typed Structured Primer IR
  - resolved types and names
  - source-level functions and structure
  - backend independent
      ↓
Control-flow Lowering
      ↓
Control-flow IR / Core IR
  - function bodies as basic blocks
  - basic blocks
  - explicit terminators
  - backend independent
      ↓
Backend Lowering
      ↓
Backend IR
      ↓
Emitter
      ↓
Artifact
```

`Control-flow IR` and `Core IR` are provisional names. Their final name and exact responsibilities will be decided during design.

Each stage has the following role:

| Stage | Primary responsibility | Excludes |
| --- | --- | --- |
| Structured Primer IR | Typed Primer meaning and structure | ABI, registers, target instructions |
| Control-flow IR | Explicit functions, blocks, branches, and terminators | Target-specific ABI and registers |
| Backend IR | Decisions required by an output route and target | Frontend semantic analysis |
| Artifact | Output consumed by an external tool or runtime | Mutable compiler state |

This separation preserves structured meaning while making the transformation into control flow observable. Control-flow IR does not need to require SSA immediately. Value merging can be decided when concrete control-flow syntax is designed.

### Separate provenance from source locations

A future provenance model conceptually distinguishes the following information:

```text
Entity
  - compilation-local identity
  - stage
  - kind
  - origin

Origin
  - Source(source identity, span)
  - Derived(input entities, pass)
  - Synthetic(pass, reason)
```

This is a conceptual model, not a fixed Rust API or public schema.

- `Source` represents an entity directly associated with source.
- `Derived` represents an entity transformed from one or more entities in an earlier stage.
- `Synthetic` represents an entity generated by a pass or lowering for an explicit reason without a direct predecessor.
- Entity identifiers reference relationships within a compilation and do not grant operational authority.

The current bytecode `Source(span)` and `Synthetic` origins can remain as the first implementation in this direction. Primer does not need to generalize every internal representation before a concrete use case exists.

Removal cannot be represented only by adding provenance to surviving entities. When required, Primer can compare snapshots before and after a pass or record the transformation explicitly.

### Retain artifact mappings as side data

An emitter can record ranges of emitted text or instructions and their corresponding internal entities in a separate mapping:

```text
Emission Map
  artifact range -> backend entity
  backend entity -> origin chain
```

The mapping is not embedded into the artifact. It remains detached, read-only data so that enabling observation does not alter generated output.

### Treat observations as immutable snapshots

Primer does not expose mutable compiler state. Information from a selected stage is provided as an immutable snapshot detached from internal state.

A snapshot has the following properties:

- read-only;
- does not feed information back into compilation;
- generated deterministically from the same explicit conditions;
- produces the same artifact whether observation is enabled or disabled;
- does not expose internal ownership or memory addresses.

Retaining an internal snapshot and making it a public compatibility API are separate decisions.

## Security and reproducibility

- Source text and file paths are exposed only when explicitly requested.
- Actual secret values are not exposed even when broad observation is requested.
- Diagnostics that need only a location do not implicitly include source text or paths.
- Observation identifiers do not authorize file operations, pass execution, or compiler-state mutation.
- External plugins and mutation hooks are designed as capabilities separate from observation APIs.
- Observations do not include timestamps, memory addresses, random values, or incidental `HashMap` ordering.
- Observation output has limits on size and element count for large or adversarial inputs.
- Diagnostic and observation text handles control characters safely.

## Agreed syntax direction

This section distinguishes syntax implemented as part of the current specification from design direction for future implementation. An unimplemented item is not yet a language-reference feature.

### Declarations, mutability, and references

- A new binding uses `name: type_spec = expression;`; `let` and `var` are not added as declaration markers.
- `type_spec` is always present and contains either a concrete type or `infer`.
- Bindings are immutable by default. Only a reassignable binding uses `mut name: type_spec = expression;`.
- `mut` is not a type; it grants reassignment to the name.
- A future `ref T` is a read-only reference. Mutation through a reference is not part of the initial reference design.

`mut` and reassignment are implemented in the current specification. `ref` has only a design direction and is not implemented.

### Functions and entrypoint

- A named function is introduced explicitly with `fn`.
- Parameter and return types use explicit concrete types. A function without a result specifies `void`.
- A value-returning function uses an explicit `return expression;`; a trailing block expression is not an implicit return.
- A `void` function may finish at the end of its block and may exit early with `return;`.
- A program without `main` receives a compiler-generated entrypoint for its top-level executable statements.
- When an explicit `fn main` exists, it cannot be combined with top-level executable statements.
- Command-line arguments enter through a read-only runtime-provided `Args`, not raw `argc` and `argv`.
- A target that does not support requested argument functionality diagnoses it instead of silently supplying an empty value.

Functions, `void`, explicit `main`, and `Args` are not implemented.

### Structured control

- `if condition { ... } else { ... }` is implemented as a statement. The `else` block is optional.
- `while condition { ... }` is implemented as a statement. Its condition is evaluated before its body on every iteration.
- `for (start; condition; update) { ... }` is implemented as a statement. The start statement may be a binding or an assignment. The start statement, continuation condition, body, and update statement remain distinct in Primer IR.
- `break;` and `continue;` are implemented and target only the innermost loop.
- Blocks create lexical scopes. Inner bindings are not visible outside, while an inner block may reassign an outer `mut` binding.
- The source language never introduces `goto`, arbitrary-position labels, or computed goto.
- `foreach` and `switch` / `case` / `default` are future basic control constructs.
- A name introduced by `foreach` also requires `: type_spec`.
- Cases do not fall through implicitly, and no case-ending `break;` is required.
- Backend branches and jumps are permitted, but retain their relationship to the structured control that produced them.

`foreach` and `switch` are not implemented. Whether a value-producing form of `if` should be added later remains open.

## Decisions for Issue #2

[Issue #2](https://github.com/Hokutaka/Primer/issues/2) should resolve the following questions in order before implementation begins.

### 1. IR stages

- Does Structured Primer IR remain the primary semantic representation?
- Is Control-flow IR introduced as a separate stage?
- What invariants does each stage preserve?
- Which stages become public observation surfaces?

The recommendation is to preserve Structured Primer IR and introduce Control-flow IR separately. Additional public observation surfaces should wait until the internal representation and its use cases are stable.

### 2. Functions and entrypoint

- Are current top-level statements retained?
- Are they lowered into an implicit entrypoint?
- Is an explicit `main` required?
- How are parameters, return types, and local scopes represented?
- When are recursion and forward references allowed?

### 3. Control flow

- Is `if` a statement or a value-producing expression?
- Which type is accepted as a condition?
- How are early returns and unreachable blocks handled?
- Are block parameters, phi values, or another representation used for value merging?
- How do loops, `break`, and `continue` relate?

### 4. User-defined types and aggregates

User-defined types are a requirement for making Primer a general-purpose language. Aggregates are considered as their first concrete implementation.

Agreed decisions and remaining questions for named product types are maintained separately in the [named product type design](product-types.en.md).

The design decides:

- whether named types are nominal or identified only by structure;
- how type aliases differ from new types;
- whether the first form is a named product type, tuple, array, or sum type;
- how construction and field access are represented;
- to what extent aggregates are values;
- how far immutability, updates, copy, move, and borrow semantics are introduced;
- when generic types and type arguments are introduced;
- at which stage memory layout is decided;
- where the ABI boundary lies.

The primary candidate uses nominal identity so that named types with different meanings remain distinct, with a named immutable product type as the first implementation. Tuples, arrays, sum types, generic types, recursive types, references, and custom layouts can be added on the same type-system foundation after their requirements and dependencies are understood.

Primer IR retains meaning such as type names, fields, and type arguments, while size, layout, alignment, and calling conventions are decided during or after backend lowering. Provenance and transformation records trace how a type is decomposed into fields, memory, registers, and instructions, including the point where its abstraction is lost.

The ability to define a type is distinct from allowing external code to modify the type checker or compiler state directly. A user-defined type is Primer program input, not an extension authority that crosses the observation and control boundary.

### 5. Backend rollout policy

Primer must choose between:

| Policy | Benefit | Cost |
| --- | --- | --- |
| Implement every route together | Language semantics and all routes remain aligned | Each feature becomes a large change |
| Introduce support incrementally with explicit status | Meaning and IR can be validated in smaller steps | Requires capability reporting and unsupported-feature diagnostics |

Incremental rollout must never allow unsupported routes to panic, emit invalid artifacts, or silently fall back. Support status must be machine-readable and deterministic.

## Proposed issue breakdown

Issue #2 remains the parent design issue. After agreement, it can be divided into the following issues, with names and scope adjusted during discussion:

1. Define provenance requirements and terminology
2. Decide the Structured IR and Control-flow IR boundary
3. Define backend feature rollout and capability reporting
4. Add block and lexical-scope representation
5. Define functions and the program entrypoint
6. Add boolean and comparison semantics
7. Lower conditional control flow
8. Add return and function calls
9. Add loops and loop exits
10. Define user-defined type identity and semantics
11. Add the first nominal immutable product type
12. Lower user-defined aggregate layout in each backend
13. Record artifact ranges in an emission map
14. Define a versioned public observation schema when required

The approximate dependencies are:

```text
Provenance requirements
      ↓
IR boundary
      ├── blocks and scopes
      │     ├── functions
      │     └── control flow
      ├── user-defined type semantics
      │     └── first aggregate representation
      └── emission map

backend rollout policy
      ↓
each vertical feature slice

functions + user-defined type semantics
      ↓
backend ABI and layout

internal observation use cases
      ↓
versioned public observation schema
```

### Secret design issues outside Issue #2

`Secret` depends on types, control flow, and observation snapshots, but its security purpose is distinct from the language-structure scope of Issue #2. It proceeds through separate design issues:

1. Define Secret value semantics and explicit data-flow propagation
2. Define explicit declassification and observation redaction
3. Define secret-dependent control flow and side-channel boundaries

The approximate dependencies are:

```text
user-defined type semantics
control-flow semantics
immutable observation snapshots
          ↓
Secret value semantics
          ↓
redaction and explicit declassification
          ↓
secret-dependent control and stronger guarantees
```

## Completion criteria for a language feature

Each language feature is complete when the applicable criteria are met:

- syntax and semantics are documented in the language reference;
- the AST and Primer IR retain appropriate source ranges or provenance;
- new transformation boundaries and invariants are documented;
- diagnostics are structured and carry appropriate source locations;
- backends that declare support emit deterministic artifacts;
- unsupported output routes return explicit diagnostics;
- normal behavior, errors, and transformation results have snapshot tests;
- `cargo check`, all tests, clippy, and format checks pass;
- Japanese and English documentation remain synchronized.

## Not fixed yet

The following remain future candidates and are not fixed before a concrete need exists:

- the final name of Control-flow IR;
- SSA and the specific value-merging representation;
- the public observation file format;
- a generic `observe` command;
- the Observation Bundle schema;
- new targets such as RISC-V;
- the concrete set of optimization passes;
- external plugins or mutation hooks;
- the module system and multi-source format;
- concrete syntax and introduction order for generic types, sum types, recursive types, references, and custom layouts;
- forcing all tests into one testing technique.

These ideas are deferred rather than rejected. Primer should define their boundaries and use cases before committing to a design.

## Next steps

1. Review this draft and add missing observation questions.
2. Decide whether to separate Structured Primer IR and Control-flow IR.
3. Decide the backend rollout policy.
4. Update Issue #2 with decisions and open questions.
5. Split the first three design issues from Issue #2.
6. Begin small vertical implementation slices only where the design is settled.

Before implementation accelerates, Primer should align on one criterion: what must be observable for a transformation to be explainable?
