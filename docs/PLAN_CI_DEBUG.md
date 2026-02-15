# 🎼 Plan: CI Debugging & Fixes (Tyrus v1.0.1)

**Objective:** specific compiler errors in generated Rust code that are failing CI tests locally.

## 🐛 Identified Issues

1.  **Enum Access Error (`E0423`)**:
    - _Symptom:_ `Status.Active` generated instead of `Status::Active`.
    - _Location:_ `tyrus_codegen/src/convert/func.rs` (Member Expression).
    - _Fix:_ Detect Enum types and use `::` separator.

2.  **Vec Method Chaining (`E0599`)**:
    - _Symptom:_ `.map()` called on `Vec<T>`.
    - _Location:_ `tyrus_codegen/src/convert/func.rs` (Call/Member Expression).
    - _Fix:_ Inject `.iter()` or `.into_iter()` before array methods.

3.  **Type Inference (`E0282`)**:
    - _Symptom:_ `None` generated without type hints in typical `serde_json` contexts.
    - _Location:_ `tyrus_codegen` (Null/Undefined conversion).
    - _Fix:_ Use `Option::<T>::None` or ensure context provides type (more complex, might need explicit `None::<String>` placeholder or better `serde_json::json!` handling).

## 👥 Orchestration Team

| Agent                | Role          | Focus Area                                                                |
| -------------------- | ------------- | ------------------------------------------------------------------------- |
| `debugger`           | **Diagnosis** | Locate exact lines in `func.rs` producing the invalid code.               |
| `backend-specialist` | **Fixer**     | Patch `tyrus_codegen` logic.                                              |
| `test-engineer`      | **Validator** | Run specific failing tests (`test_tier1_features`, `verify_equivalence`). |

## 📅 Execution Strategy

### Phase 1: Diagnosis & Fix (`debugger` + `backend-specialist`)

- [ ] **Fix Enum Access:** Update `convert_member_expr` to check if `object` is an Identifier that maps to a known Enum (or heuristic for Capitalized names).
- [ ] **Fix Array Methods:** Update `convert_member_expr` or `convert_call_expr` to intercept `map`, `filter`, `forEach` on Arrays and prepend `.iter()`.
- [ ] **Fix Option Inference:** Review where `None` is generated.

### Phase 2: Verification (`test-engineer`)

- [ ] Run `cargo test --package integration_tests --lib -- test_regression::test_tier1_features`
- [ ] Run `cargo test --package integration_tests --lib -- verify_equivalence`
- [ ] Run `cargo test --bin test_types`

### Phase 3: Validation

- [ ] Run full `debug_ci.sh` or `ci.yml` steps locally.

---

## 🚦 Decision Point for User

**Plan created.**

1. Fix Enum `.` to `::`.
2. Fix Array `.map` to `.iter().map`.
3. Fix `None` inference.

**Do you approve? (Y/N)**
