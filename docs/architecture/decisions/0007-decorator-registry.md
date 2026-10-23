# 7. Decorator Registry

Date: 2026-05-02
Status: Accepted. To be replaced for new work by [ADR 0016](0016-ppr-foundation-and-constitution.md) (contradicts foundation decision D5) once ADR 0016 is accepted; legacy-scoped until the code is deleted.

## Context

Tyrus translates a finite catalog of NestJS decorators (`@Controller`, `@Get`/`@Post`/..., `@Body`, `@HttpCode`, `@UseGuards`, etc.) into Axum handler scaffolding. Until PRs #109/#111/#115, each decorator was recognized through string-compare match arms scattered across four hot files:

- `crates/tyrus_codegen/src/convert/class/method.rs:61` — `matches!(name, "Get" | "Post" | "Put" | "Delete" | "Patch")` to extract the verb.
- `crates/tyrus_codegen/src/convert/class/routing.rs:131-138` — a **second** `match http_method.as_str()` to map the same verb to its `axum::routing::*` constructor.
- `crates/tyrus_codegen/src/convert/class/method.rs:121-145` — `find_param_decorator` returning `Option<String>`, then a match on `"Body"`/`"Param"`/`"Query"` to pick the extractor.
- `crates/tyrus_analyzer/src/decorators.rs:46-110` — yet another `if ident.sym == *"Module"` / `Injectable` / `Controller` ladder for DI graph extraction.

Plus a heuristic `class_name.ends_with("Controller")` in `class/mod.rs:26` that flagged classes as controllers by **string suffix** rather than by decorator presence.

This violated two rules already on record in the project memory:

> `feedback_compiler_fundamentals.md`: **Two Layers — Generic AST Handler + Semantic Registry (HashMap, config, trait impls), NOT match arms scattered across files.**

> `feedback_architecture_principles.md`: **NEVER add a `match name { "Foo" => ... }` inside a structural handler.**

The drift was set to compound. Phase 9.1 (`class-validator` → `garde`) introduces 11 new validation decorators (`@IsString`, `@IsEmail`, `@MinLength`, `@MaxLength`, `@Min`, `@Max`, `@IsOptional`, `@ValidateNested`, `@IsPhoneNumber`, `@Matches`, `@IsNotEmpty`). Without restructuring, each new decorator costs ~3 file edits in hot paths. Linear growth in scattered match arms = quadratic risk of regression.

## Decision

Replace string-compare dispatch with a **trait-based registry**, in two complementary crates:

### `tyrus_decorator_kinds` (zero-dep, lightweight)

Single source of truth for decorator name → kind classification:

```rust
pub enum DecoratorKind {
    Module, Injectable, Controller, UseGuards,           // class
    HttpGet, HttpPost, HttpPut, HttpDelete, HttpPatch,   // method
    HttpCode,
    Body, Param, Query,                                   // param
}

impl DecoratorKind {
    pub fn from_name(name: &str) -> Option<Self> { ... }
    pub fn scope(self) -> DecoratorScope { ... }
    pub fn axum_routing_fn_name(self) -> Option<&'static str> { ... }
}
```

This crate has zero dependencies (no `proc_macro2`, no `swc`), so it is freely consumed by both `tyrus_analyzer` (for DI graph classification) and `tyrus_codegen` (for handler dispatch) without dragging compiler-only crates into the analyzer.

### `tyrus_codegen::decorators` (registry + handlers)

Three traits, one per scope:

```rust
trait ClassDecoratorHandler  { fn kind(&self) -> DecoratorKind; fn apply(&self, class, call, ctx: &mut ClassDecoratorContext);  }
trait MethodDecoratorHandler { fn kind(&self) -> DecoratorKind; fn apply(&self, method, call, ctx: &mut MethodDecoratorContext); }
trait ParamDecoratorHandler  { fn kind(&self) -> DecoratorKind; fn emit_extractor(&self, param, name, type) -> TokenStream; }

pub(crate) struct DecoratorRegistry {
    class:  HashMap<DecoratorKind, Box<dyn ClassDecoratorHandler>>,
    method: HashMap<DecoratorKind, Box<dyn MethodDecoratorHandler>>,
    param:  HashMap<DecoratorKind, Box<dyn ParamDecoratorHandler>>,
}
```

A process-wide `OnceLock<DecoratorRegistry>` carries the default instance built once via `default_registry()`. Lookup is O(1); the `apply_*_decorators` methods iterate decorators of a node and dispatch to handlers.

### File layout

Adding a new decorator is now:
1. Add one variant to `DecoratorKind` and one arm in `from_name`.
2. Write one handler struct (10-30 lines) — colocated by **scope**, not by decorator name (`params.rs` holds `BodyHandler`, `ParamHandler`, `QueryHandler` together; `http_method.rs` holds five handler instances of one parameterized struct).
3. Register it in `default_registry()`.

