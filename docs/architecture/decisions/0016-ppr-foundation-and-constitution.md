# 16. PPR Foundation, the Constitution (R1–R12) and the Amendment of R13

Date: 2026-10-22
Status: Proposto — awaiting owner acceptance. Not in force until accepted. Amends [ADR 0013](0013-power-of-ten-v2.md) (R13) and reopens [ADR 0015](0015-rejected-tooling.md) (Miri, sanitizers, cargo-careful).

## Context

The 2026-10-13 audit concluded that the prototype compiler cannot be repaired bug by bug and that the foundation must be rebuilt as a third iteration in the same repository (foundation decision record, local plan, sha256 `128b55c16df803204c7257992cc3129cd89bce5b7d5a741169cbbb16e3e271ae`). The audit measured defects that the existing standards (Power of Ten, ADR 0008/0013, and Development Flow, ADR 0014) did not prevent:

- 143 of 145 probes exited 0 while producing wrong or no output; decorators outside a closed list were dropped by `from_name(...).map(...)` without a verdict.
- Generated Rust was assembled by string in six files; 83 of 145 outputs did not compile; eight builds produced seven different `main.rs`.
- "All equal" was stated against 13.8% of the evidence; documentation carried five different test counts.
- Roughly 26 thousand words of governance coexisted with checkboxes that did not exist and 3 of 70 PRs merged with a red check.
- ADR 0007 (name-keyed decorator registry) contradicts the owner's generic-decorator rule while still labeled Accepted.

The owner's answers of 2026-10-14 and 2026-10-20 fixed the direction: the first milestone is the **floor** (the whole application running on an embedded V8, 0% compiled, with an oracle and receipts); promotion to native Rust comes in the next milestone and only by measured gate (Q2, Q3). The Gate 0 ceiling measurement returned GO on the four database endpoints (evidence under `.claude/plan/oracle-seed/ceiling/results-2026-10-20/`, local, not versioned; not re-measured here).

Two parts of the existing standards cannot survive that direction unchanged:

1. **PoT R13** requires `#![forbid(unsafe_code)]` in every crate. Embedding V8 needs FFI and therefore `unsafe` in at least one crate (`tyrus_host`).
2. **ADR 0015** rejected Miri, sanitizers and cargo-careful on the premise that no `unsafe` exists, and states that weakening R13 "invalidates three rows of this table and requires revisiting them in the same ADR". This ADR is that revisit.

## Decision

### 1. Adopt Piso–Promoção–Recibo (PPR) as the foundation

PPR is defined in [`docs/standards/CONSTITUTION.md`](../../standards/CONSTITUTION.md): the floor (pinned embedded V8, Rust platform, output proven identical to the reference Node), promotion (native Rust only with certificate, obligation and measured gate, revocable) and receipt (no number or "equal" without a CI-written ledger entry). The invariants I1 to I9 of the foundation record (total verdicts, zero decisions by name, optimization never necessary for correctness, hypothesis versus guard versus proof, empirical revocable promotion, verdict as artifact, pinned reference and engine with fingerprint, deterministic emitted project, value-only boundary) are carried by Constitution R1 to R12.

### 2. Adopt the Constitution R1 to R12

`docs/standards/CONSTITUTION.md` holds the twelve rules. Each states its mechanism and its **activation phase**; a gate whose subject does not exist yet prints `SKIPPED(<phase>)`, never `PASS`. Enforcement lands unit by unit (table in the Constitution); this ADR adopts the norm, not the gates.

### 3. Dual regime

New crates fall under the Constitution; the prototype crates stay under the Power of Ten until they are deleted (see Regime de transição). The PoT header is limited to "legacy crates".

### 4. Amend PoT R13: a closed allowlist for `unsafe`

R13 text is replaced by:

> Every crate declares `#![forbid(unsafe_code)]` **except the crates on the allowlist**. The allowlist is closed and lives in this ADR and in `gates.toml [unsafe_allowlist]`. Adding a crate requires an ADR that amends this section.

Allowlist:

| Crate | Status | Why `unsafe` is needed |
|---|---|---|
| `tyrus_host` | allowed now | FFI to the embedded V8 (via `deno_core`/`v8`) and the codec at the isolate boundary |
| `tyrus_platform` | **conditional**: effective only if the floor-platform ADR (0017, after SP1) chooses a line that requires it | Rust implementation of `internalBinding`, if platform option (c) is chosen |
| `tyrus_heap` | **conditional**: effective only if the N2 GC spike (`m2-spike-n2-gc`) shows a safe slab is insufficient (foundation record, D22: safe slab first, zero `unsafe`) | slab heap |

Rules for allowlisted crates:

- The workspace lint `[workspace.lints.rust] unsafe_code` moves from `forbid` to `deny`. This is necessary: a lint level set to `forbid` by the workspace table is passed to rustc as a command-line `-F`, and no crate that inherits it can then use `#[allow(unsafe_code)]`. Observed on this machine with rustc 1.98.0: `rustc -F unsafe_code` with `#![allow(unsafe_code)]` fails with `E0453: allow(unsafe_code) incompatible with previous forbid`, while `-D unsafe_code` plus `#[allow(unsafe_code)]` on a module compiles.
- Every crate **outside** the allowlist keeps `#![forbid(unsafe_code)]` at its root. All 12 current members already have it, so nothing changes for them; the `unsafe` gate keeps verifying it.
- An allowlisted crate uses `#![deny(unsafe_code)]` at the root and `#[allow(unsafe_code)]` **only on modules named in `gates.toml`**; each `unsafe` block carries a `// SAFETY:` comment, enforced by `clippy::undocumented_unsafe_blocks` at deny.
- The `unsafe` gate (unit `gov-gates-toml`) is extended: crate outside allowlist without `forbid` fails; allowlisted crate with `unsafe` outside a named module, or without `// SAFETY:`, fails.

### 5. Decision on Miri, sanitizers and cargo-careful (reopening ADR 0015)

The premise of the three ADR 0015 rows ("structurally nothing for them to find") no longer holds for the allowlisted crates only. Decision, scoped to those crates (everything else keeps the original rejection):

| Tool | Decision | Scope and activation | Cost note |
|---|---|---|---|
| **Miri** | **Adopt, scoped** | CI job on a pinned nightly running `cargo miri test` over the unit tests of allowlisted crates that exercise `unsafe` without crossing FFI; tests that call into V8 are marked `#[cfg_attr(miri, ignore)]`. State `SKIPPED(host-v8-embed-offline)` until the first `unsafe` exists; blocking from the PR that introduces it | Miri does not execute foreign code, so it checks the Rust-side invariants (codec, slab, wrapper lifetimes), not V8. ADR 0015 recorded 10 to 100x test time; with the scope above the set is small `[NV]`. The `miri` component is **not installed** on this machine (`cargo miri --version` fails on stable 1.98.0), so nothing here was run |
| **AddressSanitizer** | **Adopt, non-blocking nightly** | integration tests of `tyrus_host` once the host exists; report-only for one release, then reconsider blocking | needs nightly `-Zsanitizer=address`; the prebuilt V8 archive is not instrumented, so only Rust-side and FFI-boundary memory errors are visible `[NV]` |
| **ThreadSanitizer, MemorySanitizer** | **Keep rejected** | — | premise: one thread per isolate (foundation record D8); MSan additionally needs instrumented std and C++ dependencies |
| **cargo-careful** | **Keep rejected** | — | marginal over Miri on the same crates |

Mark of provisional status: this table is a **recommendation** for the owner. The owner has not decided; the alternatives and consequences are in the next section.

### 6. ADR housekeeping (Constitution R12)

- ADR 0007 (decorator registry) contradicts foundation decision D5 and is marked superseded for new work; it governs the legacy code only until deletion.
- ADRs 0009 (mutex re-entrance for generated service state; the new design has no `Arc`/`Mutex` service state) and 0012 (array dispatch inside the legacy codegen) are marked legacy-scoped.
- ADRs 0008 and 0013 are marked scope-narrowed (legacy crates); 0013 additionally "R13 amended by ADR 0016"; 0015 is marked reopened for the allowlist.
- A superseded or legacy-scoped ADR is **deleted** when its code leaves (`m2-prototype-delete`); `rules-lint` will check the index.
- Mechanical trigger for a new ADR: three or more crates touched, a new public trait, or a new lane (Constitution R12).
- The ADR index in `docs/ARCHITECTURE.md` is updated in the same commit.

### 7. Not decided here

