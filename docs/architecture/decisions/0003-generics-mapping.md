# 3. Generics Mapping (TypeScript -> Rust)

Date: 2026-02-10
Status: Accepted

## Context

TypeScript has a structural type system with very flexible generics (`any`, partial constraints). Rust has a nominal system with monomorphization.
We need to let users define generic classes and functions in TS that compile to Rust.

## Decision

We will map TS Generics to Rust Generics with default Trait Bounds.

### Mapping Rules:

1.  **Declaration:**
    - TS: `class Box<T> { ... }`
    - Rust: `struct Box<T> { ... }`

2.  **Automatic Trait Bounds:**
    - Every generic parameter `T` in Rust will automatically receive:
      `T: serde::Serialize + serde::Deserialize + Clone + Debug + Default`
    - _Rationale:_ Backends need to serialize data (JSON), clone state, and debug. Without these traits, using `T` would be too restricted.

3.  **PhantomData:**
    - If a `T` parameter is declared but not used in the struct's fields:
    - Rust: Add a `_phantom: std::marker::PhantomData<T>` field.
    - _Rationale:_ The Rust compiler rejects unused generic parameters.

4.  **Generics Inheritance:**
    - We will not support complex TS constraints (`T extends keyof U`) in the first release. They will be treated as plain `T`.

## Consequences

### Positive

- Enables reusable DTOs (`ApiResponse<T>`).
- Ensures generic types are useful (serializable).

### Negative

- Excessive restriction: not every `T` needs to be `Default`, but we are forcing it. This may prevent the use of types that don't implement `Default`.
- _Mitigation:_ In the future, we could analyze type usage to relax the bounds.
