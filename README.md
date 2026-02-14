# Tyrus

> **TypeScript → Rust Transpiler**
> _Academic Project: High-Performance Source-to-Source Compiler_

![CI Status](https://github.com/Gefferson-Souza/Tyrus/actions/workflows/ci.yml/badge.svg)
![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

**Tyrus** (formerly _Oxidizer_) is an experimental software engineering tool designed to analyze modern TypeScript code (including NestJS patterns) and transpile it into idiomatic, safe, and performant Rust code.

The goal is not to support 100% of the TypeScript specification, but rather to define an **"Oxidizable Standard"** subset that enables writing robust TS backends and compiling them to a native binary.

## 🚀 Philosophy

1. **Safety First:** If the TS code is unsafe (`any`, `eval`), Tyrus rejects the compilation.
2. **Idiomatic Output:** We don't generate "JavaScript in Rust". We generate real Rust (`Result`, `Option`, `Structs`, `Tokio`).
3. **Opinionated:** We focus on modern backend architecture (Controller/Service/Repository).

## 📦 Supported Features (The Oxidizable Standard)

### Data Structures

- [x] `interface` → `struct` (with `serde::Serialize/Deserialize`)
- [x] `class` DTOs → `struct`

### Type System

- [x] Primitives: `string`, `number` (`f64`), `boolean`
- [x] Collections: `Array<T>` → `Vec<T>`
- [x] Optionals: `T | undefined` → `Option<T>`
- [x] Generics: `Box<T>` → `Box<T>`

### Logic & Expressions

- [x] `async/await` → `async fn` / `.await`
- [x] `if/else`
- [x] `while` loops
- [x] Unary expressions: `!`, `-`, `+`
- [x] Binary expressions (arithmetic, comparison, logical)
- [x] Array methods: `map`, `filter`, `push`, `join`
- [x] String methods: `replace`, `split`, `trim`, `toUpperCase`

### Frameworks & I/O

- [x] **NestJS Controllers:** `@Get`, `@Post`, `@Body` → `Axum Handlers`
- [x] **HTTP Client:** `axios.get`, `fetch` → `reqwest`
- [x] **JSON:** `JSON.stringify/parse` → `serde_json`

## 🛠 Installation

Prerequisites: Rust 1.75+ and Cargo.

```bash
# Clone the repository
git clone https://github.com/Gefferson-Souza/Tyrus.git
cd Tyrus

# Build the project
cargo build --release

# The binary will be at ./target/release/tyrus
```

## 📖 Usage

### Check Compatibility

Analyzes the project and reports errors or Oxidizable Standard violations.

```bash
./target/release/tyrus check ./path/to/project/index.ts
```

### Build (Transpile)

Generates Rust code in the `tyrus_output` directory.

```bash
./target/release/tyrus build ./path/to/project/index.ts
```

After completion, you'll have a complete Rust project. Simply enter the directory and run `cargo run`.

## 🧪 Testing

The project uses a custom test harness (`tyrus_test_utils`) that guarantees all generated code is compilable.

```bash
# Run the full suite (Unit + Integration + Snapshots)
cargo test --workspace

# Update snapshots if they changed:
cargo insta review
```

## 🤝 Contributing

This is an academic and open-source project. Contributions are welcome, as long as they follow the `CODE_OF_CONDUCT.md` and the guidelines in `CONTRIBUTING.md`.

## 📄 License

MIT License - See [LICENSE](LICENSE) for details.
