# Project Alignment Plan: Safe Transpilation & Architecture

**Goal:** Align `TypeRust` codebase with the architectural principles defined in the new documentation:

1.  _The Architecture of Safe Transpilation_
2.  _From Source to Syntax_
3.  _Analysis of Rust Development Paradigms_

## Phase 1: Architectural Audit & Safety Hardening

- [ ] **Panic Elimination**: Scan codebase for `unwrap()`, `expect()`, and `todo!()`. Replace with proper `Result<T, OxidizerError>` handling.
  - _Rationale_: "Parser is a Minefield". Panics are unacceptable vulnerabilities.
- [ ] **AST Robustness**: Verify `tyrus_codegen` uses robust ADTs (Enums/Structs) as defined in _From Source to Syntax_.
  - _Note_: `swc_ecma_ast` is already AST-based, but our _conversion logic_ must be strict.
- [ ] **Error Handling**: Ensure all potential failure points in `tyrus_codegen` return `Err` (e.g., "unknown node type") rather than silent failure or panic.

## Phase 2: CI/CD & Quality Assurance

- [ ] **Linting (Strict)**: Enforce `clippy::pedantic` or at least `-D warnings` in `ci.yml`.
  - _Reference_: "Code always has to follow best practices".
- [ ] **Test Coverage**: Run `verify_equivalence` and `integration_tests`.
  - _Action_: Ensure `todo.ts` issues (ownership) are clearly documented or resolved if blocking.
- [ ] **CI Workflow**: Verify `.github/workflows/ci.yml` runs:
  - `cargo check`
  - `cargo clippy`
  - `cargo test`
  - `cargo fmt --check`

## Phase 3: Documentation & Roadmap

- [ ] **Update Documentation**: Update `README.md` to reflect the "Safe Transpilation" philosophy.
- [ ] **Artifact Update**: Ensure `walkthrough.md`, `task.md`, and logic maps are current.

## Phase 4: Verification

- [ ] **End-to-End Validation**: Run full suite.
- [ ] **Orchestration Report**: Generate final report.

## Execution Strategy

1.  **Audit**: `grep` for panics/todos.
2.  **Refactor**: Fix identified unsafe patterns.
3.  **Verify**: Run local CI simulation.
4.  **Document**: Update docs.
