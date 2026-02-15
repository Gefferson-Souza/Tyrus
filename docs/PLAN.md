# 📋 Regression Resolution Plan: Safe Transpilation Hardening

This plan outlines the architectural stabilization of Tyrus to resolve current `cargo test` failures caused by paradigm mismatches between TypeScript (Shared-Mutable) and Rust (Owned-Immutable).

## 🛠 Problem Analysis

The root cause is a **Memory Semantics Mismatch**:

- **Services (Singletons)**: Currently generated as `Arc<T>`, but methods use `&mut self`. Rust forbids mutable borrowing of data inside an `Arc` without a lock.
- **Iterator Semantics**: TS `map(val, index)` does not find a direct 1:1 match in Rust's `.map()`, leading to argument count errors.
- **String Handling**: Perceived "Magic Strings" in TS lead to `String` vs `&str` errors in Rust when calling native methods (e.g., `.contains()`).

## 🧱 Proposed Changes

### 1. Interior Mutability Policy (`tyrus_codegen`)

We will implement "Automatic Lock Injection" for all classes identified as Services/Controllers.

- **Class Definition**: Use `Arc<Mutex<T>>` for dependency inboxes instead of raw `Arc<T>`.
- **Method Generation**:
  - If Service/Controller: Use `&self`.
  - **Auto-Locking**: In `convert_member_expr`, if accessing `this.field` where `field` is a Service, automatically wrap the access in a scoped lock: `{ let mut field = self.field.lock().unwrap(); field.method() }`.
  - **Atomic Preference**: For primitive fields (`number`, `boolean`), use `std::sync::atomic` types (AtomicF64, AtomicBool) to avoid Mutex overhead where possible.

### 2. Iterator Semantic Wrapper (`tyrus_codegen/src/convert/func.rs`)

Replace string-based "guesswork" with a robust AST-driven transformation for Array methods.

- **Detection**: Check if `map`, `filter`, or `forEach` closures have 2 arguments.
- **Transformation**:
  - TS: `arr.map((v, i) => ...)`
  - Rust: `arr.into_iter().enumerate().map(|(i, v)| { ... }).collect()`
- **Safety**: Ensure `filter` correctly handles the `&(index, value)` reference pattern to prevent ownership leaks.

### 3. String & Borrow Hardening

Implement a dedicated `borrow_aware_concat` helper in `tyrus_codegen`.

- TS: `str1 + str2` -> Rust: `format!("{}{}", str1, str2)` or `str1 + &str2`.
- Detect calls to `.contains()` or `.starts_with()` and automatically borrow the argument if it's a `String`.

### 4. Axiom / Http Client Fix (`tyrus_codegen`)

- Ensure `HttpClient` (axios wrapper) is correctly generated with `axios` crate or a localized `mod axios` for the test fixtures.

## 🧪 Verification Plan

### Automated Tests

- **Regression Suite**: `cargo test --package integration_tests --lib -- verify_equivalence`
- **Fixture Audit**: Run `cargo check` on all `tests/fixtures/*/dist` directories.
- **Clippy**: `cargo clippy --workspace -- -D warnings`

### Manual Verification

- Review the generated `src/utils/http_client.rs` for correct mutability and `axios` usage.
