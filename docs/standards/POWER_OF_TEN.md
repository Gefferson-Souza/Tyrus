# Tyrus Power of Ten

> **Strict rules for Tyrus development.** Inspired by the NASA/JPL "Power of Ten Rules for Developing Safety-Critical Code" (Holzmann, 2006), adapted to Rust + transpiler context.
>
> **Status:** Accepted (ADR 0008 / 2026-05-03). Amended to 14 rules by ADR 0013 (2026-08-04). Binding for all contributions to `main`.
>
> **Scope (proposed by ADR 0016, effective when accepted):** binding for the **legacy crates** only (the prototypes listed in `gates.toml [legacy] crates`, once that file exists). New crates follow the [Constitution](CONSTITUTION.md). R13 is amended by ADR 0016: a closed allowlist of crates may use `unsafe`.

---

## Why these rules

Tyrus turns adversarial TypeScript input into Rust source code. Two failure modes are catastrophic:

1. **Silent miscompiles** — generated Rust looks plausible but means something different from the source TS. Caught only by semantic equivalence tests.
2. **Architectural drift** — string-compare matches scattered across files; one-off hardcoded names that grow linearly per supported feature, ending in unmaintainable gambiarra.

These rules are not bureaucracy. Each one closes a specific failure mode this project has already lived through (see "Origins" notes per rule).

The rules are **enforced**, not aspirational: clippy lints, CI gates, pre-commit hooks, and review checklists. **A rule with no enforcement is just a wish.**

---

## The 14 rules

### Rule 1 — Bounded Control Flow

**Rule.** No `goto`-equivalent constructs. Recursion is permitted only when (a) bounded by a structural invariant of the input AST (e.g., expression depth) **or** (b) explicitly justified in a code comment citing the bound. Other recursion must be lifted to an iterative loop with an explicit work queue.

**Why.** SWC ASTs from adversarial `.ts` input can be deeply nested. Unbounded recursion in codegen = stack overflow on user input. Iterative lowering is also easier to instrument and pause for diagnostics.

**Enforce.** `clippy::only_used_in_recursion` (pedantic, active via `[workspace.lints]`); review checklist item "recursion bound documented?". A nightly fuzz target with a 10k-deep nested expression fixture is planned, not yet wired — tracked in #229.

**Severity.** HIGH (review-blocking).

---

### Rule 2 — Bounded Loops

**Rule.** Every loop must have a statically reasoning-able upper bound: it iterates over a finite collection, an integer range, or terminates on a condition derived from input size. `loop {}` without an explicit `break` guarded by such a bound is forbidden in production crates.

**Why.** Transpiler must be O(n) in input size. Unbounded loops are how compilers hang on malformed input.

**Enforce.** Code review checklist; `clippy::infinite_loop` (where stable). The `cargo fuzz run transpile` nightly target is planned, not yet wired — tracked in #229.

**Severity.** HIGH.

---

### Rule 3 — Minimal Scope

**Rule.** Bindings (`let`, `const`, struct fields, mutable state on `RustGenerator`) live in the narrowest scope that satisfies their use. New fields on long-lived structs require a justification line in the PR description. Prefer parameter passing or `RefCell` over global/struct-wide state.

**Why.** `RustGenerator` mutable fields (`string_vars`, `current_class_state_fields`, etc.) are correctness-critical. Tighter scope = fewer hidden coupling bugs across codegen modules.

**Enforce.** `clippy::needless_pass_by_value`, `clippy::redundant_field_names`; review checklist; `pub(crate)` audit.

**Severity.** MEDIUM (warning + follow-up issue).

---

### Rule 4 — Function Size and Shape

**Rule.** Functions ≤ 50 lines. ≤ 5 parameters. ≤ 4 nesting levels. Files ≤ 400 lines (800 absolute hard cap). No function may both mutate `self` **and** emit a `TokenStream` longer than 30 lines without being split.

*Escape hatch (ADR 0013).* When splitting a `quote!`-heavy emission function would fragment one coherent template into artificial pieces, a targeted `#[expect(clippy::too_many_lines, reason = "…")]` with a written justification is permitted. `#[expect]` — never `#[allow]` — so the compiler flags the exception the day it stops being needed. The total count of such exceptions is reported per PR; growth without justification is a review finding.

