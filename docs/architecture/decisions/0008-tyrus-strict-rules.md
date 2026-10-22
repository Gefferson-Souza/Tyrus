# 8. Tyrus Strict Rules (Power of Ten)

Date: 2026-05-03
Status: Accepted. Scope narrowed to the legacy crates by [ADR 0016](0016-ppr-foundation-and-constitution.md), effective when ADR 0016 is accepted.

## Context

The project entered a phase of rapid PR delivery (5 PRs in one session: #139, #140, #141, #142, #144). Two failure modes surfaced:

1. **PR #142 merged with strict-rule violations.** Three `clippy::expect_used` violations and one `clippy::items_after_test_module` violation reached `main`. Verified locally: `cargo clippy --workspace --all-targets -- -D warnings` fails on `main` HEAD.

   Root cause: the pre-commit hook (`scripts/pre-commit`) runs `cargo clippy --workspace -- -D warnings` *without* `--all-targets`. CI (`.github/workflows/ci.yml`) runs the same command, also without `--all-targets`. Clippy on the library target alone does not lint `#[cfg(test)] mod tests` blocks, where the violations lived. Both gates were silently lenient on test code.

2. **Architectural drift risk going forward.** With the registry pattern (ADR 0007) only partially adopted (`stdlib/mod.rs` still has scattered match arms; `convert/expr/call.rs::try_convert_array_method` duplicates `stdlib/array.rs` responsibility), a continued PR-per-day cadence could reintroduce hardcoded patterns the team has explicitly committed to remove.

The project already encodes its rules in three places — `CLAUDE.md`, `.cargo/config.toml`, and a set of memory feedback files — but no single document is authoritative, no enforcement is unified, and no formal precedent (ADR) exists for the rules themselves. Without that, the rules drift, partial adoption looks like full adoption, and incidents like PR #142 happen silently.

This ADR is the response.

## Decision

Adopt the **Tyrus Power of Ten** (12 rules, document at `docs/standards/POWER_OF_TEN.md`) as the binding contribution standard for the project, inspired by but adapted from the NASA/JPL Power of Ten Rules for Developing Safety-Critical Code (Holzmann, 2006).

### Authority

`docs/standards/POWER_OF_TEN.md` is the single source of truth. Existing rule statements scattered across `CLAUDE.md`, the memory feedback files, and inline comments are now backed by — and must be consistent with — that document.

### Enforcement (in scope of this ADR's follow-up PR)

A separate PR delivers:

1. `scripts/gates.sh` — single shell script encoding all 5 gate invocations with the `--all-targets` flag where applicable.
2. `scripts/pre-commit` — refactored to call `scripts/gates.sh`. Hook and CI become impossible to drift apart by construction.
3. `.github/workflows/ci.yml` — same `scripts/gates.sh` invocation in the Format / Clippy / Build / Security jobs.
4. The known violations introduced by PR #142 (`crates/tyrus_orchestrator/src/format.rs`) are fixed in the same PR — adding `--all-targets` would otherwise break CI on `main`.

### Rule changes

The 12 rules are listed in `POWER_OF_TEN.md`. Three are entirely new vs prior project state (Rules 8, 9, 10 — though all three encode existing memory rules). Rule 6 is tightened: test-only `#[cfg(test)]` modules using `.expect()` must explicitly carry `#[allow(clippy::expect_used)]`. Rule 4's file-size limit is reaffirmed at 400 (soft) / 800 (hard).

## Consequences

### Positive

- **Single source of truth.** Future contributors and ML agents have one document to consult, not five files plus tribal memory.
- **Hook = CI parity by construction.** `scripts/gates.sh` makes the gap that produced PR #142 architecturally impossible — both invokers run the same script.
- **Audit trail.** Future violations are tracked against numbered rules, easing incident reviews.
- **Enforcement automated.** CRITICAL rules block CI; HIGH rules block review; MEDIUM rules track follow-up. Severity is named in the rule itself, removing reviewer-by-reviewer drift.

### Negative

- **Initial pain.** The current `main` HEAD violates Rule 6 + 9 (PR #142 leftovers). The follow-up enforcement PR must fix these before merge, or `--all-targets` cannot be enabled.
- **More overhead per PR.** Authors must consult the rules and the PR template grows. Mitigation: the template is a checklist, not free-form prose.
- **Risk of rule rot.** Rules added now that turn out to be wrong require a superseding ADR. To prevent silent erosion: rule amendments must be ADRs that explicitly mark this ADR or `POWER_OF_TEN.md` as superseded in part.

### Neutral / scope-defining

- **No retroactive enforcement.** Code merged before this ADR is grandfathered. New violations must be fixed; pre-existing ones are tracked as follow-up issues but do not block unrelated PRs.
- **NASA omissions are intentional.** Rules 3 (no dynamic alloc), 5 (assertion density), 8 (preprocessor), 9 (pointer levels) from the original NASA list are dropped or replaced for reasons documented in `POWER_OF_TEN.md` § "Intentional omissions". This is a deliberate fork, not an oversight.

## Alternatives considered

### Keep rules scattered across `CLAUDE.md` + memory files

**Rejected.** That was the status quo, and PR #142 happened anyway. Multiple sources of truth = no source of truth.

### Adopt NASA Power of Ten verbatim

**Rejected.** Rules 3 (no dynamic alloc), 5 (assertion density), 9 (pointer levels) target C-era safety-critical embedded systems. Tyrus is a Rust transpiler running on developer machines — different threat model, different idioms. Forcing those rules would degrade idiomatic Rust without commensurate safety gain.

### Adopt only `clippy::pedantic` (existing community pattern)

**Rejected.** `pedantic` includes hundreds of style preferences; many conflict with `quote!`-heavy codegen. We adopt specific clippy lints (Rules 4, 6) that target real failure modes and complement them with structural rules (Rules 7, 8) that no clippy lint expresses.

### Wait until Phase 9 to formalize

**Rejected.** Phase 9 (validation pipes) introduces 11 new decorators — exactly the scenario where Rule 8 (two-layer architecture) prevents drift. Formalizing during Phase 9 means catching the drift after the fact. The cost of this ADR now is paid back the first time a contributor follows Rule 8 instead of working around it.

## References

- Holzmann, G. J. (2006). [The Power of 10: Rules for Developing Safety-Critical Code](https://web.eecs.umich.edu/~imarkov/10rules.pdf). *IEEE Computer*, 39(6), 95–97.
- ADR 0007 — Decorator Registry. Establishes the two-layer architecture pattern formalized as Rule 8.
- `docs/standards/POWER_OF_TEN.md` — the rules themselves.
- Incident: PR #142 (`test(orchestrator): add format_code regression gates for #131`) merged with Rule 6 + 9 violations.
- Memory: `feedback_architecture_principles`, `feedback_compiler_fundamentals`, `feedback_code_quality_strict`, `feedback_local_first_validation`, `feedback_branch_workflow`, `feedback_commit_convention`, `feedback_semantic_equivalence`.
