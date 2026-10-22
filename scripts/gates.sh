#!/bin/sh
# Tyrus quality gates — single source of truth.
#
# This is the ONLY place gate commands are defined. Both `scripts/pre-commit`
# (run by the local git hook) and `.github/workflows/ci.yml` (run in CI)
# invoke this script. Drift between local and CI is therefore architecturally
# impossible.
#
# Rule 9 of docs/standards/POWER_OF_TEN.md (Local-First Validation Parity)
# mandates this single-source design. Adopted via ADR 0008.
#
# Usage:
#   ./scripts/gates.sh             # run all gates
#   ./scripts/gates.sh all         # same as above
#   ./scripts/gates.sh fmt         # cargo fmt --check
#   ./scripts/gates.sh clippy      # cargo clippy --workspace --all-targets -D warnings
#   ./scripts/gates.sh test        # cargo nextest run --workspace --all-targets
#   ./scripts/gates.sh coverage    # cargo llvm-cov nextest --workspace --fail-under-lines <threshold>
#   ./scripts/gates.sh filesize    # enforce Rule 4 file ceiling (≤ 400 lines per .rs)
#   ./scripts/gates.sh deny        # cargo deny check
#   ./scripts/gates.sh audit       # cargo audit --deny warnings
#   ./scripts/gates.sh machete     # cargo machete (unused dependencies)
#   ./scripts/gates.sh unsafe      # Rule 13: forbid(unsafe_code) in every crate root
#   ./scripts/gates.sh replica     # oracle/apps/replica: parity, request set, tooling tests (no Docker)
#
# Environment variables:
#   TYRUS_SKIP_DENY=1      Skip cargo deny check (use only when tool unavailable).
#   TYRUS_SKIP_AUDIT=1     Skip cargo audit (use only when tool unavailable).
#   TYRUS_SKIP_MACHETE=1   Skip cargo machete (use only when tool unavailable).
#   TYRUS_SKIP_COVERAGE=1  Skip cargo llvm-cov coverage check (e.g. fast pre-commit path).
#   TYRUS_SKIP_FILESIZE=1  Skip the Rule 4 file ceiling check.
#   TYRUS_COVERAGE_MIN     Override the minimum line-coverage %
#                          (default: 73 — the 2026-05-03 baseline floor;
#                           ramp-up to 80 tracked in issue #163).
#
# Exit code: 0 on all gates pass, non-zero on first failure.

set -e

# --- Gate definitions --------------------------------------------------------
#
# Flags are intentionally identical between local hook and CI. If you change
# one of these, the change applies to BOTH simultaneously — preventing the
# silent divergence that allowed PR #142 to merge with expect_used violations.

gate_fmt() {
    echo "=== gate: fmt ==="
    cargo fmt --all -- --check
}

gate_clippy() {
    echo "=== gate: clippy ==="
    # --all-targets is critical: it lints test code too. Without this flag,
    # `#[cfg(test)] mod` blocks bypass the strict lint rules entirely.
    # --locked: builds resolve exactly the committed Cargo.lock (Rule 12).
    cargo clippy --workspace --all-targets --locked -- -D warnings
}

gate_test() {
    echo "=== gate: test ==="
    # nextest by default runs lib tests, integration tests, and doctests.
    # We deliberately do NOT pass --all-targets here: it would try to run
    # benchmarks as test binaries, but bench/runtime_comparison emits a
    # custom report format that does not match libtest's enumeration
    # protocol. clippy's --all-targets gate already lints the bench code;
    # actually executing benches belongs to the dedicated bench job.
    cargo nextest run --workspace --locked
}

