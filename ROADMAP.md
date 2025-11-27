# 🗺️ Oxidizer Roadmap

## 🏁 Milestone 0: The Foundation (Current)

- [x] Project Scaffolding (Cargo Workspace)
- [x] CLI Basic Structure (`clap`)
- [x] **Tracer Bullet:** Connect CLI to Parser (Read a file and print "Success")
- [ ] CI/CD Pipeline Setup (GitHub Actions)

## � Milestone 1: The Analyzer (Complete)

- [x] **Analyzer Core:** Implement `ox_analyzer` crate
- [x] **Linter Rules:**
  - [x] Block `any`
  - [x] Block `eval`
  - [x] Block `var`
- [x] **Refactoring:** Compliance with Guidelines (Newtype, Tracing, Tests)
- [x] Implement basic Lints:
  - [x] Ban `any` type
  - [x] Ban `eval`
  - [x] Check for `var` usage
- [ ] Error Reporting with `miette` (Visual spans)

## 🏁 Milestone 2: The Type Transpiler (Complete)

- [x] Convert TS `interface` -> Rust `struct`
- [x] Map primitive types (`string` -> `String`, `number` -> `f64`)
- [x] Auto-derive `Serialize, Deserialize` (Serde)
- [x] Output `.rs` files using `quote!`

## 🏁 Milestone 3: Logic & Functions (Complete)

- [x] Convert simple `fn` declarations
- [x] Basic control flow (`if`, `return`)
- [x] Binary expressions (Math)

## 🚀 Milestone 4: The Modern Stack (Complete)

- [x] **Async/Await Support:**
  - [x] Convert `async function` -> `pub async fn`
  - [x] Unwrap `Promise<T>` return types
  - [x] Convert `await expr` -> `expr.await`
- [x] **Class Support:**
  - [x] Split `class` into `struct` (properties) + `impl` (methods)
  - [x] Convert `constructor` -> `pub fn new() -> Self`
  - [x] Convert `this.prop` -> `self.prop`
  - [x] Add `&self` to instance methods
- [x] **HTTP Client mapping** (`axios` & `fetch` -> `reqwest`)
- [x] **Standard Library Mapping:**
  - [x] Math (max, min, round, floor, ceil, abs, random)
  - [x] String (includes, replace, split, toUpperCase, toLowerCase, trim)
  - [x] Array (push, map, filter, join)
  - [x] JSON (stringify, parse)
  - [x] Console (log, error)
- [x] **Variable Declarations:**
  - [x] `const`/`let` -> `let` bindings
  - [x] Variable initialization support

## ✅ QA & Compliance (Complete)

- [x] **Code Quality:**
  - [x] Fix all compiler warnings
  - [x] Clippy compliance
- [x] **Guidelines.md Compliance:**
  - [x] Newtype Pattern (`FilePath`)
  - [x] Visitor Pattern (AST traversal)
  - [x] Rich error handling (miette)
- [x] **Testing Infrastructure:**
  - [x] Unit tests (8 passing)
  - [x] Snapshot tests (insta)
  - [x] Compilation tests (rustc validation)
  - [x] Complex E2E fixtures

## 📚 Phase 1.5: Standard Library Compliance (The "Shim" Layer) (Complete)

_Goal: 100% coverage of essential JS/TS APIs mapped to Rust equivalents._

### 🧮 Math & Numbers

- [x] **Math Object:**
  - [x] `Math.max/min` -> `f64::max/min` (or `std::cmp`)
  - [x] `Math.round/floor/ceil` -> `.round()/.floor()/.ceil()`
  - [x] `Math.abs` -> `.abs()`
  - [x] `Math.random()` -> `rand::random()` (Requires `rand` crate)

### 🧵 Strings

- [x] **Query:** `.includes()` -> `.contains()`
- [x] **Transformation:**
  - [x] `.toUpperCase/LowerCase()` -> `.to_uppercase/lowercase()`
  - [x] `.replace(a, b)` -> `.replace(a, b)`
  - [x] `.trim()` -> `.trim()`
  - [x] `.split(sep)` -> `.split(sep).collect::<Vec<_>>()`

### 📦 Arrays & Iterators

- [x] **Transformation (Lazy):** `map`, `filter` -> `iter().map()...`
- [x] **Mutation:** `push` -> `push`
- [x] **Utility:** `.join()` -> `.join()`

### 📅 Dates & JSON

- [x] **JSON:**
  - [x] `JSON.stringify` -> `serde_json::to_string`
  - [x] `JSON.parse` -> `serde_json::from_str`
- [ ] **Date:** (Deferred to future update)

### ✅ Execution Verification

- [x] **Execution Tests:** All implemented standard library features are verified by compiling AND executing the generated Rust code.

## 📦 Phase 2: Module System & Project Structure

_Goal: Support multi-file projects with imports and exports._

- [x] **Import/Export Transpilation:**
  - [x] `import { Foo } from './bar'` -> `use crate::bar::Foo;`
  - [x] `export class Bar {}` -> `pub struct Bar ...`
  - [ ] `export default` handling
- [x] **File System Mapping:**
  - [x] Map TS file structure to Rust module structure (`mod.rs`)
  - [x] Handle `index.ts` resolution

## 🧬 Phase 3: Advanced Type System (Complete)

_Goal: Support complex TypeScript types and generics._

- [x] **Arrays:** `number[]` -> `Vec<f64>`
- [x] **Optionals:** `string | undefined` -> `Option<String>`
- [x] **Type References:** `User` -> `User` (Struct linkage)
- [x] **Type Aliases:** `type ID = string` -> `type ID = String;`

## 🧩 Phase 4: Generics (Complete)

_Goal: Support user-defined generics in classes, interfaces, and functions._

- [x] **Generic Interfaces:** `interface Box<T> { value: T; }`
- [x] **Generic Classes:** `class Wrapper<T> { ... }`
- [x] **Generic Functions:** `function identity<T>(arg: T): T { ... }`
- [ ] **Trait Implementation:** `class User implements IPrintable` (Future/Stretch)
