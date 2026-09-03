# Use cases for design decisions

[日本語](use-case-analysis.ja.md)

**Status: Draft**

This document evaluates Primer design candidates against possible future use cases. It does not commit Primer to a product plan or supported scope.

Primer does not restrict itself to one use. Concrete scenarios are design stress tests. They show what becomes observable under a design, what remains unobservable, which uses become possible, and which costs or risks follow.

The [compiler evolution plan](evolution-plan.en.md) defines the broader direction. The [observability contract](observability.en.md) defines current observation boundaries.

## General-purpose language premise

Primer is not a language for a preselected set of uses. Numerical computing, ML, systems work, visualization, tools, and business applications are examples of computations a general-purpose language may express, not a list of permitted domains.

The design therefore follows these principles:

- Core language features do not depend on the names or implementations of a particular domain.
- User-defined types, functions, and operations follow the same provenance principles as built-in entities.
- The observation foundation is not rebuilt for every new use.
- Domain-specific advanced optimizations remain explicit transformation boundaries.
- Scenarios reveal design capabilities and limits rather than exclude uses.

"Can do anything" has at least the following levels:

| Level | Meaning |
| --- | --- |
| Expressibility | Types, functions, control flow, and data structures can be composed to describe the computation |
| Practical usability | Required libraries, runtimes, I/O, performance, and development support are available |
| Output-route support | A selected backend and target can produce an executable artifact |

Primer does not unnecessarily restrict expressibility. This does not imply that the current implementation already provides every library, runtime, and backend. What the language can express and what is practical today remain distinct.

### Separate computation from representation

A computation does not necessarily require syntax or built-in types dedicated to its domain. Matrix multiplication and convolution, for example, can be described using general numbers, arrays, functions, loops, and memory operations. Tensor types, library operations, intrinsics, and specialized IR are alternative representations that attach different structure and semantic information to the same computation.

```text
user-defined abstraction
      ↓
generic functions, loops, and data structures
      ↓
control flow and memory operations
      ↓
backend operations or intrinsics
      ↓
target instructions
```

Primer does not fix one representation as the only correct form. Instead, it prioritizes observing:

- which representations a computation passed through;
- which structure and semantic information each representation retained;
- where a transformation removed a higher-level abstraction;
- the basis for replacing one representation with another;
- which properties were preserved and which could change.

Specialized representations can preserve intent, carry validation rules, and make optimization candidates easier to discover. Their absence does not make a computation inexpressible. When Primer recognizes a particular structure in a general representation, it records the recognition conditions and transformation. When it cannot recognize the structure, general entity and transformation observations remain available.

### Equivalence of alternative representations

Being able to transform into another representation does not mean the transformation preserves every notion of the same result. Primer needs to distinguish the equivalence claimed by a transformation.

| Equivalence | Preserved property | Example |
| --- | --- | --- |
| Language-semantic equivalence | Observable behavior defined by Primer | Lowering integer arithmetic into an instruction sequence with the same semantics |
| Bitwise equivalence | Output bit patterns and exceptional behavior | A transformation that also preserves floating-point operation order |
| Numerical tolerance | A defined error bound or tolerance | Approximate operations, mixed precision, or quantization |
| User-defined contractual equivalence | Specified conditions such as shape, range, or statistical properties | Approximate transformations for ML or scientific computing |
| Unverified equivalence | No guarantee despite a similar apparent purpose | Replacing a detected pattern without supporting evidence |

For example, `(a + b) + c` and `a + (b + c)` are equal over real-number mathematics but can produce different floating-point values because of rounding. When fusion, reassociation, vectorization, or quantization introduces an alternative representation, the transformation states what it preserves.

Provenance can relate two representations but does not itself prove equivalence. Transformation rules, applicability conditions, type rules, validation, tests, and numerical comparison are combined as needed.

### General observation and domain-specific explanation

For an arbitrary user-defined program, Primer can observe general information such as:

- how types and names were resolved;
- which functions, values, and expressions became which IR entities;
- how an entity was split, merged, or removed;
- which representation a backend and target selected.

However, arbitrary code does not always reveal its domain intent safely. A function combining loops, array access, multiplication, and addition can execute and be traced, but Primer cannot automatically claim whether the intended operation is a convolution, matrix multiplication, or signal filter.

Domain-specific explanations may require explicit semantic information such as:

