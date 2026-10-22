# 12. Array Method Dispatch — Ownership Boundary

Date: 2026-05-07
Status: Accepted. Legacy-scoped by [ADR 0016](0016-ppr-foundation-and-constitution.md), effective when ADR 0016 is accepted; deleted with the legacy code.

## Context

Tyrus translates JavaScript array methods (`Array.prototype.*`) into idiomatic Rust. Two dispatchers historically share the work, and the split was implicit:

1. `crates/tyrus_codegen/src/convert/expr/call_array.rs::RustGenerator::try_convert_array_method` — fired in `convert_general_call`, after stdlib dispatch. Owns methods whose emission requires IR context: closures with optional `index` callback parameters (`map`/`filter`/`forEach`), state-field detection through `MemberExpr` (`push` against `this.<state-field>`), or non-trivial overload disambiguation (`replace` → 2-arg becomes `replacen(_, _, 1)`).
2. `crates/tyrus_codegen/src/stdlib/array.rs::handle` — fired in `try_handle_method_call`, before any of the call-site IR-aware paths. Owns Vec-only methods that operate on the converted obj-tokens plus arg slice (`find`, `join`, `includes`, `indexOf`, `slice`, `concat`, `reverse`, `pop`, `sort`, `shift`, `flat`, `flatMap`).

The split was correct in principle — IR-context vs. token-level transform — but it was undocumented, and `find` ended up registered in *both* dispatchers. Because `try_handle_method_call` runs first in `convert_call_expr`, the stdlib branch always won; the `call_array.rs` arm was dead code. Tracked as #152 (CRITICAL Rule 8 — eliminate dispatch duplication).

## Decision

The two dispatchers stay separate, with a documented boundary and explicit deferrals. No method appears in both authoritative `match` arms.

### Ownership table

| Method | Owner | Reason |
|--------|-------|--------|
| `map` | `call_array.rs` | TS callback may take `(value, index)` — need `enumerate`/`(closure)(v, i as f64)` template |
| `filter` | `call_array.rs` | Same index-arity branch + inline-filter optimization for arrow predicates |
| `forEach` | `call_array.rs` | Same index-arity branch |
| `some` | `call_array.rs` | Trinity with `every`/`find` via `convert_iter_adaptor_call` |
| `every` | `call_array.rs` | (see above) |
| `reduce` | `call_array.rs` | 1-arg vs 2-arg distinguishes `Iterator::reduce` from `Iterator::fold` |
| `push` | `call_array.rs` | `this.<state>.push(...)` needs Mutex-aware emission via `MemberExpr` introspection |
| `replace` | `call_array.rs` | Arity-driven dispatch to `replacen(_, _, 1)` for 2-arg form |
| `find` | `stdlib/array.rs` | Pure receiver-token operation, no closure context needed beyond the predicate |
| `join`, `includes`, `indexOf`, `slice`, `concat`, `reverse`, `pop`, `sort`, `shift`, `flat`, `flatMap` | `stdlib/array.rs` | (same — pure Vec ops) |

### Deferral protocol

When `stdlib/array.rs::handle` is invoked for a method owned by `call_array.rs`, it returns `None`. The `try_convert_member_call` chain in `convert/expr/call.rs` then unwinds through `try_convert_fetch_call` to `convert_general_call`, which calls `try_convert_array_method` — the call_array path takes over.

The deferral list is concentrated in one match arm:

```rust
// stdlib/array.rs
match method {
    "push" | "map" | "filter" | "forEach" | "some" | "every"
        | "reduce" | "replace" => None,        // owned by call_array.rs
    "find" => handle_find(gen, &obj_tokens, args),
    "join" => handle_join(...),
    // ... 11 pure Vec-op methods
}
```

Both files carry a comment block pointing back to this ADR.

## Consequences

**Positive**:
- The `find` duplicate is gone. A future reader can grep for `"find"` in the codegen tree and find exactly one authoritative emitter.
- Adding a new array method has a deterministic placement rule: needs `MemberExpr` or `CallExpr` IR? → `call_array.rs`. Otherwise → `stdlib/array.rs`. Reviewers can enforce this from the ownership table without re-deriving the design.
- The deferral arm in `stdlib/array.rs::handle` makes the partition explicit at the dispatch site.

**Trade-offs**:
- Two files instead of one. A unified `BuiltinMethodRegistry` (decorator-registry-style trait + handler structs from ADR 0007) would collapse them, but doing so requires lifting state-field detection and `MemberExpr` access through the registry boundary. Out of scope for #152 — tracked separately under #157 (`BuiltinTypeRegistry` consolidation across Map/Set/Promise/Date/Array/Record).
- Reviewers must remember to check both files when changing array semantics. Mitigated by the cross-file comment pointers and the ADR ownership table.

**Negative**:
- None known. The split mirrors a real type distinction (IR-aware vs. pure Vec op); collapsing it without the registry would just push the conditional into one giant function.

## References

- Issue #152 — CRITICAL Rule 8 — Eliminate array-method dispatch duplication
- Issue #157 — MEDIUM — Introduce `BuiltinTypeRegistry` to consolidate Map/Set/Promise/Date/Array/Record dispatch (the longer-term consolidation)
- ADR 0007 — Decorator registry (the registry pattern this ADR defers adopting)
- ADR 0008 — Tyrus Power of Ten (Rule 8: two-layer architecture)
- `crates/tyrus_codegen/src/convert/expr/call_array.rs::try_convert_array_method`
- `crates/tyrus_codegen/src/stdlib/array.rs::handle`