**Why.** Every file split during Phases 1-8 surfaced bugs. Shape constraints are objective, machine-checkable code-review currency.

**Enforce.** `clippy::too_many_lines`, `clippy::too_many_arguments` — active via `[workspace.lints]` (pedantic group, thresholds in `clippy.toml`); the file ceiling via the `filesize` gate in `scripts/gates.sh` (+ 1:1 CI step). `clippy::cognitive_complexity` is nursery-only; its threshold sits pre-configured in `clippy.toml` awaiting lint stabilization.

**Severity.** CRITICAL (CI-blocking).

**Origin.** PR #109 — `convert_assign_expr` reached 88 lines, violated the limit, blocked PR1 of #129 until refactored (PR #139).

---

### Rule 5 — Test Equivalence Density

**Rule.** Every codegen change that affects emitted Rust ships with at least one *semantic equivalence test* in `tests/src/equivalence/` — running TS through Node and the generated Rust binary, comparing stdout byte-for-byte. Pure refactors are exempt only when accompanied by a green snapshot diff (`cargo insta review`).

**Why.** Replaces NASA's "≥ 2 asserts/function" with the Rust-idiomatic, transpiler-specific equivalent: behavior parity is the only assertion that matters here. A passing unit test on a mismatched-semantics codegen is worse than no test.

