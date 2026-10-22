# Tyrus Architecture

Tyrus is structured as a multi-stage compilation pipeline, designed for modularity and extensibility. This document describes the flow from TypeScript source to a final, runnable Rust project.

## The Compilation Pipeline

### 1. Parsing (`tyrus_parser`)

The entry point uses the `swc_ecma_parser` to ingest TypeScript source files.

- **Input:** `.ts` source code.
- **Output:** Abstract Syntax Tree (`swc_ecma_ast::Program`).
- **Responsibility:** Ensure the input is syntactically valid TypeScript.

### 2. Semantic Analysis (`tyrus_analyzer`)

This stage validates the AST against the **Oxidizable Standard**.

- **Input:** AST + source code + file name.
- **Lint rules (8):** Bans `var`, `any`, `eval`, `for-in`, `delete`, `with`, labeled statements. (Note: `try-catch` was unblocked in Phase 6.1)
- **Unsupported API detection (11):** Blocks `document`, `window`, `navigator`, `localStorage`, `sessionStorage`, `XMLHttpRequest`, `require`, `setTimeout`, `setInterval`, `clearTimeout`, `clearInterval`.
- **Visitors:** `LintVisitor` (8 rules) + `DecoratorVisitor` (NestJS metadata) + `UnsupportedApiVisitor` (blocked APIs).
- **Output:** `AnalysisResult { errors, diagnostics, graph }` — errors are `TyrusError`, diagnostics are structured `Diagnostic` with severity/code/span/suggestion.
- **Reports:** `format_pretty()` for colored terminal output, `format_json()` for tooling integration.

### 3. Orchestration (`tyrus_orchestrator`)

The coordinator of the full pipeline, decomposed into focused modules:

- **`lib.rs`** — Public API: `check()`, `build()`, `build_project()`, `build_simple_project()`
- **`gate.rs`** — Analyzer gate shared by every emission path: renders findings on stderr and refuses on hard lint errors before any write (#188)
- **`pipeline.rs`** — Core multi-file build orchestration (walk/parse, analyze, DI graph, transpile, mod.rs)
- **`scaffold.rs`** — Project scaffolding: `generate_main_rs()`, `generate_cargo_toml()`, `generate_mod_rs()`
- **`format.rs`** — Code formatting and `AppError` code generation

Responsibilities:
- Manages multi-file resolution, project scoping, and Rust project structure generation.
- Resolves singleton patterns (NestJS Services) to `Arc<T>` / Axum `State` via `tyrus_di`.
- Uses `tyrus_di` to topologically sort dependencies and determine instantiation order.

### 4. Dependency Management (`tyrus_di`)

A dedicated crate for handling the application's dependency graph.

- **Input:** Module metadata and provider definitions.
- **Algorithm:** Topological sort via `petgraph`.
- **Output:** Ordered initialization list, separation of Modules vs Providers vs Controllers.

### 5. Code Generation (`tyrus_codegen`)

- **Responsibility:** Converts the validated AST to Rust `TokenStream` using `quote!` macros — never string concatenation.
- **Output:** Formatted Rust code written to `src/`.

---

## Crate Breakdown

| Crate               | Responsibility                                                        |
| :------------------ | :-------------------------------------------------------------------- |
| `tyrus_cli`         | CLI (clap). 4 commands: `check`/`build`/`compile`/`run`. Branded banner, progress pipeline, `--quiet`/`--json` flags. Installable globally via `cargo install --path crates/tyrus_cli`. |
| `tyrus_parser`      | Wraps SWC parser. Input: `.ts` file. Output: `swc_ecma_ast::Program`. |
| `tyrus_ast`         | Typed IR: `TyrusModule`, `TyrusExpr`, `TyrusStmt`, `TyrusDecl`, `TyrusType`. SWC→IR type lowering (`lower_type.rs`). |
| `tyrus_analyzer`    | `LintVisitor` (7 rules) + `DecoratorVisitor` + `UnsupportedApiVisitor` (11 APIs). Structured `Diagnostic` output with JSON/pretty formatters. Uses `tyrus_decorator_kinds::DecoratorKind::from_name` for decorator classification. |
| `tyrus_codegen`     | Core transpilation. `RustGenerator` visitor converts TS AST to Rust. Hosts the `decorators` module — trait-based registry replacing the previous scattered match-arm dispatch. |
| `tyrus_decorator_kinds` | **Single source of truth** for NestJS decorator name → `DecoratorKind` classification. Zero external dependencies; consumed by both the analyzer (DI graph extraction) and the codegen (handler dispatch). See [ADR 0007](architecture/decisions/0007-decorator-registry.md). |
| `tyrus_di`          | NestJS-style DI engine. Uses `petgraph::DiGraph` for topo sort.       |
| `tyrus_orchestrator`| Coordinates the full pipeline: parse → analyze → codegen.            |
| `tyrus_diagnostics` | `TyrusError` variants with `miette` for rich error reporting.         |
| `tyrus_common`      | Shared types (`FilePath` newtype), config, filesystem utilities.       |
| `tyrus_test_utils`  | Test helpers including `assert_rust_compiles()`.                      |

---

## Code Generation Module Structure (`tyrus_codegen`)

The `crates/tyrus_codegen/src/` tree is decomposed into focused, single-responsibility modules. The top-level split is `convert/` (AST → Rust translation) plus `decorators/` (registry-based NestJS decorator dispatch — see [ADR 0007](architecture/decisions/0007-decorator-registry.md)) plus `stdlib/` (built-in API mappings).

```
src/
├── lib.rs              — public entry point, ControllerMetadata, generate()
├── convert/
│   ├── mod.rs          — module declarations and re-exports
│   ├── interface.rs    — RustGenerator struct definition + Visit impl (pipeline entry point)
│   ├── helpers.rs      — shared utilities: to_snake_case, to_pascal_case, is_string_expr
│   ├── fn_decl.rs      — function declaration processing (process_fn_decl)
│   ├── module.rs       — module/import handling
│   ├── type_mapper.rs  — TypeScript → Rust type mapping (incl. Map/Set/Date)
│   ├── stmt/           — statement conversion
│   │   ├── mod.rs          — dispatcher + convert_stmt, convert_stmt_recursive
│   │   ├── var_decl.rs     — variable declarations (ident, object/array destructuring)
│   │   ├── loops.rs        — while, for-of, for, do-while  (for-in is analyzer-blocked)
│   │   ├── switch.rs       — switch → match
│   │   └── try_catch.rs    — try-catch → Result matching
│   ├── class/          — class → struct+impl
│   │   ├── mod.rs          — dispatcher + property conversion (decorator-driven, no name suffix heuristic)
│   │   ├── constructor.rs  — constructor transpilation + DI
│   │   ├── method.rs       — method transpilation; method/param decorators dispatched through registry
│   │   ├── routing.rs      — Axum router generation + map_status_code (static STATUS_CODES table)
│   │   ├── getter_setter.rs — get/set → method calls
│   │   └── mutation.rs     — self-mutation detection
│   └── expr/
│       ├── mod.rs      — expression dispatcher (convert_expr)
│       ├── binary.rs   — binary operators (convert_bin_expr)
│       ├── call.rs     — function/method calls, axios/fetch/array methods
│       ├── member.rs   — property access, mutex state (convert_member_expr)
│       ├── arrow.rs    — arrow functions → closures (convert_arrow_expr)
│       ├── literal.rs  — literals, object/array/template expressions (incl. object spread)
│       └── misc.rs     — assignments (`=`/`-=`/`*=`/`/=`/`%=`/`&=`/`|=`/`^=`/`<<=`/`>>=`), updates, optional chaining
├── decorators/         — Decorator registry (ADR 0007)
│   ├── mod.rs          — DecoratorRegistry, ClassDecoratorHandler/MethodDecoratorHandler/ParamDecoratorHandler traits, default_registry(), shared_registry()
│   ├── controller.rs   — @Controller("/path") handler
│   ├── use_guards.rs   — @UseGuards(...) handler
│   ├── http_method.rs  — @Get/@Post/@Put/@Delete/@Patch (one parameterized struct, 5 instances registered)
│   ├── http_code.rs    — @HttpCode(N) handler
│   └── params.rs       — @Body, @Param, @Query handlers (one file, 3 structs)
└── stdlib/
    ├── mod.rs, console.rs, array.rs, string.rs, math.rs,
    └── json.rs, object.rs, map_set.rs   (Map<K,V>→HashMap, Set<T>→HashSet)
```

### Decorator Dispatch Flow

```
class/method.rs::extract_method_decorators
    → decorators::shared_registry().apply_method_decorators(method, &mut ctx)
        → for each decorator on the method:
            → DecoratorKind::from_name(ident) classifies the decorator
            → registry.method_handler(kind) finds the handler
            → handler.apply(method, call, &mut ctx) populates MethodDecoratorContext
    → ctx.http_method (Option<DecoratorKind>) and ctx.http_code (Option<u16>) drive subsequent code emission
```

The same pattern applies to class-level (`apply_class_decorators` → `ClassDecoratorContext`) and param-level (`first_param_decorator_kind` + `param_handler.emit_extractor`) decorators. **No file in `convert/` matches on decorator names; all name-based decisions are concentrated in `tyrus_decorator_kinds::DecoratorKind::from_name`.**

### Key Transpilation Patterns

- **Types:** `string→String`, `number→f64`, `boolean→bool`, `Promise<T>→Result<T, AppError>`, `Record<K,V>→HashMap<K,V>`, `Map<K,V>→HashMap<K,V>`, `Set<T>→HashSet<T>`
- **Interfaces** → `#[derive(Serialize, Deserialize)] struct` with serde
- **String union types** (`type Status = "open" | "closed"`) → Rust enums
- **Array methods** (`.map`, `.filter`, `.forEach`, `.find`, `.some`, `.every`) → iterator chains with `.collect()`. Supports `(item, index)` callbacks via `.enumerate()`
- **String methods** (`.includes→.contains`, `.replace→.replacen`, `.split→.split().collect()`, etc.)
- **Classes** → structs with `impl` blocks. State fields use `Arc<Mutex<T>>` for interior mutability. Constructor-injected deps wrapped in `Arc<T>`. Controller detection is decorator-driven (presence of `@Controller(...)`), not by class-name suffix.
- **NestJS decorators** → Axum, dispatched through the `decorators` registry: `@Controller("/path")` → `router()` + `FromRequestParts`, `@Get/@Post/@Put/@Delete/@Patch` → `axum::routing::get`/`post`/etc, `@HttpCode(N)` → `(StatusCode, Json<T>)` tuple return, `@Body/@Param/@Query` → `axum::Json<T>`/`Path<T>`/`Query<T>` extractors, `@UseGuards(...)` → `axum::middleware::from_fn` layer, `@Injectable()`/`@Module()` → DI graph (analyzer-side).
- **Async/await** → `pub async fn` with tokio, `await` → `.await`
- **Object spread (`{...base, field: v}`)** → struct update syntax
- **Assignment operators (`=`/`-=`/`*=`/`/=`/`%=`/`&=`/`|=`/`^=`/`<<=`/`>>=`)** → equivalent Rust compound assignments
- **`Date.now()`** → `chrono::Utc::now().timestamp_millis()`

---

## Visitor Pattern

Four visitors traverse the SWC AST via `swc_ecma_visit::Visit`:

1. `LintVisitor` — rejects `var`, `any`, `eval`, `for-in`, `delete`, `with`, labeled (in `tyrus_analyzer`)
2. `DecoratorVisitor` — extracts `@Module`, `@Injectable`, `@Controller` metadata (in `tyrus_analyzer`); classifies decorator names via `tyrus_decorator_kinds::DecoratorKind::from_name`, never via raw string compare
3. `UnsupportedApiVisitor` — detects DOM, browser, timer, CommonJS APIs (in `tyrus_analyzer`)
4. `RustGenerator` — produces Rust token streams (entry point: `interface.rs`); delegates all NestJS decorator handling to the registry in `decorators/`

---

## Testing Architecture

Tests live in the `tests/` crate, organized by tier:

```
tests/
├── src/
│   ├── cli.rs         — CLI integration tests (help, version, check, build, flags)
│   ├── unit/          — unit tests for isolated functions
│   ├── snapshot/      — insta snapshot tests for codegen output
│   ├── compilation/   — compilation tests: generate Rust, run rustc
│   └── equivalence/   — semantic equivalence: TS and Rust produce identical output
└── fixtures/
    ├── tier1/         — core language features
    ├── tier2/         — advanced type system
    ├── tier3/         — ecosystem & async
    └── tier4/         — framework integration (NestJS → Axum)
```

317 tests (measured 2026-09-06, `cargo nextest run --workspace`; 1 skipped) plus 4 doc-tests. Integration layers: equivalence (102), unit (66), snapshot (21), CLI (13, plus 8 analyzer-gate contract tests in `cli_gate.rs`), compilation (8), tier4 E2E (5), trybuild (1). Crate-level unit tests: codegen (30), analyzer (27), common (9), ast (8), decorator_kinds (6), cli (5), orchestrator (3, the formatter contract), diagnostics (3), test_utils (2). The HTTP-equivalence E2E test (`tier4_tests::test_http_equivalence_rust_server`) transpiles the reference NestJS project, compiles the generated Rust, starts both servers, and compares responses byte-for-byte.

---

## Strict Quality Rules

The project enforces strict correctness via:

- **`.cargo/config.toml`:** `-Dwarnings` plus `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::todo`, `clippy::unimplemented` as hard errors.
- **`clippy.toml`:** Cognitive complexity threshold 15, function lines threshold 50, max 5 parameters.
- **`deny.toml`:** Dependency license and security audit via `cargo-deny` (see [ADR 0011](architecture/decisions/0011-supply-chain-hygiene.md)).
- **CI:** GitHub Actions with `actions/checkout@v4`, `Swatinem/rust-cache@v2`, `cargo nextest`, and end-to-end demo compilation verification.

---

## Architectural Decision Records

Numbered ADRs under [`docs/architecture/decisions/`](architecture/decisions/) are the binding record of architectural choices. Most-recent-first:

- [ADR 0019 — Oracle and Harness Tooling](architecture/decisions/0019-oracle-and-harness-tooling.md) — Proposto: pinned Node tuple, Python `-I` stdlib harnesses, `cargo-mutants`, `jsonschema` dev-dependency and its `Cargo.lock` bumps, npm/Docker pinning.
- [ADR 0015 — Tooling Evaluated and Rejected](architecture/decisions/0015-rejected-tooling.md) — Ferrocene, Miri, sanitizers, cargo-vet/crev, restriction-as-group: why not, and what would reopen each.
- [ADR 0014 — Development Flow Rules (F1–F10)](architecture/decisions/0014-development-flow-rules.md) — process standard companion to the Power of Ten; see [DEVELOPMENT_FLOW.md](standards/DEVELOPMENT_FLOW.md).
- [ADR 0013 — Power of Ten v2](architecture/decisions/0013-power-of-ten-v2.md) — R13 (forbid unsafe), R14 (stable error codes), amendments to R4/R5/R6/R9/R12, Consortium traceability annex.
- [ADR 0012 — Array Method Dispatch — Ownership](architecture/decisions/0012-array-method-dispatch-split.md) — boundary between IR-context array calls (`call_array.rs`) and pure Vec ops (`stdlib/array.rs`).
- [ADR 0011 — Supply-Chain Hygiene Policy](architecture/decisions/0011-supply-chain-hygiene.md) — license allowlist, advisory ignore list, dependabot grouping, CODEOWNERS (PR #126 backfill).
- [ADR 0010 — Formatter Contract](architecture/decisions/0010-formatter-contract.md) — idempotence, error-propagation, no-bypass guarantees for `tyrus_orchestrator::format` (PR #142 backfill).
- [ADR 0009 — Mutex Re-entrance Protocol](architecture/decisions/0009-mutex-re-entrance-protocol.md) — block-scoped read + read-then-write split for `@Injectable` state fields (PR #141 backfill).
- [ADR 0008 — Tyrus Strict Rules (Power of Ten)](architecture/decisions/0008-tyrus-strict-rules.md) — adoption of the 12-rule contribution standard.
- [ADR 0007 — Decorator Registry](architecture/decisions/0007-decorator-registry.md) — trait-based handler registry replacing scattered string-compare match arms.
- [ADR 0006 — Safe Transpilation Infrastructure](architecture/decisions/0006-safe-transpilation-infrastructure.md)
- [ADR 0005 — Codegen Module Decomposition](architecture/decisions/0005-codegen-module-decomposition.md)
- [ADR 0004 — NestJS to Axum Mapping](architecture/decisions/0004-nestjs-to-axum.md)
- [ADR 0003 — Generics Mapping](architecture/decisions/0003-generics-mapping.md)
- [ADR 0002 — Async Transpilation](architecture/decisions/0002-async-transpilation.md)
- [ADR 0001 — Stack and Monorepo](architecture/decisions/0001-stack-and-monorepo.md)
- [ADR 0000 — Use Markdown ADR](architecture/decisions/0000-use-markdown-adr.md)

---

## Tech Stack

- **Source Language:** TypeScript (via SWC)
- **Target Language:** Rust (1.75+)
- **Macro Engine:** `quote`, `proc_macro2`
- **Internal Web Engine:** `axum` 0.7 (for NestJS mappings)
- **Test Runner:** `cargo nextest` + `insta` (snapshot testing)
