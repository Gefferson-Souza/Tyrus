# 🗺️ Tyrus Roadmap

> **This file is the canonical work queue.** The autonomous delivery loop
> (`.claude/workflows/tyrus-feature-delivery.md`, entry point `/deliver`) follows it in
> order: **open CRITICAL bugs → Now → Next → Later**. Lines link the issue that is their
> system of record where one exists; work without an issue gets one before any branch (F1).
> Re-prioritizing this file is an owner decision. Status lines state measured facts with
> their date — never aspirations (ADR 0013).

## Current state (measured 2026-09-06)

- **321 tests green** (317 via `cargo nextest` + 4 doc-tests) · coverage 78.3% (measured 2026-08-30; target 80%, #163)
- `tyrus build`/`compile`/`run` run the analyzer and refuse forbidden TS before writing anything (#188)
- 11 crates + `tests` member · Power of Ten v2 (14 rules, ADR 0013) · 9 local gates = CI (R9)
- Multi-module NestJS projects transpile, compile, and serve HTTP responses equivalent to Node
- Versioned agent harness: `.claude/` (settings, hooks, rules, agents, skills, workflows)

## Now — UAT criticals (dogfooding the compiler as a user would)

1. **[#191](https://github.com/Gefferson-Souza/Tyrus/issues/191)** HIGH — `build` returns exit 0 on structurally incomplete output (no `fn main`, dropped statements). Same defect class: [#256](https://github.com/Gefferson-Souza/Tyrus/issues/256) (top-level `const` + declared `main()` drops the const).
2. **[#265](https://github.com/Gefferson-Souza/Tyrus/issues/265)** HIGH — unknown decorators are dropped (analyzer `continue`, codegen ignores). Owner rule: decorators are generic; only `@Module`/`@Injectable`/`@Controller` + routing/params are semantic. ADR-shaped (generic form to be chosen).
3. **[#266](https://github.com/Gefferson-Souza/Tyrus/issues/266)** — "Oxidizable NestJS Profile" spec: the finite semantic-override list, everything else generic. Docs only; owner reviews scope.
4. **[#189](https://github.com/Gefferson-Souza/Tyrus/issues/189)** HIGH — `tyrus run` looks for a `tyrus_app` binary but project scaffolds emit `server`.
5. **[#190](https://github.com/Gefferson-Souza/Tyrus/issues/190)** HIGH — `@Injectable()` constructor body initializers silently dropped in `new_di()`.
6. **[#192](https://github.com/Gefferson-Souza/Tyrus/issues/192)** HIGH — no file-extension validation (`.js`/`.txt` accepted, Oxidizable Standard bypassable).
7. **[#194](https://github.com/Gefferson-Souza/Tyrus/issues/194)** HIGH — untyped parameters become `serde_json::Value` instead of an analyzer error.
8. **[#255](https://github.com/Gefferson-Souza/Tyrus/issues/255)** — `declare var` in `.d.ts` trips E1001, so `build <dir>` refuses projects with ambient declarations (surfaced by the #188 gate).

## Next — elite testing (standardization campaign, phase 4)

- **[#229](https://github.com/Gefferson-Souza/Tyrus/issues/229)** — wire declared-but-missing enforcement: R7/R8 grep gates, nightly cron (audit + fuzz), gate-parity check.
- **[#217](https://github.com/Gefferson-Souza/Tyrus/issues/217)** — `cargo-mutants --in-diff` on PRs as the R5 coverage-emptiness check.
- **[#154](https://github.com/Gefferson-Souza/Tyrus/issues/154)** — 10 critical equivalence tests (HTTP, async/await, optional chaining, …).
- **[#163](https://github.com/Gefferson-Souza/Tyrus/issues/163)** — ramp coverage threshold 73% → 80% (currently 78.3%).

## Later — features, refactors, release

- **Decorator registry PR #5:** implement `@Headers` purely via `decorators/params.rs` + one `register_param` line — the empirical proof that new decorators touch zero legacy files (ADR 0007).
- **Phase 9 (NestJS validation):** class-validator → `garde`, `ParseIntPipe`, `ValidationPipe`.
- **[#157](https://github.com/Gefferson-Souza/Tyrus/issues/157)** — `BuiltinTypeRegistry` consolidating Map/Set/Promise/Date/Array/Record dispatch (R8).
- **[#223](https://github.com/Gefferson-Souza/Tyrus/issues/223)** — range guard for numeric-enum `i32` casts.
- **[#230](https://github.com/Gefferson-Souza/Tyrus/issues/230)** — lint-attribute cleanup (duplicate denies, blanket allow).
- **[#158](https://github.com/Gefferson-Souza/Tyrus/issues/158)** — remaining 17 functions over 50 lines.
- **[#143](https://github.com/Gefferson-Souza/Tyrus/issues/143)** — `Arc<Mutex<primitive>>` → `Arc<Atomic*>` for state fields.
- **[#128](https://github.com/Gefferson-Souza/Tyrus/issues/128)** — resolve the two accepted RUSTSEC advisories.
- **[#184](https://github.com/Gefferson-Souza/Tyrus/issues/184)** — v0.1.0 release *(owner decision — the loop must not act on this)*.

## Research horizon (unscheduled)

Class inheritance via traits/composition (`enum_dispatch`) · cross-function type inference (integer vs `f64`) · `Date` → `chrono` · IR optimization
passes · formal verification of semantic preservation.

---

## Completed milestones (history — details live in git/CHANGELOG)

- **M1–M8 · Foundation → NestJS (2026 Q1):** SWC parser integration, core transpilation
  (arithmetic/logic/control flow), Oxidizable Standard analyzer, interfaces→structs+serde,
  generics, async→tokio, decorators→Axum, `tyrus_di` DI graph, unified `main.rs` scaffold.
- **M9–M12 · Hardening + decomposition:** panic-free codebase, module decomposition
  (`func.rs`/`class.rs` → focused modules), tiered test architecture
  (unit/snapshot/compilation/equivalence).
- **M13 · Semantic equivalence:** `assert_output_equivalent()` running Node vs compiled
  Rust; control-flow expansion (for/do-while/switch); dozens of codegen bug fixes.
- **Sprint 1:** compound assignment ops, `Map`/`Set`, `this.method()`, object shorthand,
  `Date.now()`, object spread.
- **Caminho C (ADR 0007):** trait-based `DecoratorRegistry` replaced scattered match-arm
  dispatch; legacy heuristics deleted.
- **Standardization campaign (2026-08-05, 12 PRs):** Power of Ten v2 (14 rules) +
  Development Flow (F1–F10) adopted as binding spec, `[workspace.lints]` with pedantic +
  restriction lints, 9-gate local/CI parity, stable `TYRUS-EXXXX` error codes (R14),
  `missing_docs` on boundary crates, versioned Claude Code harness.
