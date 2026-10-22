# 13. Power of Ten v2 — Rules 13–14 and Amendments

Date: 2026-08-04
Status: Accepted (amends ADR 0008). Scope narrowed to the legacy crates and R13 amended (closed allowlist for `unsafe`) by [ADR 0016](0016-ppr-foundation-and-constitution.md), effective when ADR 0016 is accepted.

## Context

The Power of Ten (ADR 0008) has been binding since 2026-05-03. Three months of enforcement exposed gaps between what the document says and what the project does, plus two classes of defect no existing rule covers:

1. **R9 divergence in the wild.** The `filesize` gate existed in `scripts/gates.sh` and ran in the pre-commit hook, but had no corresponding CI step — exactly the hook/CI divergence R9 calls a CRITICAL bug, sitting inside R9's own enforcement stack. The mandated gate list in the rule text was also stale (missing `filesize` and `coverage`).
2. **R5 said 80%, the gate enforced 73%.** The rule text claimed a coverage threshold the project does not enforce, which teaches readers that rule text is aspirational — corrosive to the whole document's authority.
3. **Panicking indexing is not covered by R6.** `x[i]`, `&s[a..b]` and `.unwrap()` inside `Result`-returning functions panic exactly like the constructs R6 bans, but none of the enforced lints caught them.
4. **No rule prevents `unsafe`.** Nothing stops an `unsafe` block from entering a codebase that structurally never needs one.
5. **No error identity.** `TyrusError` variants have no stable codes; the `--json` envelope (#198) exposes message strings, which consumers will match on and which break with every wording change.
6. **R4 forces artificial splits.** In `quote!`-heavy emission functions, the 50-line limit sometimes fragments one coherent template into pieces that are harder to review than the original.

Independent research (Rust API Guidelines; Safety-Critical Rust Consortium coding guidelines, draft v0.1; clippy lint taxonomy; mutation-testing practice) was conducted to align the amendments with current industry standards rather than invent project-local dogma.

## Decision

Amend `docs/standards/POWER_OF_TEN.md` from 12 to 14 rules:

**New rules**

- **R13 — Forbid Unsafe Code (CRITICAL).** `#![forbid(unsafe_code)]` in every crate root, grep-gated. `forbid` (non-overridable), not `deny`. Consequence: Miri, sanitizers and `cargo-careful` are *structurally* unnecessary and are rejected as tooling — recorded in ADR 0015, the companion decision of this amendment campaign.
- **R14 — Stable Error Codes (HIGH).** Every `TyrusError` variant carries a unique `TYRUS-EXXXX` code and a category; JSON output exposes codes; tests assert codes, not messages.

**Amendments**

- **R4:** targeted `#[expect(clippy::too_many_lines, reason = "…")]` escape hatch for coherent `quote!` templates; `#[expect]` mandated over `#[allow]`; per-PR exception count reported.
- **R5:** coverage text now states the *enforced* threshold (73%) with a +1 pp/sprint ramp to 80% (issue #163), naming `gates.sh`/CI as source of truth; `cargo mutants --in-diff` added as the coverage-emptiness check.
- **R6:** forbidden-construct list extended with `clippy::indexing_slicing`, `clippy::string_slice`, `clippy::unwrap_in_result`.
- **R9:** "every gate in `gates.sh` has a CI step" is now the rule's literal text; gate list updated to the real seven (fmt, clippy, filesize, test, coverage, deny, audit); `--locked` mandated for CI; staged-gates provision cross-referenced to DEVELOPMENT_FLOW.md F4.
- **R12:** `--locked` on all CI cargo invocations; `cargo machete` for dead dependencies; MSRV declared in `[workspace.package] rust-version` plus a CI job compiling on it.

**Annex.** A chapter-level traceability table maps R1–R14 to the Safety-Critical Rust Coding Guidelines (Rust Foundation / Consortium). Chapter-level, not guideline-ID-level, because the guideline IDs are draft-unstable (v0.1); rules with no counterpart are marked project-specific rather than force-fitted.

## Consequences

**Positive**

- The document's claims match the project's enforcement again — restoring the "rule text is binding" property ADR 0008 depends on.
- Panic surface shrinks: indexing/slicing panics become compile errors; `unsafe` becomes impossible rather than reviewed.
- Error codes turn the `--json` envelope into a real contract and unblock `--explain`-style documentation later.
- External traceability (Consortium annex) grounds the standard in recognized industry work — relevant for the project's academic positioning.

**Negative / accepted costs**

- Enforcing the R6 additions and R13 requires code changes across the workspace (lint fixes, crate-root attributes) — scheduled as dedicated units (CI hardening, workspace lints, error taxonomy) in the standardization RFC; until those land, the amended rules are binding for *new* code and tracked for existing code.
- The R4 escape hatch can be abused; the per-PR `#[expect]` count report is the counterweight.
- MSRV declaration adds one more CI job (~2 min) per PR.

## Enforcement status at adoption

Rule text marks each not-yet-wired mechanism with its tracking issue, so the standard never silently overstates enforcement:

| Amendment | Wiring | Tracked in |
|---|---|---|
| R9 filesize CI step, `--locked` | CI workflow | #213 |
| R13 `forbid(unsafe_code)` + grep gate | crate roots + `gates.sh` | #214 |
| R6 indexing/slicing lints | `[workspace.lints]` migration | #215 |
| R14 error codes | `tyrus_diagnostics` + JSON envelope | #216 |
| R5 mutation testing | CI + local tooling | #217 |
| R12 machete / MSRV | CI workflow | #213 |

## Alternatives rejected

- **Relax R4 to clippy's default 100 lines** — rejected: the 50-line JPL-heritage limit is the document's identity; an audited escape hatch preserves rigor without forcing artificial fragmentation.
- **Guideline-ID-level Consortium mapping** — rejected while upstream IDs are draft-unstable; chapter-level mapping is honest and maintainable.
- **Encode R14 as message-format conventions instead of codes** — rejected: formats drift; opaque stable codes are the only identity that survives rewording.

## References

- [POWER_OF_TEN.md](../../standards/POWER_OF_TEN.md) — the amended standard.
- [ADR 0008](0008-tyrus-strict-rules.md) — original adoption.
- [ADR 0014](0014-development-flow-rules.md) — process companion (F-rules).
- [Safety-Critical Rust Coding Guidelines](https://coding-guidelines.arewesafetycriticalyet.org/) — Rust Foundation / Safety-Critical Rust Consortium.
- [cargo-mutants](https://mutants.rs/) — mutation testing used as the R5 emptiness check.
