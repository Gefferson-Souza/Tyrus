# 15. Tooling Evaluated and Rejected

Date: 2026-08-04
Status: Accepted. Miri, sanitizers and cargo-careful reopened for allowlisted crates by [ADR 0016](0016-ppr-foundation-and-constitution.md), effective when ADR 0016 is accepted.

## Context

The 2026-08 standardization campaign (ADRs 0013/0014) included a research pass over current Rust quality tooling: Rust API Guidelines, the Safety-Critical Rust Consortium's coding guidelines, clippy lint groups, property/mutation/fuzz testing practice, and supply-chain tooling. Several well-regarded tools were evaluated and deliberately **not** adopted.

Recording rejections matters as much as recording adoptions: without this ADR, every future audit or contributor re-litigates the same decisions from scratch. Each rejection below states its condition for reopening — a rejection is only valid while its premise holds.

## Decision

The following tools are rejected for this project. Reopening any of them requires a new ADR citing a changed premise.

| Tool | Rejected because | Reopen when |
|---|---|---|
| **Ferrocene** (qualified toolchain) | Its value is the certification paper trail (ISO 26262 / IEC 61508) for regulated industries. Tyrus is an academic transpiler; no certification target exists. | The project targets a certified/regulated deployment. |
| **Miri** | Detects undefined behavior in `unsafe` code. With R13 (`#![forbid(unsafe_code)]` workspace-wide, ADR 0013) there is structurally nothing for Miri to find; runs cost 10–100× test time. | Any crate legitimately needs `unsafe` (which itself requires an ADR amending R13). |
| **Sanitizers (ASan/TSan/MSan)** | Same premise as Miri: they target memory/thread bugs that safe Rust prevents. The workspace has no `unsafe` and no exotic concurrency (single Mutex protocol, ADR 0009). | `unsafe` enters the workspace, or concurrency grows beyond the ADR 0009 pattern. |
| **cargo-careful** | Marginal UB-adjacent checking on top of safe Rust; same premise as Miri, weaker payoff. | Same as Miri. |
| **cargo-vet / cargo-crev** | Audit-trail systems designed for organizations with regulatory supply-chain requirements and audit-sharing networks. For a single-maintainer project, `cargo-deny` (licenses/bans/sources) + `cargo-audit` + Dependabot + `--locked` (R12) cover the realistic threat model; maintaining `audits.toml` would be ceremony without a second consumer. | The project gains external contributors/consumers with supply-chain requirements. |
| **clippy `restriction` as a group** | The restriction group is explicitly not designed to be enabled wholesale — it contains mutually contradictory lints. Individual restriction lints are cherry-picked instead (R6: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `indexing_slicing`, `string_slice`, `unwrap_in_result`; plus `dbg_macro`, `print_stdout`/`print_stderr`, `exit`, `mem_forget` via #215). | Never as a group; individual lints may be added anytime via the `[workspace.lints]` table. |
| **quickcheck** (vs proptest) | Semi-maintained; weaker shrinking and no composable strategies. The planned property-testing work (post-RFC testing phase) standardizes on **proptest**. | proptest becomes unmaintained. |

## Consequences

- Future audits check this table before proposing tooling; proposals that ignore it are returned with a pointer here.
- The rejections tied to R13 make the `forbid(unsafe_code)` rule load-bearing beyond style: it is the premise that keeps the verification toolchain small. Weakening R13 invalidates three rows of this table and requires revisiting them in the same ADR.
- The project accepts the residual risks explicitly: no UB detection (mitigated structurally by R13), no distributed dependency audits (mitigated by R12's audit stack).

## References

- [ADR 0013](0013-power-of-ten-v2.md) — R13 and the amendment campaign this decision belongs to.
- [ADR 0011](0011-supply-chain-hygiene.md) — the adopted supply-chain stack.
- [clippy lint groups](https://doc.rust-lang.org/stable/clippy/lints.html) — restriction-group guidance.
- Microsoft Rust engineering practices (Miri/sanitizers applicability), Rust Project Primer (audit tooling) — research inputs to the campaign.
