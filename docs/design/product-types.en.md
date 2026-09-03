# Named product type design

[日本語](product-types.ja.md)

**Status: Implemented**

This document organizes the semantics, syntax, observability, and implementation boundaries of named product types, the first user-defined type introduced in Primer.

The [language reference](../reference/language.en.md) defines the rules needed to use the feature. This document also explains why those rules were chosen and how values are transformed into emitted artifacts.

The [compiler architecture](architecture.en.md) defines the overall compiler structure, the [compiler evolution plan](evolution-plan.en.md) defines development order, and the [Secret value design](secrets.en.md) covers values containing secrets.

## Purpose

A product type groups multiple values into named fields and treats them as one meaningful value.

```primer
type Point {
    x: f64,
    y: f64,
}
```

In addition to grouping values, Primer must make the following information observable:

- the types and fields defined in source;
- which fields were explicit and which used defaults during construction;
- the type and field to which each field access resolved;
- how an aggregate was decomposed into memory, locals, registers, and instructions by a backend;
- the transformation stage at which type-level abstraction was lost.

## Agreed direction

The first product type has the following properties:

- `type` is the general entry point for declaring a type;
- types use nominal identity and are distinguished by name;
- each field has a name and a resolved type;
- fields in type definitions and aggregate literals are comma-separated;
- a trailing comma is permitted;
- fields are not directly mutated after construction;
- `mut` permits reassignment of the whole binding, not mutation inside an aggregate;
- aggregates have value semantics at the language level;
- physical copying, sharing, and decomposition are not fixed by the language;
- type names are visible throughout the top level of the same file;
- type names and value names use separate namespaces, while field names are scoped to their type;
- backend-independent type and field meaning is resolved before Primer IR;
- explicit aggregate literal values are evaluated in source order, followed by omitted defaults in type-definition order;
- memory layout and ABI decisions happen during or after backend lowering.

## Syntax

```text
type_definition :=
    "type" IDENT "{"
        field_definition ("," field_definition)* ","?
    "}"

field_definition :=
    IDENT ":" type ("=" expression)?

aggregate_literal :=
    IDENT "{"
        field_value ("," field_value)* ","?
    "}"

field_value :=
    IDENT ":" expression

field_access :=
    postfix "." IDENT
```

The current implementation rejects product types with no fields and empty aggregate literals. A diagnostic points to the `{}` in the type definition and explains that at least one field is required.

Empty types can represent markers and states that carry no data. However, C has no standard empty struct, and physical representations of zero-sized values differ by backend. Primer will add them only after their use and observation behavior are designed separately. Allowing empty types later does not change the meaning of programs accepted today.

## Type definitions

Field types are specified by the type definition:

```primer
type Point {
    x: f64,
    y: f64,
}
```

The field types are not repeated at every construction site:

```primer
point: Point = Point {
    x: 1.0,
    y: 2.0,
};
```

A field may use a built-in type or another user-defined type:

```primer
type Line {
    start: Point,
    end: Point,
}
```

`infer` is not accepted as a field type. A type definition must have a resolved shape independent of its use sites.

```primer
type Point {
    x: infer, // error
}
```

The type of a binding may still be inferred from an aggregate literal:

```primer
point: infer = Point {
    x: 1.0,
    y: 2.0,
};
```

## Type identity and name resolution

Types with identical fields remain distinct when their names differ:

```primer
type Point {
    x: f64,
    y: f64,
}

type Velocity {
    x: f64,
    y: f64,
}
```

There is no implicit conversion between `Point` and `Velocity`.

A type definition is a top-level item, not an executable statement. Conceptually, the AST uses a form that can also accommodate future function definitions:

```text
Program
  Item::TypeDefinition
  Item::Statement
```

Type names are available throughout the top level of the same file regardless of declaration order:

```primer
type Line {
    start: Point,
    end: Point,
}

type Point {
    x: f64,
    y: f64,
}
```

The compiler resolves them in these stages:

