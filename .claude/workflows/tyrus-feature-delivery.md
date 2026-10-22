# Tyrus Autonomous Delivery Workflow (multi-agent · loop · gates)

> Distilled from the duofy autonomous-delivery playbook (`kingposto-backend/.claude/workflows/backend-feature-delivery.md`) and this repo's own standardization campaign (12 PRs, 2026-08-05). Phases are **operational steps**; each one names the binding rule it enforces — `docs/standards/POWER_OF_TEN.md` (R1–R14) and `docs/standards/DEVELOPMENT_FLOW.md` (F1–F10) remain the normative spec. This document never overrides them.
>
> **Owner-authorized autonomy level:** the loop runs end to end — issue → branch → TDD → review → PR → **merge with green CI** — without asking. It STOPS for the human only at the PARA conditions listed at the bottom.

---

## Inviolable principles (always in force)

- **RULE ZERO (the project's reason to exist):** a transpilation change is "working" only when **observed equivalence** exists — the TypeScript ran under Node and the generated Rust compiled and ran, and their outputs are identical (stdout for programs, HTTP responses for the NestJS tier). Snapshot green ≠ equivalence. Token stream "looks right" ≠ equivalence. (R5 + F6)
- **VALIDATE BEFORE ASSERTING:** never assume — confirm by reading the real code (`file:line`, not a remembered symbol name) and by running the real command. Memory, summaries, and docs are *inputs to verify*, never ground truth. When memory conflicts with code, code wins and the memory gets rewritten. (F10 + MEM-RECONCILE below)
- **FOCUS = the issue's stated scope.** Opportunistic refactors, drive-by cleanups, and "while I'm here" changes do NOT enter the branch — they become new issues. One branch = one concern. (F1/R11)
- **Roadmap authority order:** open CRITICAL bugs > `ROADMAP.md` **Now** > **Next** > **Later**. The loop never invents work: every unit traces to an issue or a roadmap line, and re-prioritizing the roadmap itself is a human decision.
- **Red lines (any at risk → STOP):**
  - Never merge a codegen/analyzer behavior change without an equivalence/reproduction test that was RED first (F2, CRITICAL).
  - Never reintroduce deleted anti-patterns: `find_param_decorator`, `extract_single_decorator`, `ends_with("Controller")`, string-concat codegen (R7), scattered name-match dispatch (R8).
  - Never edit `CHANGELOG.md` (release-plz owns it) and never rewrite git history or force-push.
  - Never push with a red gate (F4) and never use CI as a debugger.
  - No panics in production code paths — `Result<T, TyrusError>` everywhere (R6).

---

## The phases (each `→` opens only when the previous gate passes)

```
P0 CAPABILITY ─▶ P1 QUEUE ─▶ P2 MEM-RECONCILE ─▶ ┌─ per WorkUnit ────────────────────────────────┐ ─▶ P7 DELIVER ─▶ loop
   crate map      roadmap+       memories of        │ P3 PLAN ▶ P4 TDD ▶ P4.5 TEST-AUDIT           │    push+PR+CI
   (1×/session)   issues→fila    the surface        │  (‖N+judge) (RED→GREEN) (‖ adversarial)      │    +merge+ledger
                                 (ground truth=code) │           ▶ P5 REVIEW ▶ P6 EQUIVALENCE+GATES │    (autonomous)
                                                    └───────────────────────────────────────────────┘
```

`(‖)` = parallel multi-agent · `(‖N+judge)` = N concurrent + judge. Tier shortcuts: see P1.

### P0 — CAPABILITY (1×/session) — *mitigates: recreating what exists*

Ground the map before any work: `CLAUDE.md` crate table + codegen module tree, verified with `ls crates/ crates/tyrus_codegen/src/**` when anything will be created. Before writing ANY new helper/module/handler, grep for an existing one (`tyrus_common::util`, `decorators/`, `stdlib/`) — reuse > new. Registry additions follow `/new-decorator` (1 handler file + 1 register line + 1 enum variant), never new dispatch sites.

**Gate:** for creation-shaped work, the plan cites what was searched and why nothing existing fits.

### P1 — QUEUE (roadmap+issues → ordered WorkUnits) — *mitigates: wrong order / diffuse scope*

`gh issue list` + `ROADMAP.md` → each unit becomes `WorkUnit{ issue, tier, deps, surface, acceptance }`.

- **Order:** CRITICAL bugs → roadmap **Now** (top to bottom) → **Next** → **Later**. Dependabot PRs are handled as interrupts (review the diff, merge on green CI; majors get a changelog read first).
- **Tier:** `trivial` (≤ 1 file, mechanical) → P3 is a written 5-line plan, P4.5 is a single skeptic pass; `medium` → full cycle; `large` (≥ 2 crates, new trait, new concept) → full cycle + F3 plan review is mandatory + F9 ADR check.
- **Surface** = crates/modules the unit touches (input for P2 and the scope guard).

**Gate:** queue written to `.claude/plan/fila.md` with tiers and order (F10 — state lives in the plan ledger).

### P2 — MEM-RECONCILE (scoped to the queue's surface) — *mitigates: trusting stale memory*

For each auto-memory claim touching the surface: read it as a *claim* → verify against live code (grep the symbol, read the `file:line`) → rewrite from the CODE if wrong, with provenance. Irreconcilable conflict (code doesn't decide) → PARA and ask.

**Gate:** surface memories verified or rewritten. Never build a plan on an unverified memory.

### P3 — PLAN (creates issue + branch) — *enforces F1, F3, F9*

1. **Issue first** (or adopt the existing one); branch `<type>/<slug>` referencing it.
2. `trivial`: 5-line plan in the issue or `.claude/plan/`. `medium`/`large`: dispatch **tyrus-planner** (read-only) for the plan; `large` additionally runs N-plans+judge via `.claude/workflows/plan-panel.js` and an adversarial pass by **plan-reviewer** (fresh context — the reviewer did not write the plan).
3. R10/F9 triggers (≥ 3 crates, new public trait, pipeline shape, new normative rule, tooling adoption/rejection) → ADR written BEFORE implementation, index updated in the same commit.

**Gate (blocks):** plan slices are small, each with file · acceptance criterion · risk; scope closed; for `large`, plan review verdict is at worst approved-with-caveats. A CRITICAL hole → replan.

### P4 — TDD (RED → GREEN, per slice) — *enforces F2, R5, R6, R7, R8*

1. **RED:** **tyrus-tester** writes the test first — semantic equivalence test in `tests/src/equivalence/` for codegen (valid TS through `assert_output_equivalent`), reproduction test for bugs. Run it; **paste the observed failure**. Not failing → the test is wrong or the feature already works (stop, re-verify the premise).
2. **GREEN:** **tyrus-codegen** implements minimally — `quote!` only, registry for name-keyed logic, no panics, ≤ 50-line functions. Pure refactors (zero behavior change) are exempt from RED but must state so in the PR body.
3. **Mutation-check:** break the key generated-output line → the test MUST fail. If it doesn't, the test is vacuous → rewrite it.
4. Snapshots changed → `cargo insta review`, accept only intentional diffs, commit `.snap` in the same change.

**Gate (blocks):** RED observed → GREEN observed → mutation-check confirmed → atomic Conventional Commit (hook validates).

### P4.5 — TEST-AUDIT — *mitigates: vacuously green tests*

`trivial`: one skeptic pass by **tyrus-reviewer** over the tests only. `medium`/`large`: adversarial panel via `.claude/workflows/adversarial-audit.js` — (a) independent mutation-check, (b) does the test exercise the *acceptance criterion* or just the type system?, (c) is the TS snippet valid TypeScript?, (d) anti-regression covered?

**Gate (blocks):** adversarial verdict APPROVED. No test is "valid" without this.

### P5 — REVIEW (fresh context, adversarial) — *enforces F5*

Dispatch **tyrus-reviewer** (xhigh, read-only) on the diff — written to a scratch file, never pasted inline; the prompt restates constraints per F8 and NEVER tells the reviewer what not to flag. CRITICAL/HIGH findings: fix and re-review. MEDIUM: fix or open an issue.

**Gate (blocks):** zero sustained CRITICAL/HIGH.

### P6 — EQUIVALENCE LIVE + GATES — *enforces F4, F6, R9, R12 — RULE ZERO's gate*

1. `/gates` — all ten (fmt · clippy · filesize · unsafe · test · coverage · deny · audit · machete · replica), staged is fine, **every gate on the exact tree being pushed**. Never read pass/fail through a pipe.
2. The equivalence suite IS the E2E: `cargo nextest run --workspace` includes every equivalence test running real Node vs real compiled Rust. NestJS-tier changes additionally show `test_http_equivalence_rust_server` green.
3. **Proof-of-delivery package** (goes verbatim into the PR body, F6): the RED failure output, the GREEN pass output, mutation-check note, gates verdict line, and — for bugs — the reproduced BEFORE symptom vs the AFTER on the same input. "The code looks correct" is never an acceptance statement.
4. **Docs same PR** (F7): if counts/trees/commands/rules changed → `/doc-sync` (dispatch **tyrus-docs**) inside this branch. `CHANGELOG.md` untouched.

**Gate (blocks):** nine green + observed equivalence + proof package assembled.

### P7 — DELIVER (autonomous through merge) — *enforces F1, F4, F10*

1. Push branch → PR via `/pre-pr` shape: Summary, `Closes #N`, Test plan with observed output, proof package inline.
2. Monitor CI (`gh pr checks` poll — no `--json` here; parse the table). **Verify the checks belong to the HEAD SHA** before merging (a monitor can catch the previous commit's green).
3. Merge (squash) on green → delete local+remote branch → update `.claude/plan/fila.md` ledger (F10) → mark the roadmap line if one closed.
4. Loop → next WorkUnit from P1's queue (P0/P2 only re-run if the session or surface changed).

---

## Agent team per phase

| Phase | Team | Mode |
|---|---|---|
| P0–P1 | main loop (+ `Explore` for broad sweeps) | read-only |
| P2 | main loop (grep/Read vs memory) | read-only |
| P3 | **tyrus-planner** → `plan-panel.js` (large) → **plan-reviewer** adversarial | read-only, fresh contexts |
| P4 | **tyrus-tester** (RED) → **tyrus-codegen** (GREEN) | sequential — GREEN only after observed RED |
| P4.5 | **tyrus-reviewer** (trivial) / `adversarial-audit.js` panel (medium+) | read-only, fresh context |
| P5 | **tyrus-reviewer** (xhigh) | read-only, fresh context |
| P6 | main loop `/gates` + **tyrus-docs** (F7) | write only docs |
| P7 | main loop (git/gh + CI monitor) | — |

Every dispatch restates the binding constraints verbatim (F8): applicable F/R rules, branch, commit format `<type>: description`, paths not to touch, output format, and that generated-code checks are observed, not inferred. Reviews always run in a context that did not author the diff.

## Loop rules

- **PARA (stop and ask the human) when:** (a) a gate fails **2× in a row** — report with evidence, never force; (b) an architectural decision has genuinely conflicting alternatives (ADR-shaped ambiguity); (c) release/versioning decisions (e.g. #184) or anything touching git history; (d) a red line is at risk; (e) scope wants to leak past the issue; (f) irreconcilable memory-vs-code conflict; (g) roadmap re-prioritization would be needed; (h) a push/merge fails for a non-mechanical reason.
- **Exit conditions (anti-infinite-loop):** queue empty · gate stuck 2× · no measurable progress between iterations · session cost/time limit.
- **State is always persisted** in `.claude/plan/fila.md` + per-unit plan files (survives compaction, F10). Every gate records real evidence — exit codes, pasted outputs — never "trust me".
- **Disk pressure is a known failure mode:** llvm-cov needs ~4.5 GB; if ENOSPC, clear `$TMPDIR/tyrus_test_target` and `target/llvm-cov-target`, then rebuild the `tyrus` bin before the test gate.

## Error-mitigation map

| Failure that happened / can happen | Phase that now catches it |
|---|---|
| Recreating a deleted helper or a second `to_snake_case` | P0 capability + P3 reuse check |
| Trusting 168-day-old memory (dead infra, moved files) | P2 MEM-RECONCILE |
| "Tokens look right" shipped without running anything | P4 RED-first + P6 RULE ZERO gate |
| Vacuously green test (invalid TS, asserts nothing) | P4 mutation-check + P4.5 audit |
| Plausible-but-wrong AI diff | P5 fresh-context adversarial review |
| Doc drift (three contradictory test counts) | P6 step 4 — docs same PR |
| Scope creep mid-branch | P1 surface + PARA (e) |
| Merging on the previous commit's green CI | P7 SHA verification |
| Aspirational enforcement claims in docs | reviewer brief (ADR 0013) + `/doc-sync` |
| Runaway loop / cost | exit conditions |

## Workflow self-validation

Before trusting a change to THIS file in production: dry-run one real WorkUnit through all phases and confirm each gate produced **real evidence** (a pasted RED, a mutation-check that failed when it should, a reviewer finding). A phase that never blocks anything is decoration — tighten it or delete it.
