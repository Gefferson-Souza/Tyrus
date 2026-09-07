---
paths:
  - "crates/tyrus_codegen/**"
---

# Codegen Rules (binding — R7/R8, ADRs 0007/0012)

- **R7:** Rust is emitted through `quote!`/`proc_macro2::TokenStream` ONLY. `format!`
  of Rust source is forbidden anywhere under `src/convert/`. The single string-Rust
  touchpoint in the workspace is `tyrus_orchestrator::format`.
- **R8 (two layers):** structural translation dispatches by AST node *type*; anything
  keyed by *name* (decorators, stdlib calls, framework symbols) goes through a registry.
  Scattered `match name { "X" => … }` arms across files are forbidden. Adding a NestJS
  decorator costs exactly: 1 handler file in `src/decorators/` + 1 `register_*` line in
  `default_registry()` + 1 variant in `tyrus_decorator_kinds::DecoratorKind`.
- Decorator name → kind classification happens ONLY via `DecoratorKind::from_name`.
  Never compare decorator names with raw strings.
- **Decorators are generic (owner rule, restated 2026-09-07 — #265):** `Decorator` is one
  AST node type and follows the same two-layer rule as every call and method. ONE
  structural handler translates ALL decorators; the registry overrides only the NestJS
  structural set (`@Module`, `@Injectable`, `@Controller`, routing, params) because Axum
  needs different semantics there. Any other decorator — framework or user-defined — gets
  the generic translation. Never an analyzer error, never a silent drop, never a new
  handler "because the name is unknown".
- **ADR 0012 boundary:** array methods needing IR context (map/filter/forEach/some/
  every/reduce/push/replace) live in `convert/expr/call_array.rs`; pure `Vec` ops live
  in `stdlib/array.rs`. A handler that can't decide returns `None` to defer — it never
  panics (invalid TS degrades to the generic translation; the generated `cargo build`
  surfaces the error).
- Unsupported constructs in generated code emit `compile_error!("Tyrus: …")` — never
  `todo!()` and never a silent wrong translation.
- String-vs-array method dispatch relies on `string_vars`/`map_vars`/`set_vars` state
  on `RustGenerator` — that state is per-file and not lexically scoped; when touching
  it, note the collision risk across same-named variables in one file.
- **R5/F2:** every change that affects emitted Rust ships a semantic equivalence test
  in `tests/src/equivalence/` written RED first (Node stdout ≡ compiled Rust stdout).
  Pure refactors instead require a green `cargo insta review` diff.