- known operations in the standard library or another library;
- types, traits, interfaces, or contracts;
- explicit source annotations;
- a named domain-specific lowering or optimization pass.

The mechanism remains a future design decision. Primer must not infer domain intent without evidence, and it must trace transformations when explicit meaning is lost during lowering.

This distinction requires the observation model to support the following:

- identifiers and provenance on user-defined entities;
- preservation of abstraction names and boundaries while they remain useful to observe;
- records of where inlining, fusion, specialization, or another transformation removes an abstraction;
- room to associate typed explicit semantics rather than relying only on arbitrary text;
- general compiler explanations even when no domain-specific information exists.

## How to evaluate a scenario

Each use case is evaluated from the following perspectives:

| Perspective | Question |
| --- | --- |
| Subject of observation | Are values, types, IR entities, transformations, artifacts, or execution state observed? |
| Questions to answer | What should the observation explain? |
| Required capabilities | Are provenance, transformation records, rationales, an Emission Map, or runtime capture required? |
| What remains unobservable | What cannot be concluded from those capabilities alone? |
| Collection method | Is information collected at compile time, runtime, through static analysis, or by external measurement? |
| Observer effect | Can collection alter artifacts, execution order, memory use, or performance? |
| Security | Can observations contain source, inputs, secrets, models, or user data? |
| Reproducibility | Can results be compared under the same conditions, and how are randomness and parallel execution handled? |
| Public boundary | Does the information remain internal, enter a public schema, or belong to an external tool? |
| Implementation cost | What storage, runtime overhead, backend work, and compatibility cost are required? |

A scenario is not used only to justify adopting a feature. It also reveals what a candidate cannot explain and when a mechanism is excessive for the intended use.

## Scenario 1: Numerical computing and ML

As a general-purpose language, Primer aims to express ML computations through combinations of general language features. The current implementation does not yet have the functions, control flow, data structures, libraries, and runtimes needed to do so. Tensor types and specialized operations are optional representations for practicality and optimization rather than prerequisites for expressibility. This scenario does not make ML a specially permitted domain. It examines the value of Primer observability when numerical computation is brought into the language.

For numerical computing, the final value alone is not enough. The types, precision, transformations, and target representations that produced the value matter. Tracing that relationship can be a major Primer advantage.

### Layers to observe

Numerical computing and ML observations can be divided into at least four layers.

#### 1. Numerical meaning

- source-level numeric types;
- type-inference results;
- literal types and representable ranges;
- operand, result, and accumulation types;
- future structured type information such as shape, dimensions, and element type.

This layer describes the numerical computation Primer believes the program means.

#### 2. Numerical transformations

- explicit or implicit type conversions;
- promotion and narrowing in mixed precision;
- quantization and dequantization;
- operation fusion and splitting;
- layout changes for aggregates, vectors, or tensors;
- backend instruction or kernel selection.

This layer describes how resolved numerical meaning is moved into a target representation.

#### 3. Runtime numerical behavior

- values actually computed;
- summaries such as minimum, maximum, mean, and variance;
- `NaN`, positive or negative infinity, overflow, and underflow;
- saturation caused by quantization;
- vanishing or exploding gradients;
- numerical differences across backends or executions.

This layer depends on actual input and execution. It cannot be obtained from compilation alone.

#### 4. Execution characteristics

- execution time;
- code size;
- memory use;
- transfer volume;
- kernel launch count;
- target hardware utilization.

This layer is performance measurement. Primer explains transformations and rationales. Consumers such as Whitebase or a target runtime perform the measurement.

### Useful questions

| Question | Required information |
| --- | --- |
| Why did this value become `f32`? | Provenance and rationale for type resolution |
| Where was precision lost? | Numeric type and conversion chain, plus values or errors before and after conversion |
| Which operation first produced this `NaN`? | Runtime values, operation provenance, and the first anomalous location |
| Why do two backends produce different values? | Shared numerical meaning, each lowering, identical input, and execution conditions |
| Which operations were fused? | Many-to-one provenance and a transformation record |
| Why was this kernel or instruction selected? | Target conditions, capabilities, and a structured rationale |
| Where did quantization error grow? | Scale, zero point, transformation records, and numerical comparisons by stage |
| Where did a gradient vanish? | Operation-graph provenance and runtime gradient summaries |
| Did the transformation actually improve performance? | Primer transformation records and externally measured performance |

