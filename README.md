> 🌐 **Language:** English | [Português (BR)](README.pt-br.md)

# Tyrus: A High-Fidelity TypeScript-to-Rust Compiler

_Experimental project in compiler theory and semantic preservation_

[![CI Status](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml/badge.svg)](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

Tyrus is a source-to-source compiler designed to bridge the gap between high-level dynamic syntax (TypeScript) and low-level memory safety (Rust). It focuses on the formal mapping of higher-order abstractions to zero-cost Rust equivalents, exploring the boundaries of **Semantic Preservation** across differing execution models.

**Milestone:** Tyrus can now compile a NestJS CRUD API into a working Axum (Rust) server with a single command:

```bash
tyrus compile my-nestjs-project/src --output rust-server
# → Rust binary with GET/POST endpoints, DI, and in-memory state
```

---

## 🔬 Core Philosophies

### 🛡️ Zero Magic

Tyrus does not rely on a hidden runtime or garbage collection emulation. Every TypeScript construct is mapped to its most efficient Rust equivalent, leveraging Rust's ownership model and strict type system at compile time.

### 📐 Semantic Preservation

The primary goal is formal equivalence. If a TypeScript project is "Oxidizable," the generated Rust code should keep the original logic's behavior. This is not yet guaranteed: the patterns listed below are the ones verified so far.

### 🚫 The Oxidizable Standard

Tyrus enforces a strict subset of TypeScript called the "Oxidizable Standard." It rejects non-idiomatic or unsafe patterns (like `any` or `eval`) to ensure the resulting Rust code is both safe and performant.

### 🔐 Safe Transpilation Architecture

Adhering to strict "Safe Transpilation" principles:

- **Panic-Free Compilation**: Compiler logic uses robust error handling instead of panicking on invalid input.
- **Strict Linting**: The codebase is verified with `clippy::pedantic` rules (e.g., no `unwrap()`/`expect()` in production paths).
- **Formal AST Mapping**: Uses Algebraic Data Types (ADTs) to represent logic, avoiding string manipulation vulnerabilities.

---

## 🚀 Feature Tiers

### Tier 1: Core Language (Experimental)

- Primitives (`string`, `number`, `boolean`)
- Control Flow (`if/else`, `while`, `for`)
- Error Handling (`Result`, `Option`)

### Tier 2: Advanced Type System (Experimental)

- Interfaces and Type Aliases to Structs/Enums
- Generics and Polymorphism
- Comprehensive Collection Mapping (`Array<T>` -> `Vec<T>`)

### Tier 3: Ecosystem & Asynchrony (Experimental)

- `Async/Await` to Future-based concurrency
- JSON Serialization/Deserialization (via `serde`)
- HTTP Client and REST patterns (via `axum` & `reqwest`)

### 📦 Supported Patterns (Verified with Semantic Equivalence Tests)

- **String Methods** (16): `includes`, `replace`, `split`, `toUpperCase`, `toLowerCase`, `trim`, `startsWith`, `endsWith`, `toString`, `substring`, `charAt`, `indexOf`, `repeat`, `slice`, `padStart`, `padEnd`
- **Array Methods** (15): `map`, `filter`, `forEach`, `find`, `some`, `every`, `reduce`/`fold`, `join`, `includes`, `push`, `indexOf`, `slice`, `concat`, `reverse`, `pop`, `sort`, `shift`, `flat`, `flatMap`
- **Math Functions** (15): `max`, `min`, `round`, `floor`, `ceil`, `abs`, `random`, `pow`, `sqrt`, `log`, `trunc`, `sign`, `sin`, `cos`, `tan`
- **Math Constants** (2): `Math.PI`, `Math.E`
- **Console** (5): `log`, `error`, `warn`, `info`, `debug`
- **Control Flow**: `if/else`, `while`, `for`, `for-of`, `do-while`, `switch/case`, ternary, `try/catch`
- **Top-Level Statements**: `const`, `let`, expressions auto-wrapped in `fn main()`
- **Spread Operator**: `[...arr1, ...arr2]` → iterator chain, `{...obj, field}` → struct update
- **Class Inheritance**: `extends`, `super()`, method override via field flattening
- **Getters/Setters**: `get prop()` → `fn prop(&self)`, `set prop(v)` → `fn set_prop(&mut self, v)`
- **Static Methods**: `static add()` → associated function, `Class.method()` → `Class::method()`
- **Type Assertions**: `as Type` → no-op (compile-time only)
- **Numeric Enums**: `enum Direction { Up = 0 }` → `#[repr(i32)] enum` with `Display`
- **Operators**: Arithmetic, comparison, logical, `**` (exponentiation), `%` (modulo)
- **Class State**: Automatic `Arc<Mutex<T>>` wrapping for services/controllers
- **Interfaces**: `interface` -> `#[derive(Serialize, Deserialize)] struct`
- **String Unions**: `type Status = "a" | "b"` -> `enum` with `Display` and `PartialEq`
- **NestJS Guards**: `@UseGuards(Guard)` → `axum::middleware::from_fn()` layer

---

## 🛠 Installation & Usage

### Prerequisites

- Rust 1.75+ (Stable)
- Cargo

### Setup

```bash
git clone https://github.com/Gefferson-Souza/Tyrus.git
cd Tyrus
cargo build --release
```

### Install Globally

```bash
# Install tyrus as a global CLI command
cargo install --path crates/tyrus_cli

# Now use from anywhere
tyrus --version
```

### Using the Compiler

```bash
# Analyze a TypeScript file for compatibility
tyrus check ./src/index.ts

# Get JSON diagnostics (for tooling integration)
tyrus check --json ./src/index.ts

# Transpile to Rust source code (stdout or file)
tyrus build ./src/index.ts
tyrus build ./src/index.ts -o output.rs

# Transpile + compile to native binary
tyrus compile ./src/index.ts --output ./output

# Transpile + compile + execute
tyrus run ./src/index.ts --output ./output

# Suppress banner for scripting
tyrus --quiet check ./src/index.ts
```

---

## 📋 Commands Reference

<!-- AUTO-GENERATED from Cargo.toml and CI -->

**Development Commands:**

| Command | Description |
|---------|-------------|
| `cargo build --workspace` | Build all workspace crates |
| `cargo build --release` | Production build with LTO |
| `cargo install --path crates/tyrus_cli` | Install `tyrus` CLI globally |
| `cargo nextest run --workspace` | Run all tests (parallel, preferred) |
| `cargo test --workspace` | Run all tests (legacy runner) |
| `cargo test -p integration_tests` | Integration tests only |
| `cargo clippy --workspace` | Lint with strict rules (`-Dwarnings` enforced) |
| `cargo fmt -- --check` | Check formatting |
| `cargo insta review` | Review snapshot changes |

**Tyrus CLI Commands (after global install):**

| Command | Description |
|---------|-------------|
| `tyrus check <file.ts>` | Analyze a TypeScript file for Oxidizable compatibility |
| `tyrus check --json <file.ts>` | JSON diagnostic output (for tooling integration) |
| `tyrus build <file.ts>` | Transpile to Rust (stdout) |
| `tyrus build <file.ts> -o output.rs` | Transpile to Rust (file) |
| `tyrus build <dir> -o <output_dir>` | Transpile directory to Cargo project |
| `tyrus compile <file.ts> -o <output_dir>` | Transpile + compile to native binary |
| `tyrus compile <file.ts> --release` | Transpile + compile with optimizations |
| `tyrus run <file.ts>` | Transpile + compile + execute |
| `tyrus --quiet <command>` | Suppress banner for scripting |
<!-- /AUTO-GENERATED -->

---

## 🧪 Test Suite

195 tests across 8 test types and 4 feature tiers:

| Type | Count | Description |
|------|-------|-------------|
| **Equivalence** | 81 | Semantic proof: TS and Rust produce identical stdout |
| **Unit** | 49 | Fast, isolated codegen function tests |
| **Snapshot** | 20 | Full transpilation output via `insta` |
| **IR** | 21 | Typed intermediate representation lowering |
| **Compilation** | 9 | Generated Rust passes `cargo check` per tier |
| **CLI** | 7 | Integration tests for all CLI commands and flags |
| **Trybuild** | 1 | Compile-verification of generated Rust |
| **E2E/Build** | 5 | Full pipeline: transpile → compile → run server → verify HTTP |

Test types: **Equivalence** (TS↔Rust same output) · **Unit** (fast, isolated functions) · **Snapshot** (insta, codegen output) · **IR** (type lowering) · **Compilation** (generated Rust passes `cargo check`) · **CLI** (command integration) · **E2E** (HTTP server verification) · **Trybuild** (compile-verification)

---

## 📖 Thesis & Architecture

For a deep dive into the compiler's internals, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## 🏗 Code Generation Module Structure

The `tyrus_codegen` crate is organized into focused, single-responsibility modules under `crates/tyrus_codegen/src/convert/`:

```
convert/
├── mod.rs          — module declarations and re-exports
├── interface.rs    — RustGenerator struct definition + Visit impl (entry point)
├── helpers.rs      — shared utilities: to_snake_case, to_pascal_case, is_string_expr
├── fn_decl.rs      — function declaration processing (process_fn_decl)
├── module.rs       — module/import handling
├── type_mapper.rs  — TypeScript → Rust type mapping (deduplicated map_type_core)
├── stmt/           — statement conversion
│   ├── mod.rs          — dispatcher + convert_stmt, convert_stmt_recursive
│   ├── var_decl.rs     — variable declarations (ident, object/array destructuring)
│   ├── loops.rs        — while, for-of, for-in, for, do-while
│   ├── switch.rs       — switch → match
│   └── try_catch.rs    — try-catch → Result matching
├── class/          — class → struct+impl
│   ├── mod.rs          — dispatcher + property conversion
│   ├── constructor.rs  — constructor transpilation + DI
│   ├── method.rs       — method transpilation + decorators
│   ├── routing.rs      — Axum router generation + @UseGuards middleware
│   └── mutation.rs     — self-mutation detection
└── expr/
    ├── mod.rs      — expression dispatcher (convert_expr)
    ├── binary.rs   — binary operators (convert_bin_expr)
    ├── call.rs     — function/method calls, axios/fetch/array methods
    ├── member.rs   — property access, mutex state (convert_member_expr)
    ├── arrow.rs    — arrow functions → closures (convert_arrow_expr)
    ├── literal.rs  — literals, object/array/template expressions
    └── misc.rs     — assignments, updates, optional chaining
```

All Rust code is generated using `quote!` macros producing `proc_macro2::TokenStream` — never string concatenation.

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.