1. register all top-level type names;
2. resolve every field type;
3. detect cycles that have infinite size by value;
4. type-check default values;
5. type-check executable statements.

Names are managed in separate namespaces according to their role. A namespace is the collection in which a name is registered and looked up.

- type names are registered in the type namespace;
- bindings and future function names are registered in the value namespace;
- field names are managed within the type that owns them;
- a scope cannot contain multiple definitions with the same name in the same namespace;
- a type name and a value name may use the same spelling.

The compiler chooses a namespace from the syntactic position of a name. `Point` in a type annotation and `Point` beginning an aggregate literal look in the type namespace. `point` in an expression looks in the value namespace.

```primer
type Point {
    x: f64,
}

point: Point = Point {
    x: 1.0,
};
```

A type and value may use the same spelling, although names that remain easy for readers to distinguish are recommended. The language does not assign roles by requiring particular capitalization.

Semantic analysis resolves every name reference to an entity kind and identifier. Conceptually, the retained information includes:

```text
type-ref Point -> TypeId 0
value-ref point -> BindingId 3
field-ref x -> FieldId 0
```

If a name is absent from the required namespace but exists in another namespace, the diagnostic explains the mismatch. For example, using a value named `Point` in a type position reports that a value exists but a type is required instead of only saying that the name was not found.

## Recursive types

Semantic analysis diagnoses types whose size cannot be determined because they contain one another directly by value:

```primer
type A {
    b: B,
}

type B {
    a: A,
}
```

If fixed-size references are introduced later, recursive types through references will be designed separately. This decision does not reject future references or recursive types.

## Default values

A type author may define an explicit default for a field:

```primer
type Options {
    retries: i64 = 3,
    verbose: bool = false,
    timeout: f64,
}
```

A field without a default must be supplied by an aggregate literal:

```primer
options: Options = Options {
    timeout: 10.0,
};
```

This construction uses defaults for `retries` and `verbose`. Primer does not introduce an implicit zero value for every type.

Defaults follow these rules:

- a default must match the field type;
- it is applied for each aggregate construction;
- an explicit field value suppresses that field's default;
- use of a default is retained as structured information in Primer IR;
- the first implementation handles expressions that do not depend on runtime bindings or another field of the same aggregate.

The final item scopes the first implementation and does not permanently restrict which expressions may be supported later.

## Aggregate literals

Because fields are named, their order in an aggregate literal does not need to match definition order:

```primer
point: Point = Point {
    y: 2.0,
    x: 1.0,
};
```

Semantic analysis diagnoses:

- an unknown type;
- an unknown field;
- duplicate fields;
- a missing field without a default;
- a field value whose type does not match.

Explicit field expressions in an aggregate literal are evaluated in source order. Defaults for omitted fields are then evaluated in type-definition order. A default is not evaluated when that field was supplied explicitly.

Each result is associated with its resolved field through `FieldId`. Evaluation order, deterministic field presentation in Primer IR, and physical layout chosen by a backend are therefore separate information.

Primer IR structurally retains both the actual evaluation order and the mapping to `FieldId`. It may present the field list deterministically in type-definition order, but must not reorder expression evaluation to match that presentation. Future function calls and runtime failures therefore cannot make behavior an accidental backend property.

## Field access

Field access uses `.` and can be nested:

```primer
print(point.x);
print(line.start.y);
```

Semantic analysis resolves a field access to its type and field. Backends do not receive an unresolved field name and repeat this lookup.

In an `if` or `while`, the `{` immediately after the condition starts the body. Parenthesize a construction expression when accessing one of its fields in a condition.

```primer
if (Flags { enabled: true, }).enabled {
    print(true);
}
```

## Immutability and reassignment

Fields are not directly mutated after construction:

```primer
point.x = 3.0; // error
```

A `mut` binding may be reassigned a new value of the same aggregate type:

```primer
mut point: Point = Point {
    x: 1.0,
    y: 2.0,
};

point = Point {
    x: 3.0,
    y: point.y,
};
```

`mut` does not grant external mutation authority over a stored aggregate. It means that the current binding may receive another value of the same type.

