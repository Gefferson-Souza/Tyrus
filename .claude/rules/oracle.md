---
paths:
  - "crates/tyrus_oracle/**"
  - "oracle/**"
  - "schemas/**"
---

# Oracle Rules (binding: D14, D21, I7)

- `Verdict` has ONE green set: `Identical` and `Normalized(id)`. `Verdict::class` is an
  exhaustive `match` with no wildcard arm; a new variant forces an explicit decision.
- Registries (`Normalizer`, `Assumption`) are closed. A new id is a gated event (D14): it
  changes the registry, `schemas/verdict.schema.json` and the ratchet, and needs the
  owner's review.
- `oracle/oracle.lock` pins the Node tuple (version, V8, ICU, Unicode, tz, CLDR). Moving any
  field is a gated event: re-run the fingerprint, re-record the corpus, port what changed.
  The file is canonical JSON (a test compares it byte for byte).
- The Node binary comes from `TYRUS_NODE_REFERENCE` / `TYRUS_NODE_SEED` (absolute path).
  Oracle tests that run Node FAIL when the pins are missing; never turn that into a skip.
- Error codes live in the `TYRUS-E5xxx` block (`E4xxx` is formatting, `E6xxx` is the
  ledger). Codes are never reused (R14).
- `tyrus_oracle` depends on no legacy crate; tests may read legacy `src/` as text only.
- No equivalence or speed claim from this crate without a ledger id or "NÃO VERIFICADO".