gate_coverage() {
    if [ "${TYRUS_SKIP_COVERAGE:-0}" = "1" ]; then
        echo "=== gate: coverage (SKIPPED via TYRUS_SKIP_COVERAGE=1) ==="
        return 0
    fi
    echo "=== gate: coverage ==="
    if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
        echo "ERROR: cargo-llvm-cov is not installed."
        echo "Install with: cargo install cargo-llvm-cov"
        echo "Or skip this gate locally with: TYRUS_SKIP_COVERAGE=1"
        return 1
    fi
    # Rule 5 enforcement (POWER_OF_TEN.md): workspace line coverage
    # threshold. Configurable via TYRUS_COVERAGE_MIN (default tracks the
    # 2026-05-03 baseline floor — see issue #163 for ramp-up to 80%).
    # Benchmarks and `tyrus_test_utils` are excluded from the
    # denominator. The `--ignore-filename-regex` matches against the
    # *absolute path* of each source file (NOT the relative path
    # displayed in the report), so each fragment is anchored on `/`
    # rather than on a leading `^`.
    #
    # Multi-step invocation rationale: integration tests in
    # `tests/src/cli.rs` use `assert_cmd::Command::cargo_bin("tyrus")`,
    # which expects the `tyrus` binary inside
    # `target/llvm-cov-target/debug/`. A plain `cargo llvm-cov nextest`
    # builds test binaries but not the bin — CLI tests then fail with
    # NotFoundError. Splitting the run into
    # (1) clean profraw,
    # (2) `--no-report run --bin tyrus` to populate the binary,
    # (3) `--no-report nextest --workspace` for tests, and
    # (4) a final `report` for threshold enforcement
    # keeps both worlds consistent.
    threshold="${TYRUS_COVERAGE_MIN:-73}"
    # `--profraw-only` is incompatible with `--workspace` (cargo-llvm-cov
    # warns and ignores the latter). Cleaning at the workspace target
    # is implicit, so `--profraw-only` alone is correct.
    cargo llvm-cov clean --profraw-only
    cargo llvm-cov --no-report run --bin tyrus -- --version > /dev/null
    cargo llvm-cov --no-report nextest --workspace
    # `--summary-only` only takes effect with structured output
    # (--json/--lcov/--cobertura). For the human-readable text report
    # we rely on `--fail-under-lines` for the gate signal; the per-file
    # table is informative for the contributor and forwarded by CI.
    cargo llvm-cov report \
        --fail-under-lines "$threshold" \
        --ignore-filename-regex '/tyrus_test_utils/|/benches/'
}

gate_filesize() {
    if [ "${TYRUS_SKIP_FILESIZE:-0}" = "1" ]; then
        echo "=== gate: filesize (SKIPPED via TYRUS_SKIP_FILESIZE=1) ==="
        return 0
    fi
    echo "=== gate: filesize ==="
    # Rule 4 (POWER_OF_TEN.md) file ceiling: ≤ 400 lines per `.rs` file
    # under `crates/`. ADR 0008 also names a 800-line hard cap. Pre-existing
    # soft-cap violations are allowlisted (tracked in issue #153 — refactor);
    # they still fail if they cross the hard cap. New files must respect
    # the soft cap.
    soft_cap=400
    hard_cap=800

    # Allowlisted soft-cap violators (tracked in #153). Files in this
    # set may exceed the 400-line soft cap but never the 800-line hard cap.
    allowlist_pattern='/convert/class/constructor\.rs$|/convert/interface\.rs$'

    # "<lines> <path>" for every .rs file under crates/, descending.
    all=$(find crates -type f -name '*.rs' -print0 \
        | xargs -0 wc -l \
        | awk '$2 != "total" {print $1, $2}' \
        | sort -rn)

    fail=0

    # Hard cap violations are always fatal.
    hard=$(echo "$all" | awk -v cap="$hard_cap" '$1 > cap')
    if [ -n "$hard" ]; then
        echo "FAIL: Rule 4 hard cap (> $hard_cap lines) violations:"
        echo "$hard" | sed 's/^/  /'
        fail=1
    fi

    # Soft cap violations: fatal unless the file is on the allowlist.
    soft=$(echo "$all" | awk -v lo="$soft_cap" -v hi="$hard_cap" '$1 > lo && $1 <= hi')
    if [ -n "$soft" ]; then
        non_allowed=$(echo "$soft" | grep -vE "$allowlist_pattern" || true)
        allowed=$(echo "$soft" | grep -E "$allowlist_pattern" || true)
        if [ -n "$non_allowed" ]; then
            echo "FAIL: Rule 4 soft cap (> $soft_cap lines) violations:"
            echo "$non_allowed" | sed 's/^/  /'
            fail=1
        fi
        if [ -n "$allowed" ]; then
            echo "Allowlisted soft-cap warnings (tracked in #153):"
            echo "$allowed" | sed 's/^/  /'
        fi
    fi

    if [ "$fail" -eq 0 ] && [ -z "$soft" ]; then
        echo "All .rs files under crates/ are at or below $soft_cap lines."
    fi

    return "$fail"
}