## Value semantics

Placing an aggregate in another binding does not create observably shared mutable state:

```primer
mut a: Point = Point {
    x: 1.0,
    y: 2.0,
};

b: Point = a;

a = Point {
    x: 3.0,
    y: 2.0,
};

print(b.x); // 1.0
```

A backend or runtime may physically copy, share, or decompose the value as long as this meaning is preserved. Concrete copy, move, borrow, and reference-identity mechanisms are decided when types and operations requiring them are designed.

## Primer IR

Primer IR retains at least the following meaning:

```text
TypeId
FieldId
TypeDefinition
FieldDefinition
Type::Named(TypeId)
Construct
FieldAccess
FieldValueOrigin
```

`TypeId` represents type identity. `FieldId` represents a resolved field within a type. Both are deterministic compilation-local identifiers.

The text representation has the following form, with its exact spelling fixed by snapshots.

```text
type %Point@0 {
  field %x@0: f64 = 0.0f64
  field %y@1: f64 = 0.0f64
}

%point@0: %Point@0 = construct %Point@0 {
  field %x@0 = 10.0f64 [explicit]
  field %y@1 = 0.0f64 [default]
}

print.f64 field %point@0.%x@0
```

`explicit` and `default` are structured internal information rather than display-only annotations. A field using a default is also associated with the source range that defined the default.

## Backend lowering

Primer IR retains type names, field names, types, construction, and field access. The following decisions happen during or after backend lowering:

- total aggregate size;
- field offsets and alignment;
- padding;
- placement in memory, locals, and registers;
- physical copying or sharing;
- ABI passing rules.

The current lowering uses C structs, LLVM named aggregates, QBE `alloc8` storage, WAT linear memory, the x86-64 stack, and structured Primer VM values. QBE copies values with `blit`; WAT and x86-64 use field loads and stores. These differences are observable in emitted artifacts.

The current internal WAT, QBE, and x86-64 layouts reserve eight bytes for each scalar field. This is not a language-level promise about external ABI or future layout.

Source syntax for forcing a physical layout and compatibility with an external ABI are outside this design's scope.

## Relationship to Secret

An ordinary user-defined type is not by itself a security boundary that hides content.

If `Secret` is later combined with an aggregate or field, Secret redaction and propagation rules take precedence over ordinary observation. An origin such as `default` or `explicit` may remain visible, but secret content must not be emitted.

```text
field token = <secret> [default]
```

Implementing aggregates and deciding the final syntax or release mechanism for `Secret` remain separate tasks.

## Current implementation scope

The following scope is implemented:

- top-level named product types declared with `type`;
- nominal type identity;
- fields containing built-in or user-defined types;
- aggregate construction from explicit and default values;
- field access;
- whole-aggregate binding and reassignment;
- file-wide type-name resolution;
- diagnostics for type cycles with infinite size by value;
- types, fields, construction, field access, and value origins in Primer IR;
- lowering into bytecode, the VM, and every backend;
- normal, diagnostic, and observation snapshots;
- synchronized Japanese and English documentation.

`check`, Primer IR, bytecode, the VM, C, LLVM, WAT, QBE, and Windows x86-64 all handle the same language-level semantics. Tests fix the successful behavior, diagnostics, and all eight observation artifacts.

## Deferred features

The following are not rejected. They are separated into later design decisions:

- `with` expressions that produce a new value with selected fields replaced;
- aggregate `==` and `!=`;
- `print(aggregate)` and stable formatting;
- type aliases and newtypes;
- tuples, arrays, sum types, and generic types;
- field visibility and module boundaries;
- copy, move, borrow, and references;
- recursive types through references;
- custom layout and external ABI;
- concrete `Secret` representation and propagation.

## Verification

The product observation fixture transforms one source program containing defaults, value copying, whole-binding reassignment, and nested field access into every artifact. C, LLVM, and Windows x86-64 output can additionally be checked with `clang`. Environments without WAT or QBE runtimes verify their structured IR and snapshots.
