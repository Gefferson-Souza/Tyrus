---
name: gates
description: Run the Tyrus quality gates (scripts/gates.sh) and interpret failures. Use before any push (F4), after implementation slices, or when CI is red and you need the local reproduction.
---

# /gates — run and interpret the quality gates

1. `export PATH="$HOME/.cargo/bin:$PATH"` (cargo may be absent from non-interactive shells).
2. Prefer the staged sequence over one monolithic run (long single processes have been
   killed on this machine; F4 permits staging when every gate passes on the same tree):
   - `./scripts/gates.sh fmt && ./scripts/gates.sh clippy && ./scripts/gates.sh filesize && ./scripts/gates.sh unsafe && ./scripts/gates.sh machete`
   - `cargo build --workspace --all-targets --locked` (CLI tests need the `tyrus` bin)
   - `./scripts/gates.sh test`
   - `./scripts/gates.sh deny && ./scripts/gates.sh audit`
   - `./scripts/gates.sh coverage`
3. **Never read pass/fail through a pipe** (`| tail` masks exit codes — a repeated
   incident here). Check `$?` of the gate command itself, or run it bare.
4. On failure, fix locally — CI is a verifier, never a debugger (F4). Known failure
   modes: missing `tyrus` binary → build first; ENOSPC → delete
   `$TMPDIR/tyrus_test_target`, `target/llvm-cov-target` (rebuildable), check `df -h /`;
   new RUSTSEC advisory → `cargo update -p <crate>`; new toolchain lint → fix code,
   never blanket-allow.
5. Report the per-gate verdicts and the coverage TOTAL line, with exit codes observed.

$ARGUMENTS: optional single gate name (fmt|clippy|filesize|unsafe|test|coverage|deny|audit|machete|replica).
