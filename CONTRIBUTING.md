# Contributing to Tyrus

## 🌳 Git Workflow: The "Tyrus Pattern"

We follow a strict **Feature Branch Workflow** combined with **Conventional Commits**.

### Branching Strategy

- **`main`**: Protected. Production-ready code only. No direct commits.
- **`feat/`**: New features (e.g., `feat/async-await`, `feat/new-parser`).
- **`fix/`**: Bug fixes (e.g., `fix/memory-leak`, `fix/cli-panic`).
- **`chore/`**: Maintenance, config, docs (e.g., `chore/optimize-workflow`, `docs/update-readme`).
- **`refactor/`**: Code restructuring without behavior change.

### 📝 Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/).

**Format:** `<type>(<scope>): <subject>`

**Types:**

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Changes that do not affect the meaning of the code (white-space, formatting, etc)
- `refactor`: A code change that neither fixes a bug nor adds a feature
- `perf`: A code change that improves performance
- `test`: Adding missing tests or correcting existing tests
- `chore`: Changes to the build process or auxiliary tools and libraries such as documentation generation

**Examples:**

- `feat(codegen): implement structural typing for interfaces`
- `fix(cli): resolve panic when input file is missing`
- `chore(deps): upgrade axum to v0.7`

## 🚀 Pull Request Process

1.  Create a branch complying with the strategy above.
2.  Run **all** local gates (single command — same script CI runs):
    ```bash
    ./scripts/gates.sh all
    ```
    This covers fmt, clippy, tests, coverage (≥ 80%), `cargo deny`, and `cargo audit`. See `docs/standards/POWER_OF_TEN.md` Rule 9 (Local-First Validation Parity).
3.  Open a PR to `main`.
4.  Fill out the **PR Template** completely.
5.  Wait for CI checks to pass and request review.

### Local coverage check (Rule 5)