- `ROADMAP.md` is **not** edited. A proposed rewrite (the M0/M1/M2 ladder) is delivered as a diff for the owner (re-prioritization is the owner's decision).
- The floor platform and engine line (ADR 0017), the oracle tooling (ADR 0019) and `g1_margin` ([ADR 0018](0018-g1-noninferiority-margin.md), provisional 1.05) are separate ADRs.

## Consequences

**Positive**

- Every rule has a mechanism and a phase; the gap between rule text and enforcement (the failure mode of the prototypes) is closed by construction: no gate means no rule.
- `unsafe` is confined to named crates and named modules with a documented invariant each; the grep-level legibility of R13 is kept for the other crates.
- ADR 0015 is internally consistent again: its R13-dependent rejections are decided in the same ADR that weakens R13.

**Negative / accepted costs**

- Two regimes coexist until the prototype is deleted (M2); reviewers must know which one applies. The `gates.toml [legacy]` list and the `direction` gate make that mechanical.
- A Miri job and a sanitizer job add CI time and a nightly toolchain pin (a moving part). Accepted only for the allowlisted crates; both are `SKIPPED` until `unsafe` exists.
- `deny` instead of `forbid` at workspace level weakens the compiler's own guarantee for any crate that forgets its root attribute; the `unsafe` gate compensates and is required by Constitution R10.
- The Constitution is large for a single-maintainer project. Constitution R11 caps tooling at 25% of LOC and deletes any rule without a gate.

## Alternativas rejeitadas (alternatives rejected)

| Alternative | Rejected because | Measured/known consequence |
|---|---|---|
| Keep PoT R13 unchanged and host V8 outside the workspace (a separate repository or an external binary) | The emitted project must be self-contained and deterministic (Constitution R7); an external binary breaks `--locked --offline` reproducibility and the single `gates.sh`. | Two repositories to gate; the oracle cannot fingerprint the host from this tree. |
| Drop `forbid` for the whole workspace (`unsafe` allowed anywhere, review-only) | Loses the structural guarantee for 12 current crates and every new one; "present by audit" is the weaker argument ADR 0013 rejected. | Gate must grep for `unsafe` instead of an attribute; no compiler enforcement. |
| Allow `unsafe` only through a vetted dependency (`deno_core` as is), no own `unsafe` | Does not hold: the codec at the isolate boundary and the platform bindings need own `unsafe` unless entirely delegated, which is the K8 fork-cost risk. | Unknown until SP1; the conditional allowlist entries cover it without committing now. |
| Reject Miri again with a new premise (FFI-only `unsafe`, Miri cannot execute V8) | Leaves the Rust-side invariants (codec, slab, lifetimes) unchecked by any tool. | Residual risk accepted explicitly if the owner picks this; cheapest option, zero CI cost. |
| Adopt Miri and sanitizers workspace-wide | Non-allowlisted crates have no `unsafe`, so the runs are wasted (ADR 0015: 10 to 100x test time). | Pure CI cost with no finding possible. |
| Replace the Constitution by amending the Power of Ten in place | PoT is anchored in legacy codegen concerns (R7 quote-only, R8 two-layer) that the new foundation replaces; one document would carry two contradictory regimes. | The ADR 0014 argument for two documents applies again. |

## Regime de transição (transition regime)

1. **Now (this ADR accepted):** the Constitution is in force for new crates; `gates.toml` does not exist yet, so activation is announced by each unit. Until `gov-gates-toml` merges, every Constitution rule is a review item with `SKIPPED` semantics (no rule is claimed enforced).
2. **`gov-gates-toml` merges:** `[legacy] crates` freezes the 11 prototype crates plus `tests` under the Power of Ten; `unsafe_code` moves to `deny`; the `unsafe` and `direction` gates start; thresholds (including `g1_margin` from ADR 0018) become readable.
3. **Host lands (`host-v8-embed-offline`):** first `unsafe`; `tyrus_host` enters the allowlist; Miri and ASan jobs leave `SKIPPED`.
4. **`proto-freeze`:** the prototype is frozen as a negative control with the binary hash.
5. **`m2-prototype-delete`:** the legacy crates, the PoT, and the legacy-scoped ADRs (0007, 0009, 0012, and the PoT-specific parts of 0008 and 0013) are removed (Constitution R12). Until then no investment in the prototype's codegen; the branch `fix/codegen-no-silent-statement-drop` is governed by `proto-wip-disposition`.

Rollback: reverting this ADR restores the PoT as the single standard; no code depends on it yet.

## Gatilhos de revisão (review triggers)

- Platform ADR 0017 selects a line that needs `tyrus_platform` with `unsafe`: record the entry here by amendment, enable Miri for it.
- The N2 GC spike concludes: `tyrus_heap` is activated or removed from the allowlist.
- A Miri or ASan finding, or a month with no `unsafe` change in allowlisted crates: reconsider cost of the jobs.
- A new crate asks for `unsafe`: new ADR, no silent addition.

## Perguntas ao dono (owner questions)

1. Accept PPR and the Constitution (R1 to R12) as the norm for new crates?
2. Accept the R13 amendment with the closed allowlist (`tyrus_host` now; `tyrus_platform`, `tyrus_heap` conditional)?
3. Miri and sanitizers: the recommendation above (Miri scoped and blocking; ASan non-blocking nightly; TSan, MSan, careful rejected) or the cheapest alternative (reject again, new premise)?
4. Approve the proposed `ROADMAP.md` diff (delivered separately, not applied)?

## References

- [ADR 0008](0008-tyrus-strict-rules.md) and [ADR 0013](0013-power-of-ten-v2.md) — the Power of Ten and its R13.
- [ADR 0014](0014-development-flow-rules.md) — Development Flow; unchanged.
- [ADR 0015](0015-rejected-tooling.md) — the rejections reopened here.
- [ADR 0018](0018-g1-noninferiority-margin.md) — `g1_margin`, Constitution R5.
- [`docs/standards/CONSTITUTION.md`](../../standards/CONSTITUTION.md).
- Local plan (not versioned): foundation decision record sections 1, 2 (D5, D8, D19, D21, D22, D24, D26), 7, 10 (sha256 above); `program-rfc.md` section 4.1 and 7 (sha256 `5f5be5ec2df074b2602744225f39144f309634a933db9bcb048a96bc4735a961`); owner decisions `decisoes-provisorias-2026-10-20.md`.