## Levels of numerical observation

"Seeing numbers" includes several different capabilities. The selected level changes the available uses and costs.

### 1. Observe only compile-time types and constants

**Enables**

- explain type inference, literal resolution, and conversion instructions;
- inspect mixed-precision paths statically;
- compare numeric types and instruction selection across backends;
- observe without collecting real input values.

**Does not enable or is limited by**

- cannot reveal runtime `NaN`, overflow, or value distributions;
- cannot detect precision loss dependent on input data;
- cannot reveal actual performance.

**Primary uses**

- validation of type design and lowering;
- static auditing of precision conversions;
- backend comparison.

### 2. Analyze static ranges or errors

**Enables**

- conservatively estimate possible value ranges;
- find possible overflow or precision loss before execution;
- inspect numerical risks without storing real inputs.

**Does not enable or is limited by**

- conservative results may include problems that do not occur at runtime;
- balancing analysis cost and precision is difficult for complex operations and loops;
- cannot reveal actual values or frequencies.

**Primary uses**

- numerical-safety diagnostics;
- quantization-candidate analysis;
- range validation before execution.

### 3. Collect runtime summaries

**Enables**

- inspect minimum, maximum, mean, variance, and `NaN` count;
- compare distribution changes without storing complete tensors;
- narrow down the first operation where an anomaly appears.

**Does not enable or is limited by**

- summaries cannot reconstruct individual anomalous values or positions;
- averages can hide local problems;
- collection affects runtime and memory use;
- summaries can reveal properties of the original data.

**Primary uses**

- investigation of `NaN`, overflow, and gradient anomalies;
- quantization calibration;
- comparison of numerical distributions by model layer.

### 4. Sample runtime values

**Enables**

- inspect concrete values while limiting data volume;
- trace values at selected positions, operations, or conditions;
- compare backend differences using concrete examples.

**Does not enable or is limited by**

- anomalies outside the sample can be missed;
- the sampling method changes what is observed;
- reproducible sampling rules are required;
- individual data values have high confidentiality risk.

**Primary uses**

- investigation of numerical differences;
- debugging small models or inputs;
- detailed inspection of an anomaly found in summaries.

### 5. Collect complete runtime values

**Enables**

- perform complete element-level comparison and later reanalysis;
- trace value propagation in detail for one execution;
- apply different summaries after collection.

**Does not enable or is limited by**

- storage is impractical for large tensors or long executions;
- data can directly contain model weights, training data, and user input;
- collection substantially affects execution;
- does not guarantee exact replay of parallel execution or external devices.

**Primary uses**

- small, constrained reproductions;
- detailed validation of a numerical implementation;
- exact backend-difference investigation.

Complete value collection is not a normal observation mode. It is considered only as an explicitly bounded investigation feature.

## Design implications from the ML scenario

Even if ML is not added to the roadmap, Primer can avoid unnecessarily blocking the following capabilities.

### Do not lose numerical meaning early

- Make type, precision, and conversion explicit in Primer IR.
- Do not let backends silently repeat type inference or numerical conversion.
- Leave room for future shape and element types as typed information rather than plain text.

This is consistent with the current rule that type decisions are resolved before backend lowering.

### Do not make one `Span` the limit of provenance

Fusion maps several source operations to one backend entity. Splitting maps one operation to several entities. Provenance needs to represent one-to-many and many-to-one relationships.

### Do not restrict observation data to scalars and text

Future observations may contain shapes, layouts, ranges, statistics, and multiple values. Rather than fixing a public format early, the internal model should support typed, size-bounded data.

### Structure rationales

Free-form text such as `used f32 because it is faster` is difficult to compare. At minimum, Primer should separate a rationale kind, applicable conditions, and target entities from replaceable human-readable wording.

### Separate compile-time observation from runtime instrumentation

Types, IR, lowering, and artifacts can be observed passively without changing artifacts. Capturing runtime values may require instrumentation in the VM or generated code.

Instrumented execution is an explicit mode separate from normal execution.

- Record that an artifact is instrumented.
- Include instrumentation settings in experiment conditions.
- Do not treat instrumented performance as normal performance.
- Verify that semantic output remains unchanged.
- Specify collection destinations and limits.

### Separate compile reproducibility from run reproducibility

Even if the same Primer input deterministically produces the same artifact, runtime values can differ because of randomness, parallel execution, GPUs, external libraries, or input order.

