# 18. G1 Non-Inferiority Margin (`g1_margin`)

Date: 2026-10-22
Status: Proposto — awaiting owner acceptance. The value `1.05` is **provisional** (owner instruction of 2026-10-20: "go on, we change it later if needed"); it is recorded here so that `gates.toml` can read it, and it freezes only at the first hosted measurement (see Decision, point 4).

## Context

The PPR foundation (ADR 0016) promises a hosted floor whose cost per request is no worse than the reference Node on the same application. Gate G1 was written in the foundation record as "upper limit of the 95% confidence interval (IC95) of the paired ratio of medians <= 1.0". That literal cannot pass at exact parity:

- A target statistically identical to the baseline has a true ratio of 1.0. The upper limit of a 95% interval around an estimate of 1.0 is above 1.0 with high probability by construction. The synthetic illustration below shows `P(upper <= 1.00) = 0.003` for an identical target at a per-round noise of 0.02 (see Evidence).
- To pass the literal gate the floor would need to be strictly faster than Node. The only floor-like proxy measured so far (Deno 2.9.7, no database route) cost 1.14 to 1.24x the CPU per request of Node 26.8.1 (foundation record, section 0, check VF1; not re-measured here and not a measurement of the future platform).
- Choosing the tolerance after seeing the hosted data is the opposite of what the spikes SP2 and SP4 do (limits fixed before running). The tolerance has to be decided **before** the first hosted measurement.

Gate G1 feeds the M1 exit criterion E5 (RFC `program-rfc.md`, section 2.1) and the abandonment criterion K1. `gov-gates-toml` cannot be merged until this value is decided.

## Decision

1. **G1 is amended** from "IC95 upper <= 1.0" to "IC95 upper <= `thresholds.g1_margin`" against the same Node line (Node 26.11.1, same Express app), on the database routes E2, E3, E4 and E6, which are the blocking scope. The foundation record also lists the `perf-ts` loads in pure hosted mode under G1; this ADR does not narrow that silently: following the recommendation of RFC Q-C, those loads are measured and listed (a load above the margin is a `FAIL` that restricts the claim and does not block M1), and that scope depends on the owner's confirmation of Q-C (see Perguntas ao dono). Estimator unchanged: ratio of paired medians, ABAB rounds, >= 15 rounds, disjoint CPUs, CPU per request = utime + stime, paired bootstrap with 10000 resamples and fixed seed 20261020.
2. **Provisional value: `g1_margin = 1.05`.** Allowed range for the A/A-validated choice: 1.03 to 1.10. A value outside the range needs a superseding ADR citing the A/A evidence.
3. **A/A acceptance is mandatory.** `bench-g1-parity-harness` must run Node 26.11.1 Express against itself for >= 15 ABAB rounds on the adopted rig, and the IC95 upper of that run must be <= `g1_margin`. If it is not, the margin is unreachable by noise: the rig or the number of rounds does not serve the gate, and the delivery loop stops for the owner with the observed IC95 (it does not widen the margin on its own).
4. **Immutability.** `g1_margin` and the number of rounds are fixed by a commit that precedes the first hosted measurement (the first `raw.jsonl` whose target is the hosted artifact; A/A runs of Node against Node do not count). The measurement header of each `raw.jsonl` records `g1_margin`, `g1b_upper`, the number of rounds, the seed and the number of resamples, and `m1-g1-parity` fails if those values, read from `gates.toml` at judgement time, differ from the header. The hash of the whole `gates.toml` is deliberately not the criterion: the file also carries `[legacy]`, the `state` and `activates_in` of every gate, `[claims]` and the M2 ratios, which other units edit between the first measurement and `m1-g1-parity`. After the first hosted measurement the only way to change the value is a superseding ADR, and the hosted measurement is repeated from scratch.
5. **G1b stays literal at 1.0.** G1b (IC95 upper <= 1.0 against the lowest of Node 24 and 26 x Express and Fastify) remains a report of gain, does not block M1, and without a green G1b any speed claim for the floor stays forbidden. The margin never becomes a claim: a pass at `g1_margin` supports "no worse than the margin", never "faster" or "equal".
6. **Consequence of a miss.** A database route or load with IC95 upper above `g1_margin` is listed as `FAIL` in the ledger and opens the 15-day adjustment window of K1; after it, the loop stops for the owner (stop, or downgrade to "oracle + receipts" without a speed claim). libnode is not a remedy (about 1.0x by construction).

## Evidence

Synthetic illustration, **not a performance measurement**: the script draws paired rounds from an assumed noise model (lognormal, per-round shared drift sd 0.05, independent noise sd as listed, 15 rounds) and applies the estimator of point 1 (2000 resamples instead of 10000 to bound the run to about one minute; 400 replications per cell; seed 20261020). The table gives the probability that `upper <= margin` for a given true ratio. The noise levels are assumptions: the real rig noise is **not measured yet** (that is what the A/A acceptance does).

