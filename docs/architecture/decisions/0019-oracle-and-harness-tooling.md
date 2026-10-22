# 19. Oracle and Harness Tooling

Date: 2026-10-22
Status: Proposto

## Context

The execution program (`program-rfc.md`, wave 1) adds a verification layer that did not exist in the repository: the oracle (`crates/tyrus_oracle`, `oracle/oracle.lock`), the conformance batteries (`oracle/batteries/`) and the dev replica app (`oracle/apps/replica/`). Rule F9 ([ADR 0014](0014-development-flow-rules.md)) requires an ADR before adopting tooling. Three wave-1 units carry `merge_after: adr-oracle-tooling` for that reason: `orc-oracle-lock-verdicts`, `orc-conformance-batteries` and `orc-dev-replica-app`.

This ADR records what the wave-1 branches **actually use** (measured on `feat/orc-oracle-lock-verdicts` @ `3950935`, `feat/orc-conformance-batteries` @ `4bc5667`, `feat/orc-dev-replica-app` @ `8d348ba`, and on the two out-of-repo spikes `sp2-census` and `sp4-profile` under `~/.local/share/tyrus-spikes/`), not what the plans promised. Where the two differ, the difference is stated in "Decision 4".

Status is **Proposto**: the owner accepts. Choices that rest on `decisoes-provisorias-2026-10-20.md` ("continua fazendo, depois mudamos") are marked **provisional**; every one is reversible by `git revert` plus a rerun of the affected unit.

## Decision

### 1. Tools adopted, with exact versions