The `coverage` gate uses `cargo-llvm-cov` to enforce the workspace line-coverage threshold. The current default is **73%** (the 2026-05-03 baseline floor at 73.25% with test-infra excluded); ramp-up to 80% is tracked in [issue #163](https://github.com/Gefferson-Souza/Tyrus/issues/163) alongside the equivalence-test sprint (#154).

Install once:

```bash
cargo install cargo-llvm-cov
```

Then run on demand:

```bash
./scripts/gates.sh coverage                          # uses default 73% threshold
TYRUS_COVERAGE_MIN=80 ./scripts/gates.sh coverage    # one-off stricter check
TYRUS_SKIP_COVERAGE=1 ./scripts/gates.sh all         # skip in slow paths only
```

`tyrus_test_utils` and any `*/benches/` are excluded from the denominator (they are infrastructure, not product code). Integration tests under `tests/` do not appear in the coverage report at all (they are test runners, not target code).

Internally, `gate_coverage` runs as four steps:

1. `cargo llvm-cov clean --profraw-only` — clear stale instrumentation
2. `cargo llvm-cov --no-report run --bin tyrus -- --version` — build the `tyrus` binary into `target/llvm-cov-target/debug/` so CLI integration tests (`tests/src/cli.rs::assert_cmd::cargo_bin`) can find it
3. `cargo llvm-cov --no-report nextest --workspace` — collect coverage data
4. `cargo llvm-cov report --fail-under-lines $TYRUS_COVERAGE_MIN ...` — render report and enforce threshold

`cargo-llvm-cov` must be installed locally (`cargo install cargo-llvm-cov`) — `gate_coverage` preflight-checks for it and prints an install hint if missing.

## 🧪 Test Infrastructure

Tests live in the `tests/` crate and are organized by tier:

```
tests/src/
├── unit/          — unit tests for isolated functions/utilities
├── snapshot/      — insta snapshot tests (codegen output verification)
└── compilation/   — compilation tests (generate Rust, invoke rustc)
```

### Running specific test types

```bash
# The oracle tests run the real pinned Node: export TYRUS_NODE_REFERENCE and TYRUS_NODE_SEED first (see CLAUDE.md)
# All tests (preferred: uses nextest)
cargo nextest run --workspace

# Legacy test runner
cargo test --workspace

# Only snapshot tests (insta)
cargo test -p tests snapshot

# Only compilation tests
cargo test -p tests compilation

# Update snapshots after intentional codegen changes
cargo insta review
```

### Strict code rules (enforced by CI)

- **Never** use `.unwrap()`, `.expect()`, or `panic!()` — use `?` and `Result<T, TyrusError>`.
- **Never** use `todo!()` or `unimplemented!()` — use `compile_error!()` or proper error variants.
- **Never** use string concatenation for code generation — use `quote!` macros.
- Functions must stay under 50 lines; cognitive complexity threshold is 15.
- All new code must be covered by at least one test.

## 🏗 Code Generation Module Map

When working on code generation, the relevant files are under `crates/tyrus_codegen/src/convert/`:

| Module | Responsibility |
|---|---|
| `interface.rs` | `RustGenerator` struct + `Visit` impl (pipeline entry point) |
| `helpers.rs` | `to_snake_case`, `to_pascal_case`, `is_string_expr` |
| `stmt.rs` | Statement conversion |
| `fn_decl.rs` | Function declaration processing |
| `module.rs` | Module/import handling |
| `type_mapper.rs` | TypeScript → Rust type mapping (`map_type_core`) |
| **`class/`** | **Class → struct+impl (decomposed from monolithic `class.rs`)** |
| `class/mod.rs` | Class dispatcher + property conversion |
| `class/constructor.rs` | Constructor transpilation + DI detection |
| `class/method.rs` | Method transpilation + decorator parsing |
| `class/routing.rs` | Axum router generation + `FromRequestParts` |
| `class/mutation.rs` | Static self-mutation analysis |
| **`expr/`** | **Expression conversion (decomposed from monolithic `func.rs`)** |
| `expr/mod.rs` | Expression dispatcher |
| `expr/call.rs` | Function/method calls, array methods |
| `expr/member.rs` | Property access, mutex state |
| `expr/binary.rs` | Binary operators |
| `expr/arrow.rs` | Arrow functions → closures |
| `expr/literal.rs` | Literals, objects, arrays, template literals |
| `expr/misc.rs` | Assignments, updates, optional chaining |

### Orchestrator Module Map

The orchestrator (`crates/tyrus_orchestrator/src/`) coordinates multi-file builds:

| Module | Responsibility |
|---|---|
| `lib.rs` | Slim public API (`build()` entry point) |
| `pipeline.rs` | Core multi-file build orchestration |
| `scaffold.rs` | Project scaffolding (`main.rs`, `Cargo.toml`, `mod.rs`) |
| `format.rs` | Code formatting + `AppError` generation |

## 💾 Disk Hygiene

The Tyrus build tree can reach **~22 GB** during heavy use. Two reasons:

1. `target/` carries the standard Rust artefacts (debug ≈2.5 GB, llvm-cov-target ≈2 GB, tests ≈1.6 GB).
2. `/tmp/tyrus_test_target/` is a **shared `CARGO_TARGET_DIR`** used by every `assert_rust_compiles` and `assert_output_equivalent` test (see `crates/tyrus_test_utils/src/lib.rs:18`). Each test compiles a generated Rust project that depends on `axum`, `tokio`, `serde`, `reqwest`, etc. Sharing the target dir avoids rebuilding those crates per test (≈10× speed-up); the cost is unbounded growth.

**Cleanup:**

```bash
# See current footprint
bash scripts/disk-clean.sh --report

# Drop only stale artifacts (>14 days untouched in /tmp, >30 days
# in cargo's incremental cache, >7 days .profraw files)
bash scripts/disk-clean.sh

# Full wipe — recover everything (requires next test run to rebuild deps)
bash scripts/disk-clean.sh --all
```

The shared `/tmp` target is intentional. Do **not** redirect tests to a per-test target — the suite walltime jumps from ≈60 s to ≈10 min.