```text
compile reproducibility  same observations and artifacts from the same explicit compilation conditions
run reproducibility      comparable execution results from the same execution conditions and inputs
```

Primer owns the first. The execution owner records seeds, devices, runtimes, libraries, and input identities required for the second.

## Security and privacy

The [Secret value design](secrets.en.md) defines the direction for treating confidential values in the type system.

Runtime numeric values are confidential information separate from source text and file paths. They can include:

- training data;
- user inputs;
- model weights;
- labels;
- embeddings;
- gradients;
- properties of data inferable from statistics.

Permission to disclose source does not grant permission to collect runtime values. Those permissions and settings remain separate.

Runtime observation, if introduced, preserves at least the following rules:

- collection is disabled by default;
- selected values, operations, and stages are explicit;
- element count, byte size, and execution time have limits;
- summaries, samples, and complete collection are distinct;
- huge values and control characters are not written directly to a terminal;
- persistence and external transmission require separate permission;
- observation identifiers cannot be used to execute code or modify values.

## Observer effects on execution

Runtime capture can change:

- instruction scheduling;
- operation fusion;
- memory layout and use;
- device transfers;
- parallel timing;
- execution time.

Performance from an instrumented execution is therefore not reported as normal execution performance.

Primer distinguishes collection methods when necessary:

| Method | Artifact impact | Information available |
| --- | --- | --- |
| Passive compile-time observation | Does not change artifacts | Types, IR, transformations, rationales, and artifact mappings |
| Observation inside the Primer VM | Adds overhead to the VM path | Values and execution state by bytecode instruction |
| Instrumented artifact | Explicitly changes the artifact | Values and events in the target runtime |
| External profiler or runtime | Outside Primer | Device, kernel, performance, and external execution state |

Observation results always identify the method used to obtain them.

## Boundary between Primer and external tools

Primer does not need to own every part of numerical computing or ML observation.

### Candidate Primer responsibilities

- resolve numeric types and numerical meaning;
- record transformations and provenance inside Primer;
- explain why instructions, kernels, and layouts were selected;
- observe explicitly enabled values in the Primer VM;
- retain an Emission Map through Primer-generated artifacts.

### Candidate consumer responsibilities

- manage datasets, models, and inputs;
- run external ML runtimes;
- select devices and libraries;
- measure performance;
- observe external compilers, drivers, and kernels;
- store and compare experiment results.

Associating information received from an external tool is distinct from claiming that Primer performed the external transformation.

## Implementation order not fixed by this scenario

The following can be representations or implementation strategies for practical or efficient ML in Primer. They are not treated as prerequisites for expressing the computation. This scenario does not decide whether they belong in built-in language features, libraries, or external runtimes, and it does not fix their implementation order:

- tensor types;
- automatic differentiation;
- a GPU backend;
- an ML graph IR;
- a kernel compiler;
- runtime instrumentation;
- a model format.

Each requires a separate design decision. This scenario currently asks that Primer preserve general-purpose numerical expressibility and not unnecessarily close future options for numerical explainability.

## Questions for evaluating a design

When designing a new IR, type, observation API, or backend feature, this scenario asks:

1. Can numeric type and precision decisions be traced?
2. Can one-to-many and many-to-one numerical transformations be represented?
3. Can transformation content and rationale be retained separately?
4. Can future typed observations represent non-scalar information?
5. Can observation volume be bounded?
6. Can permissions for source, runtime values, and external transmission remain separate?
7. Can an observer effect from instrumentation be distinguished from normal execution?
8. Can Primer transformations be distinguished from external runtime transformations?
9. Can compile reproducibility and run reproducibility be distinguished?
10. Which use cases become impossible if a capability is not added?
11. Can one computation be traced through several alternative representations?
12. Can a representation-replacing transformation state which equivalence it guarantees?

Primer does not need to implement every capability now. The purpose is to understand when a design closes a future option and make that choice consciously.

## Future scenarios

The same structure can evaluate other use cases:

- error and reproducibility in scientific computing;
- optimization-pass education and visualization;
- comparison across backends and targets;
- compiler investigation by agents;
- security audits and observation without confidential disclosure;
- explaining code size and resource use in embedded environments.

Adding scenarios does not narrow Primer to a single use. It tests how well design candidates withstand different requirements.
