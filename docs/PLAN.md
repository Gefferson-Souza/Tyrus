# Documentation Update Plan

**Goal:** align documentation with the current project state (Tyrus v0.1.0), reflecting the new name, test harness, and supported features.

## 1. Project Renaming (Oxidizer -> Tyrus)

- [ ] **CONTRIBUTING.md**: Update title and references.
- [ ] **docs/STANDARDIZATION_PLAN.md**: Update references to Tyrus.
- [ ] **README.md**: Ensure consistent usage (already mostly done, but double-check).

## 2. Technical Accuracy & Features

- [ ] **docs/specs/GRAMMAR.md**:
  - Add Unary Expressions (`!`, `-`, `+`).
  - Update Control Flow (if changed).
  - Add `for..of` / `for..in` if supported (check status).
- [ ] **CONTRIBUTING.md**:
  - Update "Rodando Testes" section to emphasize `cargo test --workspace` (the new harness).
  - Mention `tyrus_test_utils` assertions.

## 3. Architecture & Standards

- [ ] **docs/architecture/**:
  - Review existing ADRs for obsolescence.
  - Create a new ADR (or update existng) for "The Iron Clad Harness".
- [ ] **Language Policy**:
  - `README.md` is currently in PT-BR. `STANDARDIZATION_PLAN` suggests English default.
  - _Decision_: Keep PT-BR for now but tag as `README.pt-br.md` if we create an English one? For this sprint, just ensure internal consistency.

## 4. Execution (Phase 2)

| Agent                  | Task                                                 |
| ---------------------- | ---------------------------------------------------- |
| `documentation-writer` | Update `CONTRIBUTING.md`, `GRAMMAR.md`, `README.md`. |
| `project-planner`      | Update `STANDARDIZATION_PLAN.md`.                    |
| `backend-specialist`   | Create `docs/architecture/0005-test-harness.md`.     |

## 5. Verification

- [ ] `lint_runner.py` (markdown linting if available).
- [ ] Manual review of rendered markdown.
