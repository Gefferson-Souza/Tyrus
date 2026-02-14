# 🗺️ Tyrus Roadmap

This roadmap tracks the evolution of Tyrus from a research prototype to a production-grade compiler.

## ✅ Completed Milestones (Production Ready)

### 🏁 Milestone 1-3: The Foundation & Core Logic

- [x] **CLI & Parser:** Full integration with SWC for TS source analysis.
- [x] **Core Transpilation:** Arithmetic, logic, and control flow mappings.
- [x] **Semantic Analyzer:** Implementation of the "Oxidizable Standard" lints.

### 🏗️ Milestone 4-5: Type Excellence & Ecosystem

- [x] **Structural Typing:** Mapping TypeScript interfaces to Rust structs with Serde support.
- [x] **Generics:** Multi-type parameter support with trait bound inference.
- [x] **Async Revolution:** Mapping JS Promises to Rust Futures and `tokio` runtime.

### 🌐 Milestone 6: Framework Integration & Tier 3 Features

- [x] **NestJS Synthesis:** Transpiling Decorators to Axum handlers.
- [x] **Advanced Loops:** `for..in`, `do..while` mappings.
- [x] **Type Aliases:** String Unions to Enums, `Record<K,V>` to `HashMap`.
- [x] **Shim Layer:** 100% coverage of core `Math`, `String`, and `Array` methods.

---

## 🔬 Future Work (Academic Research)

### Tier 4: Advanced OOP & Metaprogramming

- [ ] **Class Inheritance:** Mapping complex prototype chains to Rust Traits and Composition.
- [ ] **Custom Decorators:** Support for user-defined metadata and proxy logic.
- [ ] **Macro System:** Compiling TypeScript template literals and type-level programming into Rust macros.

### Tier 5: Optimization & Verification

- [ ] **Formal Verification:** Mathematical proof of semantic preservation.
- [ ] **IR Optimizations:** LLVM-style passes on the Tyrus intermediate representation.
- [ ] **Cinterop:** Automated binding generation for C-compatible Rust libraries.
