# 9. Mutex Re-entrance Protocol for Generated `@Injectable` State

Date: 2026-05-03
Status: Accepted (retroactive — backfill of PR #141). Legacy-scoped by [ADR 0016](0016-ppr-foundation-and-constitution.md), effective when ADR 0016 is accepted; deleted with the legacy code.

## Context

`std::sync::Mutex` is **non-reentrant**: a second `.lock()` call from the same thread while a guard is alive deadlocks. Tyrus emits NestJS `@Injectable` services where private fields become `Arc<Mutex<T>>`-wrapped state. Two patterns in pre-PR-#141 codegen triggered the deadlock:

1. **Compound assigns on state fields.** `this.counter += 1` (and `-=`/`*=`/`/=`/`%=`/bitwise/shift) silently mis-compiled because the `StateField` write branch ignored the binary operator and emitted a plain `=`. Even when the codegen *did* route through a read-then-write split, the read guard and the write guard were both alive across the assign expression — second `.lock()` deadlocked.

2. **Dual reads in one statement.** `User { id: this.x, count: this.x }` and `Math.max(this.x, this.x)` kept two `MutexGuard` temporaries alive until the enclosing statement's terminating `;` — Rust's drop semantics would only release them at end-of-statement, not at the consuming expression. The second `.lock()` blocked indefinitely.

Both bugs surfaced under realistic NestJS workloads (counters, rate limiters, dashboards). Neither was caught by snapshot tests because the generated Rust *compiled*; only at runtime, on the second invocation of the controller method, did the worker thread hang.

PR #141 (`b5af050`) closed both bugs by establishing a uniform emission protocol. This ADR documents that protocol so future contributors — particularly anyone implementing #143 (atomic state fields) or revisiting `convert/expr/member.rs` / `convert/expr/misc.rs` — know the invariants are load-bearing, not incidental.

## Decision

Generated Rust for `@Injectable` state fields obeys two emission patterns. Both are implemented in `crates/tyrus_codegen/src/convert/expr/`.

The receiver token is selected by the codegen flag `RustGenerator::use_state_for_this`: handler bodies emitted under axum's `State<Arc<Self>>` pattern use `state.field`, while regular methods use `self.field`. Both paths obey the protocols below; the snippets show `self` for brevity.

Every `.lock()` call uses `.unwrap_or_else(|e| e.into_inner())`, never `.unwrap()`. This is **poisoning-safe**: a panic on another thread that left the mutex poisoned is recovered (the inner value is still consistent because the codegen never holds a guard across a fallible operation). Rule 6 (no `.unwrap()` in production code) is satisfied while preserving the lock semantics.

### Pattern A — Block-scoped read

Every read of `this.field` where `field` is a tracked state field expands to one of two forms, depending on whether the payload type is `Copy`:

```rust
// Copy payload (f64, bool, i32, usize, u64, i64): deref the guard.
{ let __g = self.field.lock().unwrap_or_else(|e| e.into_inner()); *__g }

// Non-Copy payload (String, Vec, etc.): clone via the guard.
{ let __g = self.field.lock().unwrap_or_else(|e| e.into_inner()); __g.clone() }
```

The guard `__g` is bound in an **inner block**, so Rust's drop scope ends at the closing `}` of that block — *before* the consuming expression evaluates the next subexpression. Two reads in the same statement therefore release each guard between reads.

This is implemented in `convert_this_member` (`convert/expr/member.rs`) and reached via `convert_member_expr` for every `Expr::Member` whose object resolves to `self`/`state` and whose property is registered in `current_class_state_fields`.

### Pattern B — Read-then-write split for compound writes

Compound assignments and update expressions (`+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`, prefix/postfix `++`/`--`) on state fields expand into a block holding two statements:

```rust
{
    let __new_val = #guarded_read <op> #rhs;
    *self.field.lock().unwrap_or_else(|e| e.into_inner()) = __new_val;
}
```

`#guarded_read` is the inline form `{ let __g = self.field.lock().unwrap_or_else(|e| e.into_inner()); *__g }` (Pattern A's Copy form, since compound assigns operate on numeric state).

The read guard drops at the `;` of the `let __new_val = …` line. The write guard drops at the `;` of the assignment line. The two locks therefore never coexist. The operator is preserved (no silent reduction to `=`). Update expressions (`this.x++`, `this.x--`) route through the same helper (`emit_state_field_assign`) for uniform semantics.

### Invariant

For any generated function body emitted from a method on an `@Injectable` class, **at most one `MutexGuard` for any given state field is alive at any program point**. The compiler does not enforce this — codegen does. Any new emission path that touches state fields **must** either:

- route through the existing helpers (`convert_this_member` for reads; `emit_state_field_assign` for compound writes), or
- replicate the inline form (block-scoped guard + `.unwrap_or_else(|e| e.into_inner())`) explicitly, including the receiver dispatch on `use_state_for_this`.

## Consequences

### Positive

- **Deadlock-free under realistic workloads.** The two failure patterns that triggered #141 are architecturally impossible under the protocol.
- **Predictable mental model.** "Each read = one guard, dropped at end of inner block. Each write = read-block then write-block." Reviewers do not need to reason about `MutexGuard` drop order across complex expressions.
- **Forward compatibility.** When #143 lands atomic state fields (`AtomicU64` etc.), the `emit_state_field_*` helpers are the single substitution point — no code outside them needs revision.

### Negative

- **Slight overhead.** Every state-field read costs a block scope and a deref/clone vs the naive `self.field.lock()…`. In practice negligible (the lock acquisition is the dominant cost). Atomic state (#143) eliminates the overhead for `Copy` payloads.
- **Snapshot churn.** PR #141 updated the tier4 NestJS controller and injectable-service snapshots to the guarded-block form. Future codegen tweaks affecting state reads will keep producing snapshot diffs.
- **Codegen complexity.** The path from `Expr::Member { obj: This, prop: Ident }` to emitted tokens passes through `convert_member_expr` → `convert_this_member` and consults a tracked HashMap (`current_class_state_fields`) plus a flag (`use_state_for_this`). Anyone adding a new mutating expression (e.g., a hypothetical `Math.atomic_add`) must explicitly handle the state-field branch and propagate the receiver dispatch.

## Alternatives rejected

- **`parking_lot::ReentrantMutex`.** Would lift the constraint at the cost of (a) extra dependency in every generated `Cargo.toml`, (b) loss of `Send + Sync` guarantees that NestJS `@Injectable` services rely on for axum's `with_state`, (c) silent acceptance of recursive locking — exactly the pattern Rule 1 (bounded control flow) discourages. Rejected.
- **Skip-on-error pattern (`if let Ok(g) = self.field.try_lock())`.** Trades deadlock for silent data corruption (lost writes). Worse outcome. Rejected.
- **Compile-time check via macro (`#[state]`).** Would require a procedural macro on every generated field. Adds a build dependency and a parsing step we control end-to-end via codegen. The codegen-side discipline is the simpler enforcement. Rejected for now; revisitable if user-authored Rust ever needs the same protection.

## References

- Originating PR: [#141](https://github.com/Gefferson-Souza/Tyrus/pull/141) (`b5af050`) — `fix(codegen): prevent mutex re-entrance on compound assigns and dual reads`. Closes #129.
- Implementation:
  - `crates/tyrus_codegen/src/convert/expr/member.rs::convert_this_member` (read path).
  - `crates/tyrus_codegen/src/convert/expr/misc.rs::emit_state_field_assign` (compound write path).
- Receiver dispatch: `RustGenerator::use_state_for_this: Cell<bool>` toggled around handler-body codegen.
- State-field tracking: `RustGenerator::current_class_state_fields: HashMap<String, String>` (field name → Rust type name; populated in `class/state_field.rs`).
- Regression tests: `tests/src/unit/state_mutation.rs` (covers compound `+=`/`-=`, update `++`/`--`, dual read in struct literal, dual read in fn args).
- Related: ADR 0008 (Tyrus Strict Rules — this ADR satisfies Rule 10's retroactive requirement); future ADR for #143 (atomic state) supersedes Pattern B for `Copy` payloads.
