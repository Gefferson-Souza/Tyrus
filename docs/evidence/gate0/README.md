# Gate 0 evidence (external)

Archive of the M0 Gate 0 ceiling run closed on 2026-10-20. The files are byte-for-byte copies of the run output; no number is restated in prose here. `MANIFEST.json` records origin path, size and sha256 per file, the runtime tuple, and the sha256 declared in `summary.md` for the two Rust ceiling binaries (binaries are not copied; `local_check` records whether the local build still matches).

Ledger: NOT VERIFIED (no ledger exists yet). The ledger importer (`orc-ledger-ci`) consumes `MANIFEST.json` as `external_evidence`.

Verify: `cd docs/evidence/gate0 && sha256sum -c MANIFEST.sha256`, or `sh scripts/evidence/check_gate0.sh` for the full acceptance set. Regenerate: `python3 -I scripts/evidence/gen_gate0_manifest.py`.