Script: `g1_margin_sim.py`, sha256 `1d0c83661097e76ef847e79561f112f9f23646b9b876dc0f53979e38e40abd00`; output `g1_margin_sim.out`, sha256 `31d349b0eb0d1b69705fab2e125a92b1d562703e73432ab2d683cada43fa381a` (reproduced twice with identical output; the boundary `upper == margin` passing is covered by a self-test, and replacing `<=` by `<` makes the self-test fail). The files live outside the repository for now (`~/.local/share/tyrus-wt/gov-constitution-adr-artifacts/`); `bench-g1-parity-harness` re-implements the estimator in `tyrus_oracle::stats` and replaces this illustration with the real A/A run.

| true ratio | noise sd | upper <= 1.00 | upper <= 1.03 | upper <= 1.05 | upper <= 1.10 |
|---|---|---|---|---|---|
| 1.00 (A/A) | 0.01 | 0.000 | 0.925 | 1.000 | 1.000 |
| 1.00 (A/A) | 0.02 | 0.003 | 0.515 | 0.930 | 1.000 |
| 1.00 (A/A) | 0.03 | 0.000 | 0.305 | 0.780 | 1.000 |
| 1.00 (A/A) | 0.05 | 0.005 | 0.128 | 0.420 | 0.955 |
| 1.05 | 0.02 | 0.000 | 0.000 | 0.003 | 0.907 |
| 1.08 | 0.02 | 0.000 | 0.000 | 0.000 | 0.190 |
| 1.10 | 0.02 | 0.000 | 0.000 | 0.000 | 0.003 |

What the illustration says, and no more: (a) the literal 1.0 fails A/A in practice at every assumed noise level; (b) at `1.05` an A/A run passes reliably only when the per-round noise is about 0.02 or lower, so the A/A acceptance is a real filter and may fail on a noisy rig; (c) at `1.10` the gate also passes a target whose true ratio is 1.05 about nine times out of ten at noise 0.02, which is the price of the looser margin.

## Alternativas rejeitadas (alternatives rejected)

| Alternative | Rejected because | Consequence if chosen |
|---|---|---|
| Keep the literal 1.0 | Unsatisfiable at parity (table, row 1.00 A/A). The floor would need to be strictly faster than Node to pass, contradicting "floor is not a speed gain" (foundation record, section 6). | K1 triggers by construction; the M1 exit can only be met by a faster-than-Node floor. |
| Point estimate <= 1.0 without interval | Noise decides the verdict; an A/A run passes about half the time. | Flaky gate; receipts lose meaning. |
| Symmetric equivalence test (TOST) | We care only about non-inferiority; a faster floor is not a defect. | Rejects a target that is better than Node. |
| Choose the margin after the hosted data | Tuning the tolerance to the result is what SP2 and SP4 forbid. | The gate can be made to pass; it proves nothing. |
| `1.03` | At A/A noise 0.02 an identical target passes only 0.515 of the time (synthetic). | High chance that A/A fails and stops the loop for noise reasons. |
| `1.10` | Passes a true 1.05 target 0.907 of the time at noise 0.02; 10% extra CPU per request is hard to defend as "no worse". | Gate too permissive for the floor's own claim; use only if A/A on the real rig shows `1.05` unreachable and the owner accepts it by superseding ADR. |

## Consequences

- `gates.toml` (unit `gov-gates-toml`) reads `thresholds.g1_margin = 1.05` (provisional) and `g1b_upper = 1.0`; `cfg.py validate` rejects a file without `g1_margin >= 1.0`.
- `bench-g1-parity-harness` gains the A/A acceptance (RFC acceptance test 4) and the boundary test `upper <= margin` with a mutation check against `<`.
- The M1 exit criterion E5 and the claim vocabulary use "no worse than the margin" for G1; docs must not say "faster" without G1b.
- Cost: a pass at the margin can hide a real regression of up to the margin. Accepted: it is the price of a satisfiable gate, and G1b still reports the gain.

## Gatilhos de revisão (review triggers)

- The A/A run on the real rig fails at `g1_margin`: stop, report the observed IC95, owner decides (rig, rounds, or superseding ADR).
- The owner replaces the provisional `1.05` before the first hosted measurement: update this ADR (or supersede it) and `gates.toml` in the same PR.
- The estimator or the node-line reference changes (`oracle.lock` event): re-validate the A/A acceptance.

## Perguntas ao dono (owner questions)

1. Q-B: accept `g1_margin = 1.05` as the value to freeze before the first hosted measurement (currently provisional), or fix another value in 1.03 to 1.10?
2. Q-C: confirm that only the database routes E2, E3, E4 and E6 block M1, with the `perf-ts` loads (35 programs in about 17 families, no file enumerates 14) measured and listed, or require them to block as the foundation record wrote?

## References

- ADR 0016 — the foundation and constitution this gate belongs to (Constitution R5).
- `program-rfc.md` (local plan, not versioned; sha256 `5f5be5ec2df074b2602744225f39144f309634a933db9bcb048a96bc4735a961`), section 2.1 E5, section 11 Q-B, units `bench-g1-parity-harness`, `m1-g1-parity`.
- Foundation decision record (local plan, not versioned; sha256 `128b55c16df803204c7257992cc3129cd89bce5b7d5a741169cbbb16e3e271ae`), sections 6 and 10.
- Owner decisions: `decisoes-provisorias-2026-10-20.md` (Q-B provisional).