**Enforce.** PR template checkbox "equivalence test added or N/A justified"; `cargo nextest run -p integration_tests` in CI; line-coverage gate via `cargo-llvm-cov` at the current enforced threshold (73%, ramping +1 pp per sprint to the 80% target — tracked in issue #163; the gate value in `scripts/gates.sh`/CI is the source of truth). Coverage emptiness is checked with `cargo mutants --in-diff` on PR diffs: lines that are executed but never asserted count as uncovered for review purposes (adoption tracked in issue #217 — until it lands, this check is a review-time expectation, not a CI gate).

**Severity.** CRITICAL.

**Origin.** Memory rule `feedback_semantic_equivalence`. Phase 5 (#107).

---

### Rule 6 — Total Error Handling

**Rule.** No `.unwrap()`, `.expect()`, `panic!()`, `todo!()`, `unimplemented!()` in production code. No panicking indexing either: `x[i]` on slices/`Vec`s, string slicing `&s[a..b]`, and `unwrap()` inside functions that already return `Result` are forbidden — use `.get()`, `.get(a..b)`, and `?` respectively. Fallible operations return `Result<T, TyrusError>`. Generated Rust uses `compile_error!("Tyrus: …")` for unsupported constructs — never `todo!()`. Test code (in `#[cfg(test)] mod` or under `tests/`) may use `.expect("descriptive msg")`, but must be explicitly marked with `#[allow(clippy::expect_used)]` at the test-module level.

**Why.** Codified project rule. A `.unwrap()` in a transpiler is a denial-of-service vector when the input triggers it.

**Enforce.** `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::todo`, `clippy::unimplemented`, plus the ADR 0013 additions `indexing_slicing`, `string_slice`, `unwrap_in_result` — all warn-level in `[workspace.lints]` with `warnings = "deny"` (#215), verified by `cargo clippy --workspace --all-targets --locked -- -D warnings`.

**Severity.** CRITICAL.

**Origin.** `.cargo/config.toml`. Project rule since founding.

---

### Rule 7 — Macros, Not Strings

**Rule.** All Rust code generation goes through `quote!` / `syn` / `proc_macro2::TokenStream`. String concatenation, `format!`, or `write!` of Rust source is forbidden anywhere under `crates/tyrus_codegen/src/convert/`. Templating is allowed only for non-Rust artifacts (e.g., `Cargo.toml` scaffolding) and for hard-coded `&'static str` blocks of pre-validated source (e.g., `get_app_error_code()` in `tyrus_orchestrator::format`).

**Why.** `quote!` enforces hygiene and balanced syntax. String concat silently produces invalid Rust that `prettyplease::unparse` either rejects (good) or accepts in a malformed shape (worse).

**Enforce.** Review checklist; `tyrus_orchestrator::format` is the only allowed string-Rust touchpoint. Negative grep gate (`! rg -n 'format!\("(fn |pub )' crates/tyrus_codegen/`) planned, not yet wired — tracked in #229.

**Severity.** CRITICAL.

---

### Rule 8 — Two-Layer Compiler Architecture

**Rule.** Translation logic splits into exactly two layers: (a) **Generic structural handler** dispatching by AST node *type*, and (b) **Semantic registry** keyed by *name* (decorators, stdlib calls, framework symbols). Adding a new TS framework symbol must touch ≤ 1 handler file + 1 registration line + 1 enum variant. Scattered `match name { "X" => …, "Y" => … }` arms across multiple files are forbidden.

**Why.** ADR 0007 (decorator registry) proved the win — decorator additions dropped from 4 file edits to 1 handler + 1 line. Any regression to scattered string compares is a known-bad pattern.

**Enforce.** Review checklist; ADR required for new registries (per Rule 10). Negative grep gate (`! rg 'class_name\.ends_with\("Controller"\)' crates/`) planned, not yet wired — tracked in #229.

**Severity.** CRITICAL.

**Origin.** Memory rules `feedback_architecture_principles` + `feedback_compiler_fundamentals`. ADR 0007.

---

### Rule 9 — Local-First Validation Parity

**Rule.** The pre-commit hook and CI run *the same gates with the same flags*. Single source of truth: `scripts/gates.sh` — **every gate defined there has a corresponding CI step**, and any divergence between hook and CI is a CRITICAL bug (ADR 0013: this sentence exists because the `filesize` gate ran locally for months with no CI step — closed by #213). All CI cargo invocations pass `--locked`.

The mandated gate set (as named in `gates.sh`):

```bash
fmt       # cargo fmt --all -- --check
clippy    # cargo clippy --workspace --all-targets --locked -- -D warnings
filesize  # Rule 4 line caps: soft 400 / hard 800 per .rs under crates/
unsafe    # Rule 13: #![forbid(unsafe_code)] present in every crate root
test      # cargo nextest run --workspace --locked
coverage  # cargo llvm-cov, --fail-under-lines at the enforced threshold
deny      # cargo deny check
audit     # cargo audit --deny warnings
machete   # cargo machete (unused dependencies, Rule 12)
replica   # oracle/apps/replica static checks, no Docker
```

When the monolithic `gates.sh all` run is impractical on a dev machine (resource limits), gates may be run staged — but all of them must pass on the exact tree being pushed, documented in the commit body (see DEVELOPMENT_FLOW.md Rule F4).

**Why.** PR #142 broke `--all-targets` because the hook didn't include it; CI also didn't include it; the PR merged with three `expect_used` violations and one `items_after_test_module` violation. This rule prevents repeats by removing the divergence at the source.

**Enforce.** `scripts/gates.sh` is the single script invoked by both `scripts/pre-commit` and `.github/workflows/ci.yml`. An automated `gate-parity` check diffing the two invocations is tracked in #229; parity is currently audited at review time.

**Severity.** CRITICAL.

**Origin.** Memory rule `feedback_local_first_validation`. Direct response to the PR #142 incident.

---

### Rule 10 — ADR for Architectural Decisions

**Rule.** Any change touching ≥ 3 crates, introducing a new public trait, altering the codegen pipeline shape, or establishing a new naming convention requires an ADR under `docs/architecture/decisions/NNNN-*.md` *before* the implementation PR is opened. ADRs follow the existing template (Status, Context, Decision, Consequences).

**Why.** ADR 0007 made the registry migration reviewable. Without ADRs, architectural drift is invisible — and post-hoc reconstruction of "why we did it this way" is unreliable.

**Enforce.** PR template includes "ADR linked or N/A"; review checklist; the architect agent owns ADR drafting on request.

**Severity.** HIGH.

> **Backfill 2026-05-03.** The `audit-2026-05-03` review identified three retroactive ADR gaps (PRs #126, #141, #142). Backfilled by ADRs [0009](../architecture/decisions/0009-mutex-re-entrance-protocol.md) (Mutex Protocol), [0010](../architecture/decisions/0010-formatter-contract.md) (Formatter Contract), [0011](../architecture/decisions/0011-supply-chain-hygiene.md) (Supply-Chain Hygiene). Going forward, ADRs are written *before* the implementation PR per this rule, not after.

---

### Rule 11 — One Branch = One Concern

**Rule.** Each branch and PR addresses exactly one logical concern: one feature, one fix, one refactor, one doc change. Mixed-concern PRs must be split before review. Conventional Commits format `<type>(<scope>): <subject>` is required for all commits.

**Why.** Smaller PRs review faster, revert cleaner, bisect better. Mixed concerns hide regressions.

**Enforce.** PR title regex check; reviewer authority to request split. Pre-commit hook validates Conventional Commits format on commit message.

**Severity.** HIGH.

**Origin.** Memory rules `feedback_branch_workflow` + `feedback_commit_convention`.

---

### Rule 12 — Warnings-Clean, Daily Audited

**Rule.** All code compiles with `-D warnings` across the workspace including `--all-targets`. `cargo deny check`, `cargo audit`, and `cargo clippy --workspace --all-targets` run on every PR and on a nightly schedule. Any new advisory has 7 days to be triaged via tracked issue. Additionally (ADR 0013): every CI cargo invocation uses `--locked` so builds resolve exactly the committed `Cargo.lock`; `cargo machete` guards against dead dependencies; and the MSRV is declared in `[workspace.package] rust-version` with a dedicated CI job compiling on it.

**Why.** Direct port of NASA Rule 10. Rust's compiler + clippy + supply-chain audit replaces C-era static analyzers.

**Enforce.** `[workspace.lints]` sets `warnings = "deny"` workspace-wide; `scripts/gates.sh` runs the full audit set; CI runs the same on every push/PR. The nightly cron schedule is planned, not yet wired — tracked in #229 (until then, dependabot's weekly runs plus per-PR audits are the cadence).

**Severity.** CRITICAL.

**Origin.** Phase E hygiene work (PR #126) introduced `cargo deny` + `cargo audit` + dependabot.

---

### Rule 13 — Forbid Unsafe Code

**Rule.** Every crate in the workspace declares `#![forbid(unsafe_code)]`. Not `deny` — `forbid`, which cannot be overridden by an inner `#[allow]`. A transpiler has no legitimate need for `unsafe`: it reads text, walks ASTs, and emits tokens.

**Why.** With zero `unsafe` in the workspace, whole classes of tooling (Miri, sanitizers) become unnecessary rather than merely passing — the strongest possible safety argument is structural absence, not audited presence. This also makes the safety posture legible to outside reviewers in one grep.

**Enforce.** `#![forbid(unsafe_code)]` attribute in every crate root (all 12 members, #214); the `unsafe` gate in `scripts/gates.sh` (with its 1:1 CI step) fails if any crate root lacks the attribute.

**Severity.** CRITICAL.

**Origin.** ADR 0013. Safety-Critical Rust Coding Guidelines (Unsafety chapter) — the guideline set's central premise applied at maximum strength.

---

### Rule 14 — Stable Error Codes

**Rule.** Every `TyrusError` variant carries a stable, unique code (`TYRUS-EXXXX`) and a category (`Parse` / `Analyze` / `Codegen` / `Io` / `Format`). Codes never change meaning or get reused. Machine-readable output (`tyrus check --json`) exposes the code on every error and diagnostic; tests assert on codes, never on message text.

**Why.** Message strings are presentation, not identity — they change with wording improvements and break every consumer that matched on them. Stable codes are what let the JSON envelope be a real contract (schemaVersion 1) and what make diagnostics documentable (`rustc --explain`-style) later.

**Enforce.** Compiler-enforced exhaustive matches plus uniqueness/format tests in `tyrus_diagnostics` (#216); `errorCode`/`category` exposed additively in the `--json` envelope (schemaVersion 1); CLI and analyzer tests assert codes, never messages.

**Severity.** HIGH.

**Origin.** ADR 0013. The `--json` envelope work (#198) made the missing error taxonomy visible.

---

## Severity legend

- **CRITICAL** — CI-blocking. Cannot merge until fixed.
- **HIGH** — Review-blocking. Reviewer rejects until fixed; emergency override requires written justification in PR description.
- **MEDIUM** — Warning + follow-up issue tracked in GitHub.

---

## Intentional omissions vs NASA Power of 10

- **NASA Rule 3 (no dynamic allocation after init)** is **dropped**. Rust's safe heap (`Vec`, `String`, `HashMap`) is the idiom; banning it would force `arrayvec` everywhere for negligible gain in a non-realtime transpiler. Bounded allocation is enforced indirectly via Rule 2.
- **NASA Rule 5 (≥ 2 asserts/function)** is **replaced by Rule 5 (equivalence-test density)**. C-style `assert!` density is not Rust culture; behavior-parity tests are the load-bearing artifact for a transpiler.
- **NASA Rule 8 (preprocessor restriction)** is **subsumed into Rule 7**. Rust has no preprocessor — the analogous risk is unhygienic codegen, addressed directly by mandating `quote!`.
- **NASA Rule 9 (≤ 1 level of pointer deref)** is **dropped**. Rust's borrow checker + `clippy::needless_borrow` already cover the underlying intent. A hard count rule would conflict with idiomatic `Rc<RefCell<…>>` patterns.
- **Added (no NASA equivalent):** Rules 8 (two-layer arch), 9 (local/CI parity), 10 (ADR), 11 (branch hygiene), 13 (forbid unsafe), 14 (stable error codes). These encode lessons specific to a multi-crate transpiler with ML-agent contributors and a stacked-PR workflow.

---

## Annex — Traceability to the Safety-Critical Rust Coding Guidelines

The [Safety-Critical Rust Coding Guidelines](https://coding-guidelines.arewesafetycriticalyet.org/) (Rust Foundation / Safety-Critical Rust Consortium, draft v0.1) organize their guidance in chapters mirroring the Ferrocene Language Specification. Individual guideline IDs are still draft-unstable, so this table maps at chapter level; it exists to show each Tyrus rule has a recognized industry counterpart (or is deliberately project-specific).

| Tyrus rule | Consortium chapter(s) | Notes |
|---|---|---|
| R1 Bounded Control Flow | Expressions; Functions | Recursion bounds / termination reasoning |
| R2 Bounded Loops | Expressions | Loop bound requirements |
| R3 Minimal Scope | Types and Traits; Statements | Narrowest-scope bindings |
| R4 Function Size and Shape | — | Project-specific (NASA Rule 4 heritage); no direct chapter |
| R5 Test Equivalence Density | — | Project-specific verification strategy for a transpiler |
| R6 Total Error Handling | Exceptions and Errors | No-panic discipline in production paths |
| R7 Macros, Not Strings | Macros | Hygienic generation over textual assembly |
| R8 Two-Layer Architecture | — | Project-specific (ADR 0007) |
| R9 Local/CI Parity | — | Process rule; Consortium scope is language-level |
| R10 ADR Required | — | Process rule; matches the Consortium's own deviation-record practice |
| R11 One Branch = One Concern | — | Process rule |
| R12 Warnings-Clean, Audited | Program Structure and Compilation | Warnings-as-errors, dependency auditing |
| R13 Forbid Unsafe Code | Unsafety | Applied at maximum strength (`forbid`, workspace-wide) |
| R14 Stable Error Codes | Exceptions and Errors | Diagnostic identity and machine-readable contracts |

---

## How to use this document

- **Contributors:** before opening a PR, walk the 14 rules. The PR template references this file.
- **Reviewers:** the merge bar is rule compliance, not opinion. If a PR violates a rule and the violation is justified, the rule must either be amended (separate ADR) or the PR rejected.
- **Maintainers:** propose rule changes via ADR superseding ADR 0008/0013. Never amend this document silently.

## References

- Holzmann, G. J. (2006). [The Power of 10: Rules for Developing Safety-Critical Code](https://web.eecs.umich.edu/~imarkov/10rules.pdf). *IEEE Computer*, 39(6), 95–97.
- [Safety-Critical Rust Coding Guidelines](https://coding-guidelines.arewesafetycriticalyet.org/) — Rust Foundation / Safety-Critical Rust Consortium (draft).
- ADR 0013 — Power of Ten v2 amendment (R13, R14, and rule amendments).
- ADR 0007 — Decorator Registry (precedent for Rule 8).
- Memory rules: `feedback_architecture_principles`, `feedback_compiler_fundamentals`, `feedback_code_quality_strict`, `feedback_local_first_validation`, `feedback_branch_workflow`, `feedback_commit_convention`, `feedback_semantic_equivalence`.