gate_unsafe() {
    echo "=== gate: unsafe ==="
    # Rule 13 (POWER_OF_TEN.md, ADR 0013): every crate root must declare
    # #![forbid(unsafe_code)]. `forbid` cannot be overridden by inner
    # allows, so the compiler is the real enforcement — this gate only
    # guarantees the attribute never silently disappears (or is missing
    # from a newly added crate).
    fail=0
    for root in crates/*/src/lib.rs crates/*/src/main.rs tests/src/lib.rs; do
        [ -f "$root" ] || continue
        if ! grep -q '^#!\[forbid(unsafe_code)\]' "$root"; then
            echo "FAIL: missing #![forbid(unsafe_code)] in $root"
            fail=1
        fi
    done
    if [ "$fail" -eq 0 ]; then
        echo "All crate roots forbid unsafe_code."
    fi
    return "$fail"
}

gate_machete() {
    if [ "${TYRUS_SKIP_MACHETE:-0}" = "1" ]; then
        echo "=== gate: machete (SKIPPED via TYRUS_SKIP_MACHETE=1) ==="
        return 0
    fi
    echo "=== gate: machete ==="
    if ! command -v cargo-machete >/dev/null 2>&1; then
        echo "ERROR: cargo-machete is not installed."
        echo "Install with: cargo install cargo-machete"
        echo "Or skip this gate locally with: TYRUS_SKIP_MACHETE=1"
        return 1
    fi
    # Rule 12 (POWER_OF_TEN.md): unused dependencies are supply-chain
    # surface with zero benefit. Textual scan — fast enough for every run.
    cargo machete
}

gate_deny() {
    if [ "${TYRUS_SKIP_DENY:-0}" = "1" ]; then
        echo "=== gate: deny (SKIPPED via TYRUS_SKIP_DENY=1) ==="
        return 0
    fi
    echo "=== gate: deny ==="
    cargo deny check
}

gate_audit() {
    if [ "${TYRUS_SKIP_AUDIT:-0}" = "1" ]; then
        echo "=== gate: audit (SKIPPED via TYRUS_SKIP_AUDIT=1) ==="
        return 0
    fi
    echo "=== gate: audit ==="
    # Ignored advisories are duplicated from deny.toml [advisories].ignore
    # because cargo-audit doesn't read deny.toml. Keep these in sync with
    # deny.toml whenever advisories are triaged.
    cargo audit \
        --ignore RUSTSEC-2025-0119 \
        --ignore RUSTSEC-2026-0097 \
        --ignore RUSTSEC-2026-0249 \
        --deny warnings
}

gate_replica() {
    echo "=== gate: replica ==="
    sh scripts/check_replica_parity.sh
    python3 -B -I oracle/apps/replica/make_requests.py --check
    python3 -B -I -m unittest discover -s oracle/apps/replica/test -p 'test_*.py'
    if git ls-files oracle/apps/replica | grep -q 'node_modules/'; then
        echo "FAIL: node_modules versionado em oracle/apps/replica"
        return 1
    fi
    if command -v shellcheck >/dev/null 2>&1; then
        find oracle/apps/replica/db oracle/apps/replica/tools -name '*.sh' -print0 \
            | xargs -0 shellcheck -x -P SCRIPTDIR scripts/check_replica_parity.sh
    elif [ "${TYRUS_REQUIRE_SHELLCHECK:-0}" = "1" ]; then
        echo "ERROR: shellcheck ausente e TYRUS_REQUIRE_SHELLCHECK=1"
        return 1
    else
        echo "WARN: shellcheck ausente; etapa ignorada (o CI a exige)"
    fi
}

# --- Dispatcher --------------------------------------------------------------

cmd="${1:-all}"
case "$cmd" in
    fmt)      gate_fmt ;;
    clippy)   gate_clippy ;;
    test)     gate_test ;;
    coverage) gate_coverage ;;
    filesize) gate_filesize ;;
    deny)     gate_deny ;;
    audit)    gate_audit ;;
    machete)  gate_machete ;;
    unsafe)   gate_unsafe ;;
    replica)  gate_replica ;;
    all)
        gate_fmt
        gate_clippy
        gate_filesize
        gate_unsafe
        gate_replica
        gate_test
        gate_coverage
        gate_deny
        gate_audit
        gate_machete
        echo ""
        echo "=== ALL GATES PASSED ==="
        ;;
    -h|--help|help)
        sed -n '2,25p' "$0"
        exit 0
        ;;
    *)
        echo "ERROR: unknown gate '$cmd'" >&2
        echo "Run: $0 help" >&2
        exit 2
        ;;
esac
