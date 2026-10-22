# 19. Oracle and Harness Tooling

Date: 2026-10-22
Status: Proposto

## Context

The execution program (`program-rfc.md`, wave 1) adds a verification layer that did not exist in the repository: the oracle (`crates/tyrus_oracle`, `oracle/oracle.lock`), the conformance batteries (`oracle/batteries/`) and the dev replica app (`oracle/apps/replica/`). Rule F9 ([ADR 0014](0014-development-flow-rules.md)) requires an ADR before adopting tooling. Three wave-1 units carry `merge_after: adr-oracle-tooling` for that reason: `orc-oracle-lock-verdicts`, `orc-conformance-batteries` and `orc-dev-replica-app`.

This ADR records what the wave-1 branches **actually use**, not what the plans promised. Measured on 2026-10-22, before any rebase, on three local branches that are **not pushed** (so their SHAs are dated snapshots, not durable references): `feat/orc-oracle-lock-verdicts` @ `3950935`, `feat/orc-conformance-batteries` @ `4bc5667`, `feat/orc-dev-replica-app` @ `8d348ba`; and on the two out-of-repo spikes `sp2-census` and `sp4-profile` under `~/.local/share/tyrus-spikes/`. The durable references are the file paths and hashes listed in "References"; once the branches merge, the merged tree is the authority and this paragraph is history. Where the plans and the branches differ, the difference is stated in "Decision 4".

Two facts that live in local, git-ignored planning files (`.claude/plan/` is in `.gitignore`) are copied here so that the versioned record does not depend on them: (1) the provisional policy of 2026-10-20 (owner: "continua fazendo, depois mudamos caso seja necessário"): recommendations of the program RFC are provisional until the owner reviews them, all reversible, and the owner keeps the ADRs 0016, 0018 and 0019; (2) the minimum bounds of the `Cargo.lock` bumps in Decision 4, which were read in the crates' own `Cargo.toml`.

Status is **Proposto**: the owner accepts. Choices that rest on `decisoes-provisorias-2026-10-20.md` ("continua fazendo, depois mudamos") are marked **provisional**; every one is reversible by `git revert` plus a rerun of the affected unit.

## Decision

### 1. Tools adopted, with exact versions

The **Pin** column says how each version is held, because "measured on the author's machine" and "pinned in the repository" are different guarantees:

- **repo-exact**: a committed file fixes the version (`oracle/oracle.lock`, `package-lock.json`, `replica.env`, `Cargo.lock`).
- **local**: observed on the developer machine only; nothing in the repository or in CI enforces it.
- **CI-floating**: CI installs whatever the tag or runner image currently provides.

