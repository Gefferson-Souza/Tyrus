# Tyrus Constitution

> **Twelve rules for the Piso–Promoção–Recibo (PPR) foundation.** Adopted by [ADR 0016](../architecture/decisions/0016-ppr-foundation-and-constitution.md).
>
> **Status:** Proposto — not in force until the owner accepts ADR 0016. Until then the [Power of Ten](POWER_OF_TEN.md) and the [Development Flow](DEVELOPMENT_FLOW.md) remain the only binding standards.

---

## Why a constitution

The prototypes are rule-heavy and still failed silently: the prototype exited 0 on 143 of 145 probes (it refused 2) and only 20 matched the Node output (audit section 8), unknown decorators were dropped, claims of equivalence were made against 13.8% of the evidence. The lesson recorded in the foundation decision is that rules without a mechanical gate are wishes. This document therefore states each rule together with the mechanism that enforces it and the phase in which that mechanism becomes active. **A rule whose gate does not exist yet prints `SKIPPED(<phase>)`, never `PASS`.**

## The method in three words

- **Piso (floor).** The whole closure (application, production `node_modules`, Node `lib/`) first runs on a pinned embedded V8 over a Rust platform, with output proven identical to the reference Node by the oracle.
- **Promoção (promotion).** A unit becomes native Rust only with a certificate, an equivalence obligation and a measurement gate. Two failing nightly runs in a row send it back to the floor.
- **Recibo (receipt).** A number, a percentage or the word "equal" exists only if a CI workflow wrote it to the ledger; CI rejects the rest.

## Regimes

| Regime | Applies to | Rules |
|---|---|---|
| New | every crate created after ADR 0016 is accepted | this Constitution (R1 to R12) plus the Development Flow (F1 to F10) |
| Legacy | the prototype crates listed in `gates.toml [legacy] crates` (the list only shrinks): `tyrus_analyzer`, `tyrus_ast`, `tyrus_cli`, `tyrus_codegen`, `tyrus_common`, `tyrus_decorator_kinds`, `tyrus_di`, `tyrus_diagnostics`, `tyrus_orchestrator`, `tyrus_parser`, `tyrus_test_utils`, and the `tests` member | the Power of Ten, until `m2-prototype-delete` removes the code |

No new crate depends on a legacy crate (gate `direction`). No new investment goes into the prototype's codegen.

## Numbering

Constitution rules are `R1` to `R12`, Power of Ten rules are `R1` to `R14`, and the numbers collide. Outside this file, always qualify: "Constitution R4", "PoT R4". In the table below `Rn` always means the Constitution.

## The rules

Activation phases follow the program RFC (section 7): **F-1** spikes, **F0** oracle, **M1** hosted floor, **F2** islands (M2). Units are named in the program's catalog; a unit not yet delivered is the reason a gate is `SKIPPED`.

| # | Rule | Mechanism | Activation phase | Delivered by | Prototype defect it prevents |
|---|---|---|---|---|---|
| R1 | Translate, host or refuse; nothing vanishes | census per occurrence (sum of visits equals sum of verdicts); queries return `Verdict`, never `Option`; `let_underscore_must_use` deny; a test injects a silent drop and requires the gate to fail; `cargo check` of the output | M1 at file level; M2 at node level | `place-all-hosted`; `m2-tir-lower-census` | 143 of 145 probes exited 0; decorators outside the known 14 dropped silently |
| R2 | Zero decisions by name | metamorphic rename test; `disallowed_methods` on string comparison in `lower`, `analyze`, `place` | F2 (before it: `SKIPPED`) | `m2-rename-metamorphic-gate` | `class_name.ends_with("Service")`; integer-ness by field name |
| R3 | Platform only by spec identity, with a differential vector | CI rejects an entry without an L1 vector; manifest generated from `#[builtin]`/`#[binding]`; single source declared in `gates.toml` | F2 (before it: `SKIPPED`); own-platform items in the contingent units | `m2-l1-differential-vectors` | numeric `sort`; `.length` in bytes; `1+2+"x"` yielding "12x" |
| R4 | Hypothesis, guard or proof with a witness; optimization is a refinement with an obligation | private `Guard`/`Proof` plus witness checker; registry refuses a rule without an obligation; `--no-accel` and `--all-hosted` identical; mutation score | F2 (before it: `SKIPPED`) | `m2-analyze-guard-proof` | `Arc<Mutex>` by "Service" name (18x slower, lost update); f64 `%` 43x slower |
| R5 | Promotion by empirical gate, by value, revocable, one threshold table | `partition.lock` generated and reviewed by PR; G1, G1b, G2 ([ADR 0018](../architecture/decisions/0018-g1-noninferiority-margin.md) for `g1_margin`) | M1 (G1, G1b); M2 (G2) | `bench-g1-parity-harness`, `m1-g1-parity`; `m2-promotion-g2-loop` | a benchmark that measured Node startup; a fixed "PROVEN" label |
| R6 | One emission boundary; the output always compiles | only `tyrus_emit` uses `syn` (`cargo-deny`); `cargo check` of the output; fuzz | M1: constant `src/main.rs` checked by hash; M2: `tyrus_emit` | `asm-project-emitter`; `m2-refine-emit-syn` | Rust assembled by string in 6 files; 83 of 145 outputs did not compile |
| R7 | Emitted project is self-contained and deterministic | `--locked --offline`; 5 builds in 2 directories with the same hash; `tyrus verify` | M1 | `asm-determinism-verify` | 8 builds produced 7 different `main.rs` (hash-map iteration order) |
| R8 | A claim is a CI artifact | ledger written only by the workflow; `claims --check` by template with `native_share`; `doc-facts`; negative control; only `IDENTICAL` and `NORMALIZED(id)` count as green | F0 | `orc-ledger-ci`, `orc-claims-check` | "all equal" stated against 13.8% of the evidence; five different test counts in docs |
| R9 | The oracle examines itself; the engine has a fingerprint | canaries, mutation test, sealed holdout, skew, triple with `EMIT_SKEW`, widened fingerprint probe in CI | F0 (host in M1) | `orc-self-examination`, `orc-negative-control`, `orc-fingerprint-gate`, `host-fingerprint-zero` | `.trim()` and `contains` in tests; a corpus blind to 4 engine classes |
| R10 | Total and bounded | `unsafe_code = deny` for the workspace and `#![forbid(unsafe_code)]` in every crate outside the allowlist ([ADR 0016](../architecture/decisions/0016-ppr-foundation-and-constitution.md), amendment of PoT R13); depth counter yielding `RangeError`; `unwrap`/`panic` deny | M1 (allowlist and lints); M2 (depth counter) | `gov-gates-toml`, `host-v8-embed-offline`; `m2-tir-lower-census` | unbounded recursion; 80 `format_ident!` calls that could panic |
| R11 | Governance with a budget and a root of trust | `rules-lint` crossing docs and `gates.toml`; tooling at most 25% of LOC; single aggregator gate as the only required check, `enforce_admins=true` and CODEOWNERS | F-1 and F0 | `gov-gates-toml`, `gov-rules-lint`, `gov-ci-aggregator-codeowners` | about 26 thousand words of governance; 39% of touched LOC; checkboxes that did not exist; 3 of 70 PRs merged with a red check |
| R12 | An ADR by mechanical trigger; a superseded ADR is removed | trigger: three or more crates touched, a new public trait, or a new lane; `rules-lint` on the diff; a superseded ADR is deleted when its code leaves | F-1 and F0 | `gov-rules-lint`, `gov-constitution-adr` | 16 ADRs "Accepted" including superseded ones; ADR 0007 against the generic-decorator rule |