| # | Tool | Version | Used by | Scope |
|---|---|---|---|---|
| a | Python 3 standard library, always `python3 -I` | minimum 3.12 (CI `ubuntu-latest` ships 3.12; measured 3.12.3) | batteries runner, replica harness, spike scripts | `oracle/`, `bench/`; zero `pip` dependencies. Imports measured: `argparse`, `difflib`, `hashlib`, `http.server`, `json`, `subprocess`, `unittest`, `fcntl` and the like. `fcntl` is Unix-only, which matches the CI runner and the dev machine. |
| b | Node, pinned exactly | **26.11.1** reference (V8 14.6.202.34-node.37, ICU 78.3, Unicode 17.0, tz 2026e, CLDR 48.0); **22.21.1** seed (V8 12.4.254.21-node.33, ICU 77.1, tz 2025b, CLDR 47.0) | probes, ST0, batteries goldens, replica | Tuple pinned in `oracle/oracle.lock`; located through `TYRUS_NODE_REFERENCE` / `TYRUS_NODE_SEED` (gate `node-pins` fails when unset; never a skip). CI installs both with `actions/setup-node@v7`. |
| c | `cargo-mutants` | **27.1.0** | mutation check of `tyrus_oracle` (`is_green`) | Development tool. Wave 1 does **not** wire it in `ci.yml`; see "Consequences". |
| d | `jsonschema` crate | **0.59.0**, `default-features = false` | `tyrus_oracle` tests (`oracle.lock` and verdict schemas) | **dev-dependency only**; the schema must require `cldr`. |
| e | npm lockfile policy | npm 11.20.0 (bundled with Node 26.11.1) | replica app, `sp2-census` | `package-lock.json` with exact versions and integrity, installed with `npm ci`, never `npm install`. Replica pins Nest 10.4.22, TypeORM 0.3.31, pg 8.23.1, TypeScript 5.1.6, webpack 5.112.0. `sp2-census` pins acorn 8.19.0, typescript 5.9.3, resolve.exports 2.0.4. |
| f | Docker (measured 29.9.1) | Postgres 17 image **pinned by digest** `sha256:b0f9560a2de083e2cc7382e75f808c7381a32852a7ec49117deedb300e552b24` | replica database | Container on tmpfs, `127.0.0.1` only. The digest lives in `db/replica.env`. |
| g | `cargo-nextest` | 0.9.143 | already the test runner | Listed for completeness; the oracle tests that run the real Node run under it (the branch's last commit serializes the probe tests). |
| h | `shellcheck` | CI only, **unpinned on the runner image today** (see Decision 3) | `scripts/gates.sh replica` with `TYRUS_REQUIRE_SHELLCHECK=1` | Absent on the dev machine. |
| i | Bun | 1.2.22 | negative control only (batteries must flag `sin`/`cos` differences) | Never an oracle; never in CI. |

The negative-control engines of the batteries (Node 22.21.1, Node 24.21.0, Bun 1.2.22) and the 16 mutant preloads are test fixtures for the batteries, not oracles. The single reference is the Node 26.11.1 tuple.

### 2. Tools explicitly not adopted

`pytest` and `ruff` as required dependencies; `actionlint` (until it exists on the owner's machine and the CI pins a version). Reasons in "Alternativas rejeitadas".

### 3. Pinning policy for tools that are not yet pinned

The wave-1 CI references third-party actions by tag (`actions/setup-node@v7`, `actions/checkout@v7`), as the rest of the repository already does. `shellcheck` on `ubuntu-latest` is whatever the runner image ships. Neither is a version pin in the sense this ADR demands for Node, `cargo-mutants` and `jsonschema`. **Recommendation:** keep the current action tags (consistent with the existing workflows) and pin `shellcheck` and `actionlint` by version in a separate follow-up (`gov-supply-chain-policy`, ADR 0011 Policy 4), not inside this ADR's wave. **Provisional.**

### 4. Decision point: the `Cargo.lock` bumps forced by `jsonschema`

`orc-oracle-lock-verdicts` states in its plan that `Cargo.lock` changes are "só adições". The branch actually contains 30 added packages **and three version bumps of crates already in the lock**:

| Crate | Before | After | Forced by (minimum bound read in the crate's `Cargo.toml`) | In the production graph? (`cargo tree --locked -e normal,build`) |
|---|---|---|---|---|
| `zmij` | 1.0.20 | 1.0.23 | `jsonschema-value 0.59.0` requires `zmij = "1.0.23"` | **Yes.** `zmij` is a dependency of `serde_json`; it reaches `tyrus_analyzer`, `tyrus_orchestrator`, `tyrus_cli` and `tyrus_oracle`. |
| `num-bigint` | 0.4.6 | 0.4.8 | `fraction 0.17.0` requires `num-bigint = "0.4.8"` | **Yes.** Reaches the SWC parser chain (`tyrus_parser`, `tyrus_analyzer`, `tyrus_cli`). |
| `regex-automata` | 0.4.14 | 0.4.18 | `fancy-regex 0.19.2` requires `regex-automata = "0.4.16"` | No normal/build path; dev graph only (`assert_cmd` via `bstr`, and `jsonschema`). |

The consequence the plan did not state: although `jsonschema` is a dev-dependency, `Cargo.lock` is workspace-wide, so **two of the three bumps change the lockfile versions of crates that production builds use**. All three are semver-compatible patch/minor bumps inside the existing requirement ranges; none changes a `Cargo.toml` requirement. What has **not** been measured: any behavioral diff in the generated JSON output of `tyrus check --json` caused by `zmij 1.0.23` (the crate formats floats for `serde_json`); `cargo deny --offline check licenses bans sources` on `feat/orc-oracle-lock-verdicts` @ `3950935` printed `bans ok, licenses ok, sources ok` (run 2026-10-22). `cargo deny check advisories` and `cargo audit` need the advisory database (network) and were **not** run in this session, so "no advisory" is **NÃO VERIFICADO** here and is a gate for the owner's acceptance (rerun `scripts/gates.sh deny` and `audit` on that branch).

Options, with consequences:

| Option | What happens | Cost |
|---|---|---|
| **A (recommended, provisional): accept the bumps, record them here** | Lock changes as in the branch. The production snapshot suite (`cargo nextest run --workspace`, including `check --json` snapshots) is the regression detector; the unit's PR must show it green on the exact tree. | Zero code. Two production crates move by a patch/minor version without a review of their changelogs by this ADR. |
| B: split the bumps into their own PR (`chore(deps)` for `zmij`, `num-bigint`, `regex-automata`), merged before the oracle crate | The `jsonschema` PR then only adds packages. Dependabot would propose these bumps anyway. | One extra PR in the merge queue (`Cargo.lock` is a hot file). Rule F1/R11 (one branch, one concern) is honored better. |
| C: pin `jsonschema` to an older release whose minimums fit the current lock | Needs an older `jsonschema` whose `zmij`/`fancy-regex`/`fraction` bounds are satisfied by `zmij 1.0.20`, `regex-automata 0.4.14`, `num-bigint 0.4.6`. Not measured: I did not search for such a version. | Unknown; an old release may lack `validator_for` or the `\p{Cc}` pattern support the unit's tests rely on. |
| D: drop `jsonschema`; validate the lock with a hand-written Rust check plus the schema file reviewed by eye | No lock churn, 30 fewer packages. | The RFC acceptance test (7) asks for validation "with the crate jsonschema"; the schema would no longer be mechanically exercised. Weakens the guard that `cldr` is required. |

Recommendation: **A**, with **B** as the fallback if the owner prefers the strictest reading of R11. Both are reversible. The choice is the owner's; this section only fixes the facts.

Supporting measurements (all from `feat/orc-oracle-lock-verdicts`): workspace packages in `Cargo.lock` 268 -> 298; `tyrus_oracle` normal dependencies are only `serde`, `serde_json`, `thiserror` (and their transitives); `jsonschema` appears only under `[dev-dependencies]`. The transitive `borrow-or-share` (MIT-0) does not trip `cargo deny check licenses` for this dev-dependency (confirmed above on the branch); if `jsonschema` ever becomes a normal dependency, `deny.toml` would reject MIT-0 and this ADR must be revisited (`unit plan` S-4: the owner, not the agent, decides any exception).

### 5. Supply-chain coverage

`cargo deny` and `cargo audit` ([ADR 0011](0011-supply-chain-hygiene.md)) cover `jsonschema` and its transitives because they scan the whole lock. npm lockfiles in `oracle/apps/replica/` are outside `cargo deny`; the replica is not shipped and not part of the Tyrus product, and `dependabot.yml` already has an `npm` ecosystem that can be pointed at the replica directory. That wiring is not done here. **Provisional.**

## Consequences

- Wave-1 PRs can merge once the owner accepts this ADR; the merge queue stays `orc-oracle-lock-verdicts` -> `orc-conformance-batteries` -> `orc-dev-replica-app`.
- Python is dependency-free, so a fresh clone runs the batteries and the replica harness with the system Python 3.12 and the pinned Node only.
- `cargo-mutants` is used by hand for `tyrus_oracle` (the RFC acceptance test (6) asks for zero survivors in `is_green`). It is not a CI gate in wave 1; wiring it is a decision for `gov-gates-toml` / `orc-ledger-ci`. Until then the mutation guarantee is evidence in the unit's PR body, not enforcement.
- Docker is required for the replica end-to-end run; it is not required for any other gate. The `replica-e2e` job detects changes and skips itself when the replica is untouched.
- Dev machine disk is the practical limit: the full gate set needs the free space `ops-disk-capacity` addresses; this ADR changes no gate.
- No equivalence or speed claim follows from this ADR. Ledger: NÃO VERIFICADO.

## Alternativas rejeitadas

| Alternative | Rejected because | Measured fact |
|---|---|---|
| `pytest` | Third-party dependency for a harness whose tests run with `python3 -I -m unittest discover`; the stdlib runner is enough and keeps `-I` isolation meaningful. | batteries and replica tests run under `unittest`. |
| `ruff` | Not present on the dev machine; the CI would need a pinned download; style is already bounded by R4 (function/file/parameter limits) outside Rust. | `ruff` not found on the dev machine. |
| `pip` packages of any kind | `-I` isolates from user site-packages by design; a dependency would reintroduce the ambient state the oracle exists to remove. | all imports in `oracle/` are stdlib. |
| `jsonschema` (Python) for schema validation | Not stdlib; the Rust test uses the Rust crate instead. | n/a |
| A Rust schema validator other than `jsonschema` 0.59.0 | Not evaluated beyond the plan's own build check; option D above is the minimal-dependency alternative. | plan check: `validator_for`/`is_valid` compile and run. |
| `actionlint` as a gate now | Not installed on the owner's machine; adding an unpinned CI download contradicts this ADR's pin policy. | `actionlint` not found. |
| `npm install` / floating dependency ranges in fixtures | Non-reproducible app tree; the replica golden would drift. | replica `package.json` uses exact versions; `package-lock.json` committed. |
| Postgres image by tag (`postgres:17`) | Tags move; the golden depends on the server behavior. | image pinned by digest in `db/replica.env`. |
| Bun or Node 22/24 as oracle | Differences are measured: the batteries must **reject** them (negative controls). | recorded in `oracle/batteries/tests/negative_controls.json`. |

## Gatilhos de revisão

Reopen this ADR (new ADR citing the changed premise) when any of these holds:

- Any pinned tool moves: Node 26.11.1 (the tuple change is a gated event: re-run the fingerprint, re-record the corpus), `cargo-mutants` 27.1.0, `jsonschema` 0.59.0, Postgres digest, Python minimum 3.12.
- `jsonschema` becomes a normal dependency, or `cargo deny` rejects a license in its tree (MIT-0 today).
- `cargo audit` or `cargo deny` reports an advisory against `jsonschema`, `zmij`, `num-bigint` or `regex-automata`.
- The `Cargo.lock` bumps in Decision 4 produce any diff in `tyrus check --json` output or in a snapshot.
- A harness needs a third-party Python package, or a second Python minor version in CI.
- CI gains a pinned `shellcheck`/`actionlint`, or `cargo-mutants` becomes a CI gate (update Decision 1 rows c and h).
- The replica leaves tmpfs/Docker (for example a hosted Postgres) or is shipped outside `oracle/`.

## References

- [ADR 0014](0014-development-flow-rules.md) — F9: adopting tooling requires an ADR.
- [ADR 0011](0011-supply-chain-hygiene.md) — `cargo deny`, `cargo audit`, Dependabot and the license allowlist this ADR relies on.
- [ADR 0015](0015-rejected-tooling.md) — tooling already evaluated and rejected; this ADR does not reopen any row.
- Evidence read for this ADR: `oracle/oracle.lock`, `crates/tyrus_oracle/Cargo.toml`, `Cargo.lock` diff of `feat/orc-oracle-lock-verdicts`; `oracle/batteries/goldens/node-26.11.1/meta.json` of `feat/orc-conformance-batteries` (sha256 `a673ef27352206371ad9c7adc104166d61d79c23d9b3e471cec37346399e0f3f`); `oracle/apps/replica/package.json` and `db/replica.env` of `feat/orc-dev-replica-app`; `~/.local/share/tyrus-spikes/sp2-census/package.json` and `~/.local/share/tyrus-spikes/sp4-profile/results/meta.json`.
