# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

**Tyrus** (TypeRust) is an academic TypeScript-to-Rust transpiler. It converts a strict subset of TypeScript ("Oxidizable Standard") into memory-safe Rust code. No `any`, `var`, or `eval` allowed. Cargo workspace with 13 crates under `crates/` plus the `tests` member.

## STRICT RULES (NEVER VIOLATE)

**Canonical authority:** [`docs/standards/POWER_OF_TEN.md`](docs/standards/POWER_OF_TEN.md) — 14 rules adapted from NASA/JPL Power of Ten. Adopted via [ADR 0008](docs/architecture/decisions/0008-tyrus-strict-rules.md), amended by [ADR 0013](docs/architecture/decisions/0013-power-of-ten-v2.md). Process rules live in [`docs/standards/DEVELOPMENT_FLOW.md`](docs/standards/DEVELOPMENT_FLOW.md) (F1–F10, ADR 0014). The table below is a quick reference; the docs are the binding spec.

| # | Rule | Severity | Enforced by |
|---|------|----------|-------------|
| 1 | Bounded control flow (no unbounded recursion) | HIGH | review checklist (fuzz: #229) |
| 2 | Bounded loops (O(n) in input size) | HIGH | review checklist (fuzz: #229) |
| 3 | Minimal scope for bindings + struct fields | MEDIUM | `clippy::needless_pass_by_value` |
| 4 | Functions ≤ 50 lines, files ≤ 400, ≤ 5 params, ≤ 4 nesting | CRITICAL | `clippy::too_many_lines/arguments` via `[workspace.lints]` + `gates.sh filesize` |
| 5 | Equivalence-test density (every codegen change → test) | CRITICAL | PR template + `cargo nextest` + `cargo-llvm-cov` |
| 6 | Total error handling (no `unwrap`/`expect`/`panic`/`todo`, no panicking indexing) | CRITICAL | `[workspace.lints]` clippy lints |
| 7 | `quote!` macros only — never string-concat codegen | CRITICAL | review (grep gate: #229) |
| 8 | Two-layer architecture (generic AST + semantic registry) | CRITICAL | review + ADR 0007 precedent |
| 9 | Local-first validation parity (hook == CI via `scripts/gates.sh`) | CRITICAL | shared script |
| 10 | ADR for architectural decisions | HIGH | PR template |
| 11 | One branch = one concern + Conventional Commits | HIGH | PR title regex |
| 12 | Warnings-clean + daily audited (`--all-targets`, `--locked`, `cargo deny`, `cargo audit`, `cargo machete`, MSRV job) | CRITICAL | CI workflow |
| 13 | `#![forbid(unsafe_code)]` in every crate | CRITICAL | crate-root attribute + `gates.sh unsafe` |
| 14 | Stable error codes `TYRUS-EXXXX` + category per `TyrusError` variant | HIGH | exhaustiveness/uniqueness tests + JSON contract |

## Autonomous Delivery

All work follows the versioned delivery loop — phases, agent teams per phase, gates, and
stop conditions live in [`.claude/workflows/tyrus-feature-delivery.md`](.claude/workflows/tyrus-feature-delivery.md);
the always-loaded invariants (RULE ZERO observed equivalence, never assume, focus, autonomy
boundary) are [`.claude/rules/delivery.md`](.claude/rules/delivery.md). Entry point: `/deliver`.
The canonical prioritized work queue is [`ROADMAP.md`](ROADMAP.md) (CRITICAL bugs → Now → Next → Later).

## Build & Dev Commands

```bash
# Build
cargo build --workspace

# Tests (use nextest for speed)
cargo nextest run --workspace          # All tests (parallel)
cargo test -p integration_tests        # Integration tests only
cargo test -p tyrus_codegen            # Single crate

# Lint & Format (must pass CI)
cargo fmt -- --check
cargo clippy --workspace               # warnings = "deny" via [workspace.lints]

# Run the compiler
cargo run --bin tyrus -- check <file.ts>              # Analyze for compatibility
cargo run --bin tyrus -- check --json <file.ts>       # JSON diagnostic output
cargo run --bin tyrus -- build <dir>/src --output <dir>/output   # Transpile to Rust
cargo run --bin tyrus -- compile <dir>/src --output <dir>/output # Transpile + cargo build
cargo run --bin tyrus -- run <dir>/src --output <dir>/output     # Transpile + build + execute
cargo run --bin tyrus -- --quiet <command>             # Suppress banner

# Snapshots
cargo insta review                     # Review snapshot changes

# Benchmarks
cargo bench --bench transpiler         # Transpilation speed (criterion)
cargo bench --bench runtime_comparison # Node.js vs Rust runtime comparison

# Oracle pins (required by `cargo nextest run --workspace` and gates.sh test/coverage)
export TYRUS_NODE_REFERENCE=/abs/path/to/node-v26.11.1/bin/node   # tuple in oracle/oracle.lock
export TYRUS_NODE_SEED=/abs/path/to/node-v22.21.1/bin/node
cargo run -p tyrus_oracle --bin tyrus-oracle -- lock verify --role all

# Git hooks (run once after cloning)
./scripts/setup-hooks.sh               # Install pre-commit (fmt + clippy)
```

## Architecture

```
.ts input
  → tyrus_parser       (SWC parsing → swc_ecma_ast::Program)
  → tyrus_ast           (TyrusModule IR — typed intermediate representation)
  → tyrus_analyzer     (LintVisitor + DecoratorVisitor + UnsupportedApiVisitor)
  → tyrus_di           (petgraph topological sort for DI)
  → tyrus_orchestrator (multi-file coordination, CheckResult)
  → tyrus_codegen      (quote! → proc_macro2::TokenStream)
  → formatted .rs output
```

### Crate Map

| Crate | Lines | Role |
|-------|-------|------|
| `tyrus_cli` | ~430 | CLI (clap). 4 commands: `check`/`build`/`compile`/`run`. Branded output, progress pipeline. |
| `tyrus_parser` | ~55 | Wraps SWC parser. `.ts` → `Program` |
| `tyrus_ast` | ~730 | Typed IR: `TyrusModule`/`TyrusExpr`/`TyrusStmt`/`TyrusDecl`. SWC→IR lowering. |
| `tyrus_analyzer` | ~580 | `LintVisitor` (7 rules) + `DecoratorVisitor` + `UnsupportedApiVisitor` + JSON reports. Uses `tyrus_decorator_kinds` for name → kind classification. |
| `tyrus_codegen` | ~6000 | **Core.** `RustGenerator` → TokenStream. Decomposed: `helpers/stmt/fn_decl/expr/*/class/*/decorators/*/stdlib/*`. |
| `tyrus_decorator_kinds` | ~225 | **Single source of truth** for NestJS decorator name → `DecoratorKind` classification. Zero deps; shared by analyzer + codegen. See `docs/architecture/decisions/0007-decorator-registry.md`. |
| `tyrus_di` | ~200 | DI graph (petgraph). Topological sort. |
| `tyrus_orchestrator` | ~700 | Pipeline coordination. Split: `lib/gate/pipeline/scaffold/format`; `gate.rs` runs the analyzer before every emission path (#188). |
| `tyrus_diagnostics` | ~69 | `TyrusError` + miette |
| `tyrus_common` | ~70 | `FilePath`, `to_snake_case()`, config |
| `tyrus_test_utils` | ~195 | `assert_rust_compiles()`, `compile_and_run_rust()`, `run_node()` |
| `tyrus_oracle` | ~970 | Oracle root of trust: pinned Node tuples (`oracle/oracle.lock`), closed `Verdict` algebra with `Shares`, `tyrus-oracle lock verify`. Errors `TYRUS-E5xxx`. Depends on no legacy crate. |
| `tyrus_facts` | ~760 | Front-end facts (ST1): serde types for `facts.json`, `closure.lock`, `partition.lock` (schemas in `schemas/`), canonical JSON, schema-version (`TYRUS-E7102`) and hash (`TYRUS-E7101`) validation. Depends on no legacy or runtime crate. |

### Codegen Module Structure (Current)

```
crates/tyrus_codegen/src/
├── convert/
│   ├── mod.rs            # Module declarations and re-exports
│   ├── interface.rs      # RustGenerator struct + Visit impl (entry point)
│   ├── helpers.rs        # to_snake_case, to_pascal_case, is_string_expr, is_primitive_type
│   ├── fn_decl.rs        # Function declaration transpilation
│   ├── type_mapper.rs    # TS→Rust type mapping (map_type_core)
│   ├── module.rs         # Module/import handling
│   ├── stmt/             # Statement conversion
│   │   ├── mod.rs         # Dispatcher + convert_stmt, convert_stmt_recursive
│   │   ├── var_decl.rs    # Variable declarations (ident, object/array destructuring)
│   │   ├── loops.rs       # while, for-of, for, do-while  (for-in is analyzer-blocked)
│   │   ├── switch.rs      # switch → match
│   │   └── try_catch.rs   # try-catch → Result matching
│   ├── class/            # Class → struct+impl
│   │   ├── mod.rs         # Class dispatcher + property conversion (decorator-driven)
│   │   ├── constructor.rs # Constructor transpilation + DI
│   │   ├── method.rs      # Method transpilation; param/method decorators via registry
│   │   ├── routing.rs     # Axum router generation + @UseGuards middleware + map_status_code
│   │   ├── getter_setter.rs # get/set → method calls
│   │   └── mutation.rs    # Self-mutation detection
│   └── expr/             # Expression conversion
│       ├── mod.rs         # Expression dispatcher (convert_expr)
│       ├── binary.rs      # Binary operators (+, -, *, /, ==, etc.)
│       ├── call.rs        # Function/method calls, axios/fetch/array methods
│       ├── member.rs      # Property access, mutex state (convert_member_expr)
│       ├── arrow.rs       # Arrow functions → closures
│       ├── literal.rs     # Object/array/template literals (incl. object spread)
│       └── misc.rs        # Assignments (=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=), updates, optional chaining
├── decorators/           # Decorator registry (ADR 0007)
│   ├── mod.rs             # DecoratorRegistry + Class/Method/ParamDecoratorHandler traits + default_registry()
│   ├── controller.rs      # @Controller("/path") handler
│   ├── use_guards.rs      # @UseGuards(...) handler
│   ├── http_method.rs     # @Get/@Post/@Put/@Delete/@Patch (one parameterized struct, 5 instances)
│   ├── http_code.rs       # @HttpCode(N) handler
│   └── params.rs          # @Body, @Param, @Query handlers (one file, 3 structs)
└── stdlib/               # Standard library mappings
    ├── mod.rs, console.rs, array.rs, string.rs, math.rs,
    ├── json.rs, object.rs, map_set.rs   (Map<K,V>→HashMap, Set<T>→HashSet)
```

## Type Mappings (TS → Rust)

| TypeScript | Rust |
|------------|------|
| `string` | `String` |
| `number` | `f64` |
| `boolean` | `bool` |
| `void` | `()` |
| `T[]` / `Array<T>` | `Vec<T>` |
| `Promise<T>` | `Result<T, AppError>` |
| `Record<K,V>` | `HashMap<K,V>` |
| `Map<K,V>` | `HashMap<K,V>` (`new Map()` → `HashMap::new()`) |
| `Set<T>` | `HashSet<T>` (`new Set()` → `HashSet::new()`) |
| `Date` | `String` (TODO: chrono); `Date.now()` → `chrono::Utc::now().timestamp_millis()` |
| `T \| undefined` | `Option<T>` |
| `interface` | `#[derive(Serialize, Deserialize)] struct` |
| `type Status = "a" \| "b"` | `enum Status { A, B }` |
| `class` | `struct + impl` |

## Testing Architecture

```
tests/
├── src/
│   ├── cli.rs         # CLI integration: help, version, check, build, --json, --quiet
│   ├── unit/          # FAST (<5s): Test codegen functions directly
│   ├── snapshot/      # MEDIUM (<10s): Full transpilation → insta snapshots
│   ├── compilation/   # SLOW (<60s): Batch cargo check per tier
│   └── equivalence/   # SEMANTIC: Run TS (Node) + Rust, compare stdout
│       ├── basic.rs         # Arithmetic, control flow, unary ops
│       ├── strings.rs       # String methods equivalence
│       ├── arrays.rs        # Array methods equivalence
│       ├── console.rs       # console.log formatting
│       ├── control_flow.rs  # if/else, for-of, do-while, switch, ternary
│       ├── error_handling.rs # try-catch, throw
│       ├── inheritance.rs   # extends, super(), method override
│       ├── math.rs          # Math functions and constants
│       ├── spread.rs        # Spread operator, rest params
│       ├── top_level.rs     # Top-level const/let/expressions
│       └── type_features.rs # Type assertions, numeric enums
├── fixtures/
│   ├── tier1/         # Basic: variables, math, functions, control flow
│   ├── tier2/         # Intermediate: interfaces, classes, async
│   ├── tier3/         # Advanced: generics, optional chaining, destructuring
│   └── tier4/         # NestJS: @Injectable, @Controller, full project
```

## Conventions

- **Commits:** `<type>: <description>` — Types: feat, fix, refactor, test, chore, docs, perf, ci
- **Branches:** `feat/`, `fix/`, `chore/`, `refactor/` prefixes
- **Error handling:** Always `Result<T, TyrusError>`. Never unwrap/expect/panic.
- **Generated code errors:** Use `compile_error!("Tyrus: ...")` instead of `todo!()`
- **Immutability:** Prefer `&self`, use `&mut self` only when mutation is detected
- **Code gen:** `quote!` macros only. Never string concatenation.

## Current Work

[`ROADMAP.md`](ROADMAP.md) is the canonical prioritized queue (CRITICAL bugs → Now → Next
→ Later) and carries the measured project state (test count, coverage). Do not track work
status here — this file describes structure and rules, the roadmap describes work.

**E2E reference:** `test_http_equivalence_rust_server` — transpile → compile → start
server → verify HTTP responses match the NestJS original.
