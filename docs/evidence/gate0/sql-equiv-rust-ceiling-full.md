# SQL wire equivalence: rust-ceiling-full

Verdict: **PASS** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).

## Accepted wire differences observed

- binary result format: 44 round trips
- describe statement instead of portal: 44 round trips
- parse OIDs present: 0 round trips
- binary parameters (not compared): 0 round trips

## Documented divergences (harness/known_divergences.json)

None.

## Failures

None.