No hot-path file is touched.

### Heuristic deletion

`class_name.ends_with("Controller")` in `class/mod.rs:26-30` was deleted. The `is_controller` flag now comes from `ControllerInfo.is_controller`, populated by `ControllerHandler` when the registry observes an actual `@Controller(...)` decorator on the class.

## Alternatives Considered

1. **Procedural macros (`tyrus_macros` crate).** The fully-generic transpilation plan from `.claude/plan/registry-migration.md` proposed emitting `#[controller(...)]` attributes and resolving them at the consumer's compile time via proc macros. Rejected for now: it adds substantial cognitive surface (proc macros, hygiene, syn AST) and slows generated-crate compilation. The registry achieves the architectural property (no scattered match arms) without that cost. The proc macro option remains open as a future PR if extensibility outside the transpiler becomes a priority.

2. **Generic decorator translation (no registry).** Treat every decorator uniformly: `@Foo(args)` → `#[foo(args)]`. Compiles and transpiles arbitrary decorators but provides no NestJS-specific behavior. Rejected because the value of Tyrus is precisely the *semantic mapping* (`@Get` → `axum::routing::get`, `@Body()` → `axum::Json<T>`) that a generic translator cannot do.

3. **Keep status quo (incremental Sprint 2-9 of `tyrus-production-roadmap-v3.md`).** Add new validation decorators directly into the existing match arms. Rejected per the audit: each Sprint multiplies the dispatch surface; refactoring becomes more expensive at each step. Doing the registry first is cheaper than the cumulative cost of the alternative.

## Consequences

### Positive

- **Single source of truth.** `tyrus_decorator_kinds::DecoratorKind` is consulted by both the analyzer (for DI graph) and the codegen (for handler dispatch). The literal strings `"Get"`, `"Body"`, `"Controller"`, `"Module"`, etc. appear in **one** place — `from_name`. Renames are localized.
- **O(1) decorator addition.** PR #5 will add `@Headers` by editing only `params.rs` and `default_registry()`. Zero changes to `class/method.rs`, `class/routing.rs`, or any analyzer file.
- **No more `unwrap_or` in generated code for status codes.** `map_status_code` migrated from a `from_u16(...).unwrap_or(...)` runtime fallback to a static `STATUS_CODES: &[(u16, &str)]` table + `compile_error!` for unrecognized codes.
- **Strong-typed dispatch.** `MethodDecoratorContext::http_method` is `Option<DecoratorKind>` instead of `Option<String>`; the `Option<String>` round-trip in `convert_single_param` is gone.
- **Decorator-driven controller detection.** A class is now flagged as a controller iff it carries `@Controller(...)`, not because of its name suffix.

### Negative

- **One more crate (`tyrus_decorator_kinds`).** Marginal cost; offset by the reduction in cross-crate string duplication.
- **`Box<dyn Handler>` adds vtable lookup overhead.** Negligible (decorator dispatch happens at codegen time, not at the generated server's request path), but visible in profiling.
- **More files in `decorators/`.** Consolidated where natural (`params.rs` holds 3 handlers; `http_method.rs` holds 5 instances of one parameterized struct), kept separate where the handler has its own state or non-trivial parsing (`controller.rs`, `use_guards.rs`, `http_code.rs`). Future cleanup may merge `controller.rs` + `use_guards.rs` into `class_decorators.rs` if file count becomes a concern.

### Neutral

- Generated Rust output is **byte-identical** for all existing fixtures (verified by snapshot tests in `tier4_nestjs/` and the HTTP equivalence E2E). The refactor is invisible to consumers of Tyrus.

## Implementation Trail

- PR #109: skeleton + class-level handlers (`@Controller`, `@UseGuards`)
- PR #111: method-level handlers (`@Get`/`@Post`/`@Put`/`@Delete`/`@Patch` + `@HttpCode`); duplicate-verb dispatch eliminated; `unwrap_or` removed from generated status-code emission; isolation tests added (PR #111 reinforcement)
- PR #115: param-level handlers (`@Body`, `@Param`, `@Query`); `find_param_decorator` deleted
- PR #116 (this ADR): documentation sync
- PR #5 (planned): empirical proof — implement `@Headers` purely via the registry, touching zero legacy files

## References

- `crates/tyrus_decorator_kinds/src/lib.rs`
- `crates/tyrus_codegen/src/decorators/`
- `.claude/plan/ultraplan-decorator-registry.md` (full Caminho C plan)
- Memory: `feedback_compiler_fundamentals.md`, `feedback_architecture_principles.md`
