# SQL wire equivalence: bun-express

Verdict: **PASS*** over 54 canonical requests (round-trip count, protocol, statement text, text parameters).

## Accepted wire differences observed

- binary result format: 0 round trips
- describe statement instead of portal: 0 round trips
- parse OIDs present: 0 round trips
- binary parameters (not compared): 0 round trips

## Documented divergences (harness/known_divergences.json)

- 52-post-valid-lone-surrogate trip 2: parameters differ: expected ['TST-0009', 'a�b', '3', '1', '4'] observed ['TST-0009', 'a�', '3', '1', '4']

## Failures

None.