## Vocabulary

No iteration is named by a version-style number. What exists today is "the prototypes" (two exploratory iterations) and "Tyrus" (the third iteration, this foundation). Tool versions (`v8`, `v22`, `v26`, `v147`, `deno_core 0.4xx`) are an allowlisted exception. `rules-lint` (unit `gov-rules-lint`) enforces the closed list; until it ships the rule is a review item.

## Power of Ten rules for new crates

A new crate is not under the Power of Ten, so each of its 14 rules has a stated destination. `PoT Rn` is the Power of Ten; `Rn` is the Constitution. Only mechanisms that exist or are owned by a named unit are cited.

| PoT rule | Destination for a new crate | Through |
|---|---|---|
| PoT R1 bounded control flow | replaced | Constitution R10 (depth counter yielding `RangeError`, M2); until then a review item |
| PoT R2 bounded loops | not inherited as a gate | review item under F5 (no mechanical gate exists, even in the legacy regime) |
| PoT R3 minimal scope | inherited | `[workspace.lints]` (`clippy::needless_pass_by_value`), applied to every member |
| PoT R4 function and file size, parameters, nesting | inherited | `[workspace.lints]` (`too_many_lines`, `too_many_arguments`) and the `filesize` gate with thresholds in `gates.toml` |
| PoT R5 equivalence test density | replaced | Constitution R8 and R9 (oracle, ledger) and R5 (promotion by gate) |
| PoT R6 total error handling | replaced | Constitution R10 (`unwrap`/`panic` deny) |
| PoT R7 `quote!` only | replaced | Constitution R6 (single emission boundary, `tyrus_emit`) |
| PoT R8 two-layer architecture | replaced | Constitution R2 (zero decisions by name) and R3 (platform by spec identity) |
| PoT R9 local/CI parity | inherited | Development Flow F4 (`gates.sh all`) and Constitution R11 (single aggregator gate) |
| R10 ADR for architecture | replaced | Constitution R12 (mechanical trigger) |
| PoT R11 one branch, one concern | inherited | Development Flow F1 and the commit format; unchanged |
| PoT R12 warnings-clean, audited | inherited | `--all-targets`, `--locked` and `clippy -D warnings` through `gates.sh`; `cargo deny` and `cargo audit` policy owned by `gov-supply-chain-policy` |
| PoT R13 forbid unsafe | amended | Constitution R10 and the allowlist of ADR 0016 |
| PoT R14 stable error codes `TYRUS-EXXXX` | inherited | applies to user-facing diagnostics of the CLI; the unit `cli-tyrus-build` carries it. No gate yet beyond the legacy exhaustiveness tests, so for new crates it is a review item until that unit ships |

## Relation to the other standards

- [Power of Ten](POWER_OF_TEN.md): binding for the legacy regime. Its R13 is amended by ADR 0016 (allowlist for `unsafe`). What reaches new crates is the table below.
- [Development Flow](DEVELOPMENT_FLOW.md): unchanged; F1 to F10 govern the process in both regimes.
- Foundation record and program RFC live in `.claude/plan/` (not versioned). Numbers in this file are inherited from them with provenance; nothing here is a new measurement.
