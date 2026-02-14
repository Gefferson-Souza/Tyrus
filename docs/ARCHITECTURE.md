# 🏛️ Tyrus Architecture

Tyrus is structured as a multi-stage compilation pipeline, designed for modularity and extensibility. This document describes the flow from TypeScript source to a final, runnable Rust project.

## 🔄 The Compilation Pipeline

### 1. Parsing (`tyrus_parser`)

The entry point uses the `swc_ecma_parser` to ingest TypeScript source files.

- **Input:** `.ts` source code.
- **Output:** Abstract Syntax Tree (AST).
- **Responsibility:** Ensure the input is syntactically valid TypeScript.

### 2. Semantic Analysis (`tyrus_analyzer`)

This stage validates the AST against the **Oxidizable Standard**.

- **Input:** AST.
- **Rules:** Bans `any`, `eval`, and unassigned `var`.
- **Inference:** Maps TS types to their corresponding Rust Newtypes or Primitives.
- **Output:** Validated AST + Metadata (Dependency Graph).

### 3. Orchestration (`tyrus_orchestrator`)

The "brain" of the compiler.

- **Responsibility:** Manages multi-file resolution, project scoping, and the generation of the Rust directory structure (e.g., creating `Cargo.toml`, `src/main.rs`).
- **Dependency Injection:** Resolves singleton patterns (like Services in NestJS) to `Arc<T>` or `State` in Rust using `tyrus_di`.
- **Graph Resolution:** Uses `tyrus_di` to topologically sort dependencies and determine instantiation order.

### 4. Dependency Management (`tyrus_di`)

A dedicated crate for handling the application's dependency graph.

- **Input:** Module metadata and provider definitions.
- **Algorithm:** Topological sort via `petgraph`.
- **Output:** Ordered initialization list and separation of concerns (Modules vs Providers vs Controllers).

### 5. Code Generation (`tyrus_codegen`)

The final stage that renders the Rust source code.

- **Input:** Analyzed AST.
- **Technology:** Uses the `quote!` and `proc-macro2` crates for idiomatic formatting.
- **Output:** `.rs` files that follow Rust's strict safety and ownership rules.

---

## 📦 Crate Breakdown

| Crate               | Responsibility                                               |
| :------------------ | :----------------------------------------------------------- |
| `tyrus_ast`         | Formal definition of the project's internal representation.  |
| `tyrus_cli`         | Command-line interface and user interaction logic.           |
| `tyrus_common`      | Generic utilities and shared types (e.g., `FilePath`).       |
| `tyrus_diagnostics` | Error reporting and tracing infrastructure.                  |
| `tyrus_di`          | Dependency Injection graph resolution and module management. |
| `tyrus_test_utils`  | Custom harness for regression and compiler-output testing.   |

---

## 🛠 Tech Stack

- **Source Language:** TypeScript (via SWC)
- **Target Language:** Rust (1.75+)
- **Macro Engine:** `quote`, `syn`
- **Internal Web Engine:** `axum` (for NestJS mappings)