| # | Tool | Version | Pin | Used by | Scope | Evidence |
|---|---|---|---|---|---|---|
| a | Python 3 standard library, always `python3 -I` | tested with **3.12.3** (`/usr/bin/python3`); no minimum is declared or checked anywhere | local; CI-floating (the runner's `python3`; its version was **not verified**) | batteries runner, replica harness, spike scripts | `oracle/`, `bench/`; zero `pip` dependencies. Imports measured: `argparse`, `difflib`, `hashlib`, `http.server`, `json`, `subprocess`, `unittest`, `fcntl` and the like. `fcntl` is Unix-only, which matches the CI runner and the dev machine. | `python3 --version` on the developer PATH is 3.15.0 (linuxbrew), `/usr/bin/python3` is 3.12.3; which interpreter produced each branch's results was **not recorded**. No branch has `sys.version_info`, `requires-python` or `setup-python`. Recommendation (not implemented here): the harness fails fast when `sys.version_info < (3, 12)`. |
| b | Node, pinned exactly | **26.11.1** reference (V8 14.6.202.34-node.37, ICU 78.3, Unicode 17.0, tz 2026e, CLDR 48.0); **22.21.1** seed (V8 12.4.254.21-node.33, ICU 77.1, tz 2025b, CLDR 47.0) | repo-exact (`oracle/oracle.lock`, `actions/setup-node@v7` with the exact version) | probes, ST0, batteries goldens, replica | Tuple pinned in `oracle/oracle.lock`; located through `TYRUS_NODE_REFERENCE` / `TYRUS_NODE_SEED` (gate `node-pins` fails when unset; never a skip). | `oracle/oracle.lock` and `ci.yml` of `feat/orc-oracle-lock-verdicts`; goldens `meta.json` (sha256 in "References"). |
| c | `cargo-mutants` | **27.1.0** | local (installed by `cargo binstall`) | mutation check of `tyrus_oracle` (`is_green`) | Development tool. No branch has a `cargo-mutants` config or a CI step; see "Divergências do RFC". | Only the wave-1 plan of the unit (local, ignored) mentions it; **not verified in any branch**. |
| d | `jsonschema` crate | **0.59.0**, `default-features = false` | repo-exact through `Cargo.lock` (the `Cargo.toml` requirement is the caret range `0.59.0`) | `tyrus_oracle` tests (`oracle.lock` and verdict schemas) | **dev-dependency only**; the schema must require `cldr`. | `crates/tyrus_oracle/Cargo.toml` line 26 of `feat/orc-oracle-lock-verdicts`. |
| e | npm lockfile policy | npm 11.20.0 (bundled with Node 26.11.1) | repo-exact for packages (`package-lock.json`, `npm ci`); npm itself follows the Node pin | replica app, `sp2-census` | Exact versions and integrity, never `npm install`. Replica pins Nest 10.4.22, TypeORM 0.3.31, pg 8.23.1, TypeScript 5.1.6, webpack 5.112.0. `sp2-census` pins acorn 8.19.0, typescript 5.9.3, resolve.exports 2.0.4. | `oracle/apps/replica/package.json` / `package-lock.json` of `feat/orc-dev-replica-app`; `sp2-census/package.json`. `npm --version` on the dev machine: 11.20.0. |
| f | Docker, and Postgres by digest | Docker 29.9.1 (local); image `postgres:17-alpine` (PostgreSQL **17.11**) at digest `sha256:b0f9560a2de083e2cc7382e75f808c7381a32852a7ec49117deedb300e552b24` | digest: repo-exact (`db/replica.env`); Docker engine: local / CI-floating (the runner image's Docker) | replica database | Container on tmpfs, `127.0.0.1` only. | `docker image ls --digests` shows `postgres 17-alpine` at that digest; `postgres --version` in the image prints 17.11. |
| g | `cargo-nextest` | 0.9.143 | local; **CI-floating** (`taiki-e/install-action@nextest`) | already the test runner | Listed for completeness; the oracle tests that run the real Node run under it (the branch's last commit serializes the probe tests). | `cargo nextest --version` = 0.9.143 (2026-08-04); `ci.yml` of the three branches. |
| h | `shellcheck` | **CI-floating** (whatever `ubuntu-latest` ships; absent on the dev machine) | CI-floating | `scripts/gates.sh replica` with `TYRUS_REQUIRE_SHELLCHECK=1` | See Decision 3. | `scripts/gates.sh` lines 247-254 and `ci.yml` of `feat/orc-dev-replica-app`. |
| i | Bun | 1.2.22 | local | negative control only (batteries must flag `sin`/`cos` differences) | Never an oracle; never in CI. | `oracle/batteries/tests/negative_controls.json` of `feat/orc-conformance-batteries`; `bun --version` = 1.2.22. |

The negative-control engines of the batteries (Node 22.21.1, Node 24.21.0, Bun 1.2.22) and the 16 mutant preloads are test fixtures for the batteries, not oracles. The single reference is the Node 26.11.1 tuple. Only rows b, d, e and the digest of f are enforced by the repository; rows a, c, g, h and the engine of f are not, and this ADR does not pretend otherwise.

### 2. Tools explicitly not adopted

`pytest` and `ruff` as required dependencies; `actionlint` (until it exists on the owner's machine and a version is pinned). Reasons in "Alternativas rejeitadas".

### 3. Pinning policy for tools that are not yet pinned

The wave-1 CI references third-party actions by tag (`actions/setup-node@v7`, `actions/checkout@v7`), as the rest of the repository already does. `shellcheck` on `ubuntu-latest` is whatever the runner image ships. Neither is a version pin in the sense this ADR demands for Node and `jsonschema`. **Recommendation:** keep the current action tags (consistent with the existing workflows) and accept `shellcheck` as floating, with a revision trigger (below) instead of a pin. Pinning `shellcheck` (and `actionlint`, if adopted) is a CI-supply-chain change; **no unit currently owns it**: the RFC scope of `gov-supply-chain-policy` lists `audit.toml`, `deny.toml` and the direction rule, not `shellcheck`, and [ADR 0011](0011-supply-chain-hygiene.md) Policy 4 is about CODEOWNERS membership, not tool pinning. The owner either assigns it to `gov-supply-chain-policy` (amending its scope) or accepts the floating `shellcheck`. **Provisional.**

### 4. Decision point: the `Cargo.lock` bumps forced by `jsonschema`

`orc-oracle-lock-verdicts` states in its plan that `Cargo.lock` changes are "só adições". The branch actually contains 30 added packages **and three version bumps of crates already in the lock**:

| Crate | Before | After | Forced by (minimum bound read in the crate's `Cargo.toml`) | In the production graph? (`cargo tree --locked -e normal,build`) |
|---|---|---|---|---|
| `zmij` | 1.0.20 | 1.0.23 | `jsonschema-value 0.59.0` requires `zmij = "1.0.23"` | **Yes.** `zmij` is a dependency of `serde_json`; it reaches `tyrus_analyzer`, `tyrus_orchestrator`, `tyrus_cli` and `tyrus_oracle`. |
| `num-bigint` | 0.4.6 | 0.4.8 | `fraction 0.17.0` requires `num-bigint = "0.4.8"` | **Yes.** Reaches the SWC parser chain (`tyrus_parser`, `tyrus_analyzer`, `tyrus_cli`). |
| `regex-automata` | 0.4.14 | 0.4.18 | `fancy-regex 0.19.2` requires `regex-automata = "0.4.16"` | No normal/build path; dev graph only (`assert_cmd` via `bstr`, and `jsonschema`). |

The consequence the plan did not state: although `jsonschema` is a dev-dependency, `Cargo.lock` is workspace-wide, so **two of the three bumps change the lockfile versions of crates that production builds use**. All three are semver-compatible patch/minor bumps inside the existing requirement ranges; none changes a `Cargo.toml` requirement. What has **not** been measured: any behavioral diff in the generated JSON output of `tyrus check --json` caused by `zmij 1.0.23` (the crate formats floats for `serde_json`).

`cargo deny --offline check licenses bans sources` on `feat/orc-oracle-lock-verdicts` @ `3950935` printed `bans ok, licenses ok, sources ok` (2026-10-22), but **that result says nothing about the new dev-only packages**: see Decision 5. `cargo deny check advisories` and `cargo audit` need the advisory database (network) and were **not** run in this session, so "no advisory" is **NÃO VERIFICADO** here. Because `cargo audit` is the only check that reads the whole lock (Decision 5), running it on that branch is a gate for the owner's acceptance.

Options, with consequences:

| Option | What happens | Cost |
|---|---|---|
| **A (recommended, provisional): accept the bumps, record them here** | Lock changes as in the branch. The production snapshot suite (`cargo nextest run --workspace`, including `check --json` snapshots) is the regression detector; the unit's PR must show it green on the exact tree. | Zero code. Two production crates move by a patch/minor version without a review of their changelogs by this ADR. |
| B: split the bumps into their own PR (`chore(deps)` for `zmij`, `num-bigint`, `regex-automata`), merged before the oracle crate | The `jsonschema` PR then only adds packages. Dependabot would propose these bumps anyway. | One extra PR in the merge queue (`Cargo.lock` is a hot file). Rule F1/R11 (one branch, one concern) is honored better. |
| C: pin `jsonschema` to an older release whose minimums fit the current lock | Needs an older `jsonschema` whose `zmij`/`fancy-regex`/`fraction` bounds are satisfied by `zmij 1.0.20`, `regex-automata 0.4.14`, `num-bigint 0.4.6`. Not measured: I did not search for such a version. | Unknown; an old release may lack `validator_for` or the `\p{Cc}` pattern support the unit's tests rely on. |
| D: drop `jsonschema`; validate the lock with a hand-written Rust check plus the schema file reviewed by eye | No lock churn, 30 fewer packages. | The RFC acceptance test (7) asks for validation "with the crate jsonschema"; the schema would no longer be mechanically exercised. Weakens the guard that `cldr` is required. |

Recommendation: **A**, with **B** as the fallback if the owner prefers the strictest reading of R11. Both are reversible. The choice is the owner's; this section only fixes the facts.

Supporting measurements (all from `feat/orc-oracle-lock-verdicts`): workspace packages in `Cargo.lock` 268 -> 298; `tyrus_oracle` normal dependencies are only `serde`, `serde_json`, `thiserror` (and their transitives); `jsonschema` appears only under `[dev-dependencies]`. `borrow-or-share` (MIT-0) is a transitive of `jsonschema`. Whether `deny.toml` accepts MIT-0 is **not tested by the current `licenses ok`**, because `cargo deny` does not evaluate that package at all (Decision 5). If `jsonschema` became a normal dependency, the license question would have to be answered then, and this ADR revisited (the unit plan's stop condition S-4: the owner, not the agent, decides any exception).

### 5. Supply-chain coverage

**Measured gap.** `cargo deny` in this repository does **not** evaluate the dev-only tree that `jsonschema` brings. Experiment (2026-10-22, `feat/orc-oracle-lock-verdicts` @ `3950935`, offline, with a throwaway config `[licenses] version = 2, allow = []` so that every evaluated package is rejected): `cargo deny --offline --config <file> check licenses` printed 243 rejected packages (`error[rejected]`), including `zmij-1.0.23`, `num-bigint-0.4.8`, `tempfile` and `walkdir`, and **no entry** for `jsonschema 0.59.0`, `fluent-uri`, `referencing`, `fraction` or `borrow-or-share`. `cargo deny --offline list` confirms it: `jsonschema` and `borrow-or-share` are absent from the license listing. The names only appear as parents in the inverse trees of other rejections. `criterion`, which is dev-only too, is also not evaluated. The exact mechanism inside `cargo deny` was not determined; `bans` and `advisories` use the same graph but were not measured separately.

Consequences, stated plainly:

- `cargo audit` reads the whole `Cargo.lock` and is the **only** check that covers `jsonschema` and its transitive packages. It was not run here (needs the network).
- The license and ban policies of `deny.toml` are **not enforced** on those packages. The `licenses ok` of the branch confirms nothing about them, including `borrow-or-share` (MIT-0).
- The premise "covered by `cargo deny`" in the program RFC is therefore not true for this tree. The owner either accepts the gap (dev-only, never shipped) or closes it in `gov-supply-chain-policy`; this ADR does not change `deny.toml`.
- npm lockfiles in `oracle/apps/replica/` are outside both tools; the replica is not shipped and not part of the Tyrus product, and `dependabot.yml` already has an `npm` ecosystem that can be pointed at the replica directory. That wiring is not done here.

**Provisional.**

## Divergências do RFC

`program-rfc.md` section 4.3 (`adr-oracle-tooling`) says one thing in three places and this ADR decides differently. The owner accepts the differences or has the RFC realigned:

1. `shellcheck` and `actionlint` "só no CI, com versão fixada": here `shellcheck` is accepted as CI-floating (Decision 3) and `actionlint` is not adopted.
2. `cargo-mutants` as "ferramenta de CI e de desenvolvimento": here it is development-only; wave 1 wires nothing in CI (Consequences).
3. `jsonschema` "coberto por `cargo deny`": measured false for the dev-only tree (Decision 5).

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

- Any pinned tool moves: Node 26.11.1 (the tuple change is a gated event: re-run the fingerprint, re-record the corpus), `cargo-mutants` 27.1.0, `jsonschema` 0.59.0, Postgres digest, the Python version the harnesses were tested with (3.12.3).
- `jsonschema` becomes a normal dependency, or `cargo deny list` starts listing `jsonschema` (the gap in Decision 5 closed or the graph changed), or `scripts/gates.sh deny` starts to print a license verdict for its tree (MIT-0 for `borrow-or-share` is the known candidate).
- `shellcheck` on the runner changes its major version, or a CI failure is traced to the floating version.
- `cargo audit` or `cargo deny` reports an advisory against `jsonschema`, `zmij`, `num-bigint` or `regex-automata`.
- The `Cargo.lock` bumps in Decision 4 produce any diff in `tyrus check --json` output or in a snapshot.
- A harness needs a third-party Python package, or a second Python minor version in CI.
- CI gains a pinned `shellcheck`/`actionlint`, or `cargo-mutants` becomes a CI gate (update Decision 1 rows c and h).
- The replica leaves tmpfs/Docker (for example a hosted Postgres) or is shipped outside `oracle/`.

## References

- [ADR 0014](0014-development-flow-rules.md) — F9: adopting tooling requires an ADR.
- [ADR 0011](0011-supply-chain-hygiene.md) — `cargo deny`, `cargo audit`, Dependabot and the license allowlist this ADR relies on.
- [ADR 0015](0015-rejected-tooling.md) — tooling already evaluated and rejected; this ADR does not reopen any row.
- Evidence read for this ADR (file paths on the branches named in Context, dated 2026-10-22):
  - `oracle/oracle.lock`, `crates/tyrus_oracle/Cargo.toml`, `ci.yml` and the `Cargo.lock` diff of `feat/orc-oracle-lock-verdicts`;
  - `oracle/batteries/goldens/node-26.11.1/meta.json` of `feat/orc-conformance-batteries` (sha256 `a673ef27352206371ad9c7adc104166d61d79c23d9b3e471cec37346399e0f3f`) and `oracle/batteries/tests/negative_controls.json`;
  - `oracle/apps/replica/package.json`, `db/replica.env`, `scripts/gates.sh` of `feat/orc-dev-replica-app`;
  - `~/.local/share/tyrus-spikes/sp2-census/package.json`, `~/.local/share/tyrus-spikes/sp2-census/report.md` (sha256 `2c851da31c747d99a2b0f97e71def2f3bf8a86ae6eddd3868c3155dfaa3163b8`) and `~/.local/share/tyrus-spikes/sp4-profile/results/meta.json`.
- Not verified in any branch: the `cargo-mutants` usage (row c), the Python version used by each run (row a).
