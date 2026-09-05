# Output routes and targets

[日本語](targets.ja.md)

This document distinguishes output routes, targets, artifacts, and backends in Primer's generated output.

This distinction is necessary to identify observations from the same Primer program correctly and to preserve room for additional instruction sets and execution environments.

## Terminology

### Output route

An output route describes which kind of representation is produced from Primer IR.

The current output routes are C, LLVM IR, QBE IR, WebAssembly Text, native assembly, and Primer bytecode.

### Target

A target describes the execution environment assumed by an artifact. It has the following elements when they are relevant:

- instruction set architecture, such as `x86_64`, `riscv64`, or `wasm32`;
- execution environment, such as Windows, Linux, or bare metal;
- ABI, such as Windows x64, System V, or RISC-V LP64D;
- features, such as the RISC-V M, A, F, and D extensions.

Some output routes do not have a target selected by Primer. For example, the caller of an external C compiler may decide which environment a C source artifact is compiled for.

### Artifact

An artifact is the actual observable output, such as C source, LLVM IR, assembly, or Primer bytecode.

An artifact format may have attributes such as assembly syntax or file format. Even when two artifacts are both assembly, GNU AT&T syntax and Intel syntax are distinct formats.

### Backend

A backend is an internal implementation component that lowers Primer IR for a particular output route and target and then produces an artifact.

Backend is an implementation term. It is not used as a collective name for an output route, target, and artifact.

## Current configuration

The current outputs can be described as follows:

| Output route | Target | Artifact |
| --- | --- | --- |
| C | not selected by Primer | C source `.c` |
| LLVM IR | not selected by Primer | LLVM IR `.ll` |
| QBE IR | not selected by Primer | QBE IR `.ssa` |
| WebAssembly Text | WebAssembly | WAT `.wat` |
| Native assembly | x86-64, Windows, Windows x64 ABI | GNU-style assembly `.s` |
| Primer bytecode | Primer VM | Primer bytecode `.pbc` |

"Not selected by Primer" does not mean inferred implicitly from the host environment. It means that Primer does not include target-specific decisions in that observation and that the caller of a downstream tool selects the target.

## Artifact consumer boundary

An existing output route does not imply support for every language feature. Strings currently lower to C and Primer bytecode. Other routes diagnose strings before lowering, including unused definitions. Successful semantic validation is distinct from successful generation through each route.

Artifact comparison separates the following questions. Support information is data, not permission to launch external programs.

| Question | What is checked | Owner |
| --- | --- | --- |
| Can it be generated? | Whether the Primer version supports the output route and target; source-specific errors are diagnosed during generation | Primer |
| Can it be built? | Whether suitable compilers, assemblers, linkers, and required libraries are available | Consumer |
| Can it be executed? | Whether CPU, OS, runtime libraries, and host functions meet the requirements | Consumer |
| May it be executed? | Whether user authorization and execution-environment restrictions permit it | Consumer execution policy |

Finding an external tool does not establish that a build or execution will succeed. Keep preflight checks separate from actual stage results, distinguishing unsupported routes, missing tools, policy denial, generation failure, build failure, execution failure, and timeout. Do not silently drop unexecutable routes or count them as successes.

The environment running Primer is separate from the artifact target. Calling `emit-asm` in WSL still produces GNU AT&T assembly for Windows x86-64. Linux assembly requires a separate implementation; support checks alone cannot add it. Even on Windows, Primer's internal aggregate passing convention does not guarantee external C ABI compatibility.

### What comparison establishes

The current CLI runs source in the VM through `run` and returns artifacts through `emit-*`. Comparing exit status and standard output through these commands does not require a new public API. There is no CLI for loading and executing `.pbc` files in the VM.

Matching standard output establishes matching displayed results. It does not prove equality of unprinted information such as floating-point NaN payload bits. If bitwise comparison becomes necessary, design a separate typed observation format. Numeric tolerances and newline normalization must also be explicit comparison conditions when used.

Do not treat the VM alone as an oracle. Retain checks against known expected results and language rules. Two nonzero exits do not establish the same cause of failure. Preserve the failing stage (generation, build, or execution) and each stage's diagnostics.

### Recording and safety

The consumer records input and artifact identity, Primer build identity, output route, actual target, external-tool versions and explicit options, stage results, and comparison conditions. `primer --version` reports the package version. To distinguish development builds sharing a version, also record the verified commit or executable hash. A machine-readable supported-route listing is not implemented. Define a versioned public schema from actual consumer needs when one becomes necessary.

Do not interpret strings in artifacts or observation data as executable commands or authorization. Executing generated code is a separate operation from reading observations. The consumer selects trusted tools and arguments and runs untrusted code in appropriate isolation. Timeouts and separate processes alone are not a security sandbox; also bound captured output. Do not unconditionally record source text, paths, or entire environments.

## Observation identity

When a target affects an artifact, an observation is identified by at least the following conditions:

- Primer version;
- source input;
- output route;
- target;
- target features;
- explicit options.

When these conditions are the same, Primer produces a deterministic observation.

Target information does not have to be embedded in the artifact itself. However, the output route and target that produced an artifact must be identifiable from the CLI invocation or associated metadata.

Selecting a target implicitly from the host OS, host CPU, or environment variables would allow the same explicit input to produce different results. Primer does not make implicit target choices that affect observations.

## Lowering boundary

Primer IR preserves meaning that is independent of output routes and targets.

Decisions such as the following are made after backend lowering begins:

- instruction selection;
- register use;
- stack-frame layout;
- value size, layout, and alignment;
- calling conventions;
- instructions selected for target features;
- assembly syntax and artifact format.

Target-specific IR may remain an internal boundary of each backend. Primer does not require a universal machine IR shared by every target.

## Adding a target

Adding a target preserves the following properties:

- the target has a stable identity;
- supported combinations of output routes and targets are explicit;
- target-specific decisions do not enter Primer IR;
- unsupported combinations produce diagnostics;
- target features have deterministic ordering and representation;
- the same explicit conditions produce the same artifact.

Adding a target does not grant new authority to an observation API. A target identifier is data that selects lowering conditions, not authority to execute external commands or mutate compiler state.

Selection and execution of external assemblers, linkers, and compilers remain the responsibility of the consumer of Primer artifacts.
